// 剪切板命令
use rusqlite::Connection;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tauri::{Manager, State};

use crate::clipboard::monitor::MonitorState;
use clipboard_core::storage::models::{ClipboardItem, ContentType};
use clipboard_core::storage::repository;

pub struct DbState(pub Arc<Mutex<Connection>>);

/// 监控器状态包装器，用于 Tauri 状态管理
pub struct MonitorStateWrapper(pub Arc<MonitorState>);

fn copy_image_item_to_clipboard(item: &ClipboardItem) -> Result<(), String> {
    if let Some(ref file_path) = item.file_path {
        if image_mime_type(file_path) == "image/png" {
            match std::fs::read(file_path) {
                Ok(png_bytes) => {
                    match crate::clipboard::native_macos::write_png_to_clipboard(&png_bytes) {
                        Ok(true) => return Ok(()),
                        Ok(false) => {}
                        Err(err) => {
                            log::debug!("原生写入图片剪贴板失败，回退到 arboard: {}", err);
                        }
                    }
                }
                Err(err) => {
                    log::debug!("读取原图失败，回退到 arboard: {}", err);
                }
            }
        }
    }

    let candidate_paths: Vec<String> = {
        let mut paths = Vec::new();
        if let Some(ref fp) = item.file_path {
            paths.push(fp.clone());
        }
        if let Some(ref tp) = item.thumbnail_path {
            paths.push(tp.clone());
        }
        paths
    };

    let mut result = None;
    for path in &candidate_paths {
        match clipboard_core::clipboard::image_store::load_image_as_rgba(path) {
            Ok(data) => {
                log::debug!("成功加载图片: {}", path);
                result = Some((path.clone(), data));
                break;
            }
            Err(e) => {
                log::debug!("加载图片失败 ({}): {}", path, e);
                continue;
            }
        }
    }

    match result {
        Some((_path, (rgba, w, h))) => {
            let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
            let image_data = arboard::ImageData {
                bytes: rgba.into(),
                width: w as usize,
                height: h as usize,
            };
            clipboard.set_image(image_data).map_err(|e| e.to_string())
        }
        None => Err("所有图片文件都无法加载".to_string()),
    }
}

fn image_mime_type(file_path: &str) -> &'static str {
    match Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("tif") | Some("tiff") => "image/tiff",
        _ => "image/png",
    }
}

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
pub fn copy_to_clipboard(
    state: State<'_, DbState>,
    monitor: State<'_, MonitorStateWrapper>,
    item_id: i64,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let item = repository::get_item(&conn, item_id)
        .map_err(|e| e.to_string())?
        .ok_or("项目未找到")?;

    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;

    // 先标记跳过，避免监控器在写入剪贴板后、标记前轮询到新内容
    monitor.0.skip_next();

    match item.content_type {
        ContentType::Text | ContentType::Richtext => {
            if let Some(content) = item.content {
                clipboard.set_text(content).map_err(|e| e.to_string())?;
            }
        }
        ContentType::Image => {
            copy_image_item_to_clipboard(&item)?;
        }
        ContentType::File => {
            if let Some(ref file_path) = item.file_path {
                crate::clipboard::native_macos::write_file_to_clipboard(file_path)
                    .map_err(|e| e.to_string())?;
            } else {
                return Err("文件尚未保存，无法复制".to_string());
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn delete_clipboard_item(state: State<'_, DbState>, item_id: i64) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    // 获取项目信息，用于清理关联的文件
    let item = repository::get_item(&conn, item_id).map_err(|e| e.to_string())?;

    if let Some(ref item) = item {
        if item.content_type == ContentType::Image {
            if let (Some(ref fp), Some(ref tp)) = (&item.file_path, &item.thumbnail_path) {
                if let Err(e) = clipboard_core::clipboard::image_store::delete_images(fp, tp) {
                    log::warn!("清理图片文件失败: {}", e);
                }
            }
        } else if item.content_type == ContentType::File {
            if let Some(ref fp) = item.file_path {
                if let Err(e) = clipboard_core::clipboard::file_store::delete_file(fp) {
                    log::warn!("清理文件失败: {}", e);
                }
            }
        }
    }

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

    // 先清理图片文件
    let image_items = repository::get_all_image_items(&conn).map_err(|e| e.to_string())?;
    for (fp, tp) in image_items {
        if let Err(e) = clipboard_core::clipboard::image_store::delete_images(&fp, &tp) {
            log::warn!("清理图片文件失败: {}", e);
        }
    }

    // 清理文件类型记录的存储文件
    let file_items = repository::get_all_file_items(&conn).map_err(|e| e.to_string())?;
    for fp in file_items {
        if let Err(e) = clipboard_core::clipboard::file_store::delete_file(&fp) {
            log::warn!("清理文件失败: {}", e);
        }
    }

    repository::clear_all(&conn).map_err(|e| e.to_string())?;
    repository::vacuum_database(&conn).map_err(|e| e.to_string())
}

/// 隐藏主窗口
#[tauri::command]
pub fn hide_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 获取图片的 base64 data URL，用于前端预览
#[tauri::command]
pub fn get_image_data_url(file_path: String) -> Result<String, String> {
    use base64::Engine;
    use std::fs;

    log::debug!("get_image_data_url 请求: {}", file_path);

    if !fs::metadata(&file_path).is_ok() {
        log::warn!("图片文件不存在: {}", file_path);
        return Err(format!("文件不存在: {}", file_path));
    }

    let data = fs::read(&file_path).map_err(|e| format!("读取图片失败: {}", e))?;
    log::debug!("读取图片成功，大小: {} bytes", data.len());
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    let url = format!("data:{};base64,{}", image_mime_type(&file_path), b64);
    log::debug!("生成 data URL 长度: {}", url.len());
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clipboard_core::storage::models::ContentType;

    #[test]
    fn test_图片data_url应按扩展名设置mime() {
        assert_eq!(image_mime_type("/tmp/a.png"), "image/png");
        assert_eq!(image_mime_type("/tmp/a.JPG"), "image/jpeg");
        assert_eq!(image_mime_type("/tmp/a.webp"), "image/webp");
    }

    #[test]
    #[ignore = "需要可用的 macOS 系统剪贴板"]
    fn test_图片历史记录可写回系统剪贴板() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let rgba = vec![255u8, 0, 0, 255];
        let file_path = clipboard_core::clipboard::image_store::save_original_png(
            temp_dir.path(),
            "copy-test",
            &rgba,
            1,
            1,
        )
        .unwrap();
        let thumbnail_path = clipboard_core::clipboard::image_store::save_preview_png(
            temp_dir.path(),
            "copy-test",
            &rgba,
            1,
            1,
        )
        .unwrap();

        let item = ClipboardItem {
            id: 1,
            content_type: ContentType::Image,
            content: None,
            preview: Some("1×1".to_string()),
            content_hash: "hash".to_string(),
            file_name: Some("copy-test.png".to_string()),
            file_size: Some(4),
            file_path: Some(file_path),
            thumbnail_path: Some(thumbnail_path),
            source_app: None,
            is_pinned: false,
            is_favorite: false,
            created_at: "2026-05-31 00:00:00".to_string(),
            updated_at: "2026-05-31 00:00:00".to_string(),
        };

        copy_image_item_to_clipboard(&item).unwrap();

        let image = crate::clipboard::native_macos::read_image_from_clipboard()
            .unwrap()
            .expect("系统剪贴板里应该有图片");
        assert_eq!(image.width, 1);
        assert_eq!(image.height, 1);
        assert_eq!(image.rgba, rgba);
    }
}
