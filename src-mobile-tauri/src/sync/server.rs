use std::io;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

use super::{
    export_sync_data, max_items_from_settings, merge_sync_data, now_secs, read_message,
    write_message, SyncMessage,
};

pub async fn start_server(
    port: u16,
    db_conn: Arc<Mutex<Connection>>,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    log::info!("同步服务器启动在端口 {}", port);

    Ok(tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    log::info!("同步连接: {}", addr);
                    let db_conn = db_conn.clone();
                    tokio::spawn(async move {
                        if let Err(error) = handle_socket(socket, db_conn).await {
                            log::warn!("同步连接处理失败: {}", error);
                        }
                    });
                }
                Err(error) => {
                    log::warn!("同步服务器 accept 失败: {}", error);
                    break;
                }
            }
        }
    }))
}

async fn handle_socket(
    mut socket: TcpStream,
    db_conn: Arc<Mutex<Connection>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let msg = match read_message(&mut socket).await {
            Ok(msg) => msg,
            Err(_) => break,
        };

        let response = match msg.message_type.as_str() {
            "request_sync" => {
                let data = {
                    let conn = db_conn
                        .lock()
                        .map_err(|e| io::Error::other(e.to_string()))?;
                    export_sync_data(&conn)?
                };
                SyncMessage {
                    message_type: "sync_data".to_string(),
                    data: serde_json::to_value(data)?,
                    timestamp: now_secs(),
                }
            }
            "sync_data" => {
                let data = serde_json::from_value(msg.data)?;
                let report = {
                    let conn = db_conn
                        .lock()
                        .map_err(|e| io::Error::other(e.to_string()))?;
                    let max_items = max_items_from_settings(&conn);
                    merge_sync_data(&conn, data, max_items)?
                };
                SyncMessage {
                    message_type: "sync_complete".to_string(),
                    data: serde_json::to_value(report)?,
                    timestamp: now_secs(),
                }
            }
            _ => SyncMessage {
                message_type: "error".to_string(),
                data: serde_json::json!({"message": "Unknown message type"}),
                timestamp: now_secs(),
            },
        };

        write_message(&mut socket, &response).await?;
    }

    Ok(())
}
