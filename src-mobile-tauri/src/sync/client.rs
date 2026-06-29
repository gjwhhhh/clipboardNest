use std::io;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tokio::net::TcpStream;

use super::{
    export_sync_data, max_items_from_settings, merge_sync_data, now_secs, read_message,
    write_message, SyncDevice, SyncMessage, SyncReport,
};

pub async fn sync_with_device(
    db_conn: Arc<Mutex<Connection>>,
    device: SyncDevice,
) -> Result<SyncReport, Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = TcpStream::connect(format!("{}:{}", device.address, device.port)).await?;

    let request = SyncMessage {
        message_type: "request_sync".to_string(),
        data: serde_json::Value::Null,
        timestamp: now_secs(),
    };
    write_message(&mut stream, &request).await?;

    let response = read_message(&mut stream).await?;
    let mut report = SyncReport::default();
    if response.message_type == "sync_data" {
        let remote_data = serde_json::from_value(response.data)?;
        let conn = db_conn
            .lock()
            .map_err(|e| io::Error::other(e.to_string()))?;
        let max_items = max_items_from_settings(&conn);
        report = merge_sync_data(&conn, remote_data, max_items)?;
    }

    let local_data = {
        let conn = db_conn
            .lock()
            .map_err(|e| io::Error::other(e.to_string()))?;
        export_sync_data(&conn)?
    };
    let push = SyncMessage {
        message_type: "sync_data".to_string(),
        data: serde_json::to_value(local_data)?,
        timestamp: now_secs(),
    };
    write_message(&mut stream, &push).await?;
    let _ = read_message(&mut stream).await;

    Ok(report)
}
