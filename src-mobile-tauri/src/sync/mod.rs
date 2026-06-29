pub mod client;
pub mod discovery;
pub mod server;

use clipboard_core::storage::models::{ClipboardItem, ClipboardItemCreate, ContentType};
use clipboard_core::storage::repository::{self, UpsertResult};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const MAX_SYNC_MESSAGE_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDevice {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub last_seen: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMessage {
    pub message_type: String,
    pub data: serde_json::Value,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncData {
    pub items: Vec<ClipboardItem>,
    pub settings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SyncReport {
    pub inserted: u32,
    pub updated: u32,
    pub skipped: u32,
}

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn export_sync_data(conn: &Connection) -> Result<SyncData, rusqlite::Error> {
    let items = repository::get_history(conn, None, 5000, 0)?
        .into_iter()
        .filter(|item| matches!(item.content_type, ContentType::Text | ContentType::Richtext))
        .collect();

    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let settings = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(SyncData { items, settings })
}

pub fn merge_sync_data(
    conn: &Connection,
    data: SyncData,
    max_items: u32,
) -> Result<SyncReport, rusqlite::Error> {
    let mut report = SyncReport::default();

    for item in data.items {
        if !matches!(item.content_type, ContentType::Text | ContentType::Richtext) {
            report.skipped += 1;
            continue;
        }

        let create = ClipboardItemCreate {
            content_type: item.content_type.clone(),
            content: item.content.clone(),
            preview: item.preview.clone(),
            content_hash: item.content_hash.clone(),
            file_name: None,
            file_size: None,
            file_path: None,
            thumbnail_path: None,
            source_app: item.source_app.clone(),
        };

        if repository::is_duplicate(conn, &create.content_hash)? {
            report.skipped += 1;
            continue;
        }

        match repository::upsert_item(conn, &create, max_items)? {
            UpsertResult::Inserted { .. } => report.inserted += 1,
            UpsertResult::Updated => report.skipped += 1,
        }
    }

    Ok(report)
}

pub fn max_items_from_settings(conn: &Connection) -> u32 {
    clipboard_core::storage::database::get_setting(conn, "max_items")
        .ok()
        .flatten()
        .and_then(|value| value.parse().ok())
        .unwrap_or(5000)
}

pub async fn write_message(
    stream: &mut tokio::net::TcpStream,
    message: &SyncMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bytes = serde_json::to_vec(message)?;
    let len = bytes.len() as u32;
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(&bytes).await?;
    Ok(())
}

pub async fn read_message(
    stream: &mut tokio::net::TcpStream,
) -> Result<SyncMessage, Box<dyn std::error::Error + Send + Sync>> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len == 0 || len > MAX_SYNC_MESSAGE_BYTES {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid sync message size").into());
    }

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    Ok(serde_json::from_slice(&buf)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        clipboard_core::storage::database::run_migrations(&conn).unwrap();
        conn
    }

    fn text_item(hash: &str, content: &str) -> ClipboardItem {
        ClipboardItem {
            id: 0,
            content_type: ContentType::Text,
            content: Some(content.to_string()),
            preview: Some(content.to_string()),
            content_hash: hash.to_string(),
            file_name: None,
            file_size: None,
            file_path: None,
            thumbnail_path: None,
            source_app: None,
            is_pinned: false,
            is_favorite: false,
            created_at: "2026-01-01 00:00:00".to_string(),
            updated_at: "2026-01-01 00:00:00".to_string(),
        }
    }

    #[test]
    fn merge_sync_data_inserts_new_items_and_skips_duplicates() {
        let conn = setup_db();
        let data = SyncData {
            items: vec![text_item("hash-a", "hello"), text_item("hash-a", "hello")],
            settings: HashMap::new(),
        };

        let report = merge_sync_data(&conn, data, 100).unwrap();

        assert_eq!(report.inserted, 1);
        assert_eq!(report.skipped, 1);
        assert_eq!(repository::get_item_count(&conn).unwrap(), 1);
    }

    #[test]
    fn merge_sync_data_skips_non_text_assets() {
        let conn = setup_db();
        let mut image = text_item("hash-image", "");
        image.content_type = ContentType::Image;
        image.content = None;
        let data = SyncData {
            items: vec![image],
            settings: HashMap::new(),
        };

        let report = merge_sync_data(&conn, data, 100).unwrap();

        assert_eq!(report.inserted, 0);
        assert_eq!(report.skipped, 1);
        assert_eq!(repository::get_item_count(&conn).unwrap(), 0);
    }
}
