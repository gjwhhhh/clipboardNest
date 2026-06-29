use base64::{engine::general_purpose, Engine as _};
use rusqlite::Connection;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};

use clipboard_core::clipboard::{hasher, image_store};
use clipboard_core::storage::models::{ClipboardItem, ClipboardItemCreate, ContentType};
use clipboard_core::storage::repository::{self, UpsertResult};

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

    match item.content_type {
        ContentType::Text | ContentType::Richtext => {
            if item.content.as_deref().unwrap_or_default().is_empty() {
                return Err("剪切板内容为空".to_string());
            }
            log::info!("准备复制文本内容到移动端剪切板: item_id={}", item_id);
        }
        ContentType::Image => {
            let Some(file_path) = item.file_path.as_deref() else {
                return Err("图片尚未保存，无法复制".to_string());
            };
            if !Path::new(file_path).exists() {
                return Err("图片文件不存在".to_string());
            }
            log::info!("准备复制图片内容到移动端剪切板: item_id={}", item_id);
        }
        ContentType::File => {
            return Err("移动端暂不支持复制文件".to_string());
        }
    }

    Ok(())
}

#[tauri::command]
pub fn save_clipboard_image(
    app: tauri::AppHandle,
    state: State<'_, DbState>,
    data_url: String,
) -> Result<(), String> {
    let bytes = decode_data_url(&data_url)?;
    if bytes.is_empty() {
        return Err("图片内容为空".to_string());
    }

    let image = image::load_from_memory(&bytes)
        .map_err(|e| format!("图片解码失败: {}", e))?
        .to_rgba8();
    let (width, height) = image.dimensions();
    let hash = hasher::hash_bytes(&bytes);

    let images_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("images");
    let file_path =
        image_store::save_original_png(&images_dir, &hash, image.as_raw(), width, height)
            .map_err(|e| e.to_string())?;
    let thumbnail_path =
        image_store::save_preview_png(&images_dir, &hash, image.as_raw(), width, height)
            .map_err(|e| e.to_string())?;

    let item = ClipboardItemCreate {
        content_type: ContentType::Image,
        content: None,
        preview: Some(format!("{} x {}", width, height)),
        content_hash: hash,
        file_name: Some("clipboard_image.png".to_string()),
        file_size: Some(bytes.len() as i64),
        file_path: Some(file_path),
        thumbnail_path: Some(thumbnail_path),
        source_app: None,
    };

    {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let max_items = clipboard_core::storage::database::get_setting(&conn, "max_items")
            .ok()
            .flatten()
            .and_then(|value| value.parse().ok())
            .unwrap_or(5000);

        if let UpsertResult::Inserted { deleted_resources } =
            repository::upsert_item(&conn, &item, max_items).map_err(|e| e.to_string())?
        {
            for (file_path, thumbnail_path) in deleted_resources {
                if thumbnail_path.is_empty() {
                    let _ = clipboard_core::clipboard::file_store::delete_file(&file_path);
                } else {
                    let _ = image_store::delete_images(&file_path, &thumbnail_path);
                }
            }
        }
    }
    let _ = app.emit(
        "clipboard-updated",
        serde_json::json!({
            "text": null,
            "content_type": "image"
        }),
    );

    Ok(())
}

#[tauri::command]
pub fn get_file_data_url(app: tauri::AppHandle, file_path: String) -> Result<String, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let canonical_app_data = app_data_dir.canonicalize().map_err(|e| e.to_string())?;
    let canonical_file = Path::new(&file_path)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !canonical_file.starts_with(&canonical_app_data) {
        return Err("不允许读取应用数据目录之外的文件".to_string());
    }

    let bytes = std::fs::read(&canonical_file).map_err(|e| e.to_string())?;
    let mime = match Path::new(&file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "image/png",
    };
    Ok(format!(
        "data:{};base64,{}",
        mime,
        general_purpose::STANDARD.encode(bytes)
    ))
}

fn decode_data_url(data_url: &str) -> Result<Vec<u8>, String> {
    let (_, encoded) = data_url.split_once(',').ok_or("图片 data URL 格式无效")?;
    general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| e.to_string())
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
