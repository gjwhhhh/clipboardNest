use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use super::{SyncData, SyncMessage};

/// 同步服务器
pub struct SyncServer {
    port: u16,
    data: Arc<Mutex<SyncData>>,
}

impl SyncServer {
    pub fn new(port: u16, data: Arc<Mutex<SyncData>>) -> Self {
        Self { port, data }
    }

    /// 启动同步服务器
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        log::info!("同步服务器启动在端口 {}", self.port);

        let data = self.data.clone();

        tokio::spawn(async move {
            loop {
                if let Ok((mut socket, addr)) = listener.accept().await {
                    log::info!("新连接: {}", addr);
                    let data = data.clone();

                    tokio::spawn(async move {
                        let mut buf = [0u8; 4096];
                        loop {
                            match socket.read(&mut buf).await {
                                Ok(0) => break,
                                Ok(n) => {
                                    if let Ok(msg) =
                                        serde_json::from_slice::<SyncMessage>(&buf[..n])
                                    {
                                        let response = handle_message(&data, msg).await;
                                        let _ = socket
                                            .write_all(&serde_json::to_vec(&response).unwrap())
                                            .await;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    });
                }
            }
        });

        Ok(())
    }
}

async fn handle_message(data: &Arc<Mutex<SyncData>>, msg: SyncMessage) -> SyncMessage {
    match msg.message_type.as_str() {
        "request_sync" => {
            let data = data.lock().unwrap();
            SyncMessage {
                message_type: "sync_data".to_string(),
                data: serde_json::to_value(&*data).unwrap(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            }
        }
        "sync_data" => {
            if let Ok(new_data) = serde_json::from_value::<SyncData>(msg.data) {
                let mut data = data.lock().unwrap();
                // 合并数据（简单策略：去重合并）
                for item in new_data.items {
                    if !data.items.iter().any(|i| i.id == item.id) {
                        data.items.push(item);
                    }
                }
            }
            SyncMessage {
                message_type: "sync_complete".to_string(),
                data: serde_json::Value::Null,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            }
        }
        _ => SyncMessage {
            message_type: "error".to_string(),
            data: serde_json::json!({"message": "Unknown message type"}),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        },
    }
}
