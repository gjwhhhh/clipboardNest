use rusqlite::Connection;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tauri::State;

use clipboard_core::storage::models::{ClipboardItem, ContentType};
use clipboard_core::storage::repository;

pub struct DbState(pub Arc<Mutex<Connection>>);

#[tauri::command]
pub fn get_clipboard_history(
    state: State<'_, DbState>,
    content_type: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<ClipboardItem>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let ct = content_type.and_then(|s| ContentType::from_str(&s).ok());
    repository::get_history(&conn, ct, limit.unwrap_or(100), offset.unwrap_or(0))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_clipboard(
    state: State<'_, DbState>,
    query: String,
    limit: Option<u32>,
) -> Result<Vec<ClipboardItem>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    repository::search_items(&conn, &query, limit.unwrap_or(200)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn copy_to_clipboard(state: State<'_, DbState>, item_id: i64) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let item = repository::get_item(&conn, item_id)
        .map_err(|e| e.to_string())?
        .ok_or("项目未找到")?;

    if let Some(content) = item.content {
        // 移动端使用原生插件写入剪切板
        // 这里需要调用原生插件
        log::info!("复制内容到剪切板: {}", content);
    }

    Ok(())
}

#[tauri::command]
pub fn delete_clipboard_item(state: State<'_, DbState>, item_id: i64) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    repository::delete_item(&conn, item_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pin_clipboard_item(
    state: State<'_, DbState>,
    item_id: i64,
    pinned: bool,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    repository::set_pinned(&conn, item_id, pinned).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn favorite_clipboard_item(
    state: State<'_, DbState>,
    item_id: i64,
    favorite: bool,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    repository::set_favorite(&conn, item_id, favorite).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_all_history(state: State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    repository::clear_all(&conn).map_err(|e| e.to_string())
}
