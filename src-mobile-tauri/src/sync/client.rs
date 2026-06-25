use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use super::{SyncData, SyncDevice, SyncMessage};

/// 同步客户端
pub struct SyncClient {
    data: Arc<Mutex<SyncData>>,
}

impl SyncClient {
    pub fn new(data: Arc<Mutex<SyncData>>) -> Self {
        Self { data }
    }

    /// 与指定设备同步
    pub async fn sync_with(&self, device: &SyncDevice) -> Result<(), Box<dyn std::error::Error>> {
        let mut stream = TcpStream::connect(format!("{}:{}", device.address, device.port)).await?;

        // 发送同步请求
        let request = SyncMessage {
            message_type: "request_sync".to_string(),
            data: serde_json::Value::Null,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        stream.write_all(&serde_json::to_vec(&request)?).await?;

        // 读取响应
        let mut buf = [0u8; 4096];
        let n = stream.read(&mut buf).await?;

        if let Ok(response) = serde_json::from_slice::<SyncMessage>(&buf[..n]) {
            if response.message_type == "sync_data" {
                if let Ok(remote_data) = serde_json::from_value::<SyncData>(response.data) {
                    self.merge_data(remote_data);
                }
            }
        }

        Ok(())
    }

    /// 合并远程数据
    fn merge_data(&self, remote_data: SyncData) {
        let mut data = self.data.lock().unwrap();
        for item in remote_data.items {
            if !data.items.iter().any(|i| i.id == item.id) {
                data.items.push(item);
            }
        }
    }
}
