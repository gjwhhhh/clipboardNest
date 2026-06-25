use log::info;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use clipboard_core::clipboard::hasher;
use clipboard_core::clipboard::parser;
use clipboard_core::storage::models::ClipboardItemCreate;
use clipboard_core::storage::repository;

struct LastSeen {
    hash: Option<String>,
    time: Instant,
}

pub struct MonitorState {
    last_seen: Mutex<LastSeen>,
}

impl MonitorState {
    pub fn new() -> Self {
        Self {
            last_seen: Mutex::new(LastSeen {
                hash: None,
                time: Instant::now(),
            }),
        }
    }

    pub fn should_skip(&self, hash: &str) -> bool {
        let mut last_seen = self.last_seen.lock().unwrap();
        let now = Instant::now();

        if let Some(ref prev_hash) = last_seen.hash {
            if prev_hash == hash && now.duration_since(last_seen.time) < Duration::from_secs(1) {
                return true;
            }
        }

        last_seen.hash = Some(hash.to_string());
        last_seen.time = now;
        false
    }
}

pub struct MonitorStateWrapper(pub Arc<MonitorState>);

/// 处理从原生层收到的剪切板内容
pub fn handle_clipboard_content(
    conn: &Arc<Mutex<rusqlite::Connection>>,
    state: &Arc<MonitorState>,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if content.is_empty() {
        return Ok(());
    }

    let (detected_type, normalized_text) = parser::detect_and_normalize_content(content);
    let hash = hasher::hash_text(&normalized_text);

    if state.should_skip(&hash) {
        return Ok(());
    }

    let preview = parser::generate_preview(&normalized_text, 100);
    let item = ClipboardItemCreate {
        content_type: detected_type,
        content: Some(normalized_text),
        preview: Some(preview),
        content_hash: hash,
        file_name: None,
        file_size: None,
        file_path: None,
        thumbnail_path: None,
        source_app: None,
    };

    let db_conn = conn.lock().unwrap();
    // 检查重复
    if !repository::is_duplicate(&db_conn, &item.content_hash)? {
        repository::insert_item(&db_conn, &item)?;
        info!("保存新的剪切板项目: {:?}", item.preview);
    }

    Ok(())
}
