// 剪切板监控模块
use arboard::Clipboard;
use log::{error, info};
use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, UNIX_EPOCH};
use tokio::time;
use url::Url;

use tauri::Emitter;

use clipboard_core::clipboard::hasher;
use clipboard_core::clipboard::parser;
use clipboard_core::storage::models::{ClipboardItemCreate, ContentType};
use clipboard_core::storage::repository;

struct LastSeen {
    hash: Option<String>,
    time: Instant,
}

pub struct MonitorState {
    last_seen: Mutex<LastSeen>,
    last_change_count: Mutex<Option<i64>>,
    clipboard: Mutex<Option<Clipboard>>,
    inflight_images: Mutex<HashSet<String>>,
}

impl MonitorState {
    pub fn new() -> Self {
        Self {
            last_seen: Mutex::new(LastSeen {
                hash: None,
                time: Instant::now(),
            }),
            last_change_count: Mutex::new(None),
            clipboard: Mutex::new(None),
            inflight_images: Mutex::new(HashSet::new()),
        }
    }

    /// 标记下一次检测跳过（用于 copy_to_clipboard 场景）
    pub fn skip_next(&self) {
        let mut last_seen = self.last_seen.lock().unwrap();
        last_seen.hash = Some("__skip__".to_string());
    }

    pub fn should_skip(&self, hash: &str) -> bool {
        if self.inflight_images.lock().unwrap().contains(hash) {
            return true;
        }

        let mut last_seen = self.last_seen.lock().unwrap();
        let now = Instant::now();

        // 处理跳过标记
        if let Some(ref prev) = last_seen.hash {
            if prev == "__skip__" {
                last_seen.hash = Some(hash.to_string());
                last_seen.time = now;
                return true;
            }
        }

        if let Some(ref prev_hash) = last_seen.hash {
            if prev_hash == hash && now.duration_since(last_seen.time) < Duration::from_secs(1) {
                return true;
            }
        }

        last_seen.hash = Some(hash.to_string());
        last_seen.time = now;
        false
    }

    pub fn has_processed_change_count(&self, change_count: Option<i64>) -> bool {
        let Some(change_count) = change_count else {
            return false;
        };

        *self.last_change_count.lock().unwrap() == Some(change_count)
    }

    pub fn mark_change_count_processed(&self, change_count: Option<i64>) {
        if let Some(change_count) = change_count {
            *self.last_change_count.lock().unwrap() = Some(change_count);
        }
    }

    pub fn begin_image_generation(&self, hash: &str) -> bool {
        let mut inflight = self.inflight_images.lock().unwrap();
        inflight.insert(hash.to_string())
    }

    pub fn finish_image_generation(&self, hash: &str) {
        self.inflight_images.lock().unwrap().remove(hash);
    }
}

pub async fn start_monitoring(
    conn: Arc<Mutex<rusqlite::Connection>>,
    poll_interval_ms: u64,
    state: Arc<MonitorState>,
    images_dir: std::path::PathBuf,
    files_dir: std::path::PathBuf,
    app_handle: tauri::AppHandle,
) {
    let mut interval = time::interval(Duration::from_millis(poll_interval_ms));

    loop {
        interval.tick().await;
        if let Err(e) = check_clipboard(&conn, &state, &images_dir, &files_dir, &app_handle) {
            error!("剪切板检查错误: {}", e);
        }
    }
}

fn is_image_file_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tiff"
            )
        })
        .unwrap_or(false)
}

fn first_image_path_from_file_list(file_list: Option<&[std::path::PathBuf]>) -> Option<&Path> {
    file_list
        .and_then(|paths| paths.iter().find(|path| is_image_file_path(path.as_path())))
        .map(|path| path.as_path())
}

fn file_name_from_path(path: &Path) -> Option<String> {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
}

fn image_path_from_text(text: Option<&str>) -> Option<std::path::PathBuf> {
    let text = text.map(str::trim).filter(|text| !text.is_empty())?;

    if let Ok(url) = Url::parse(text) {
        if url.scheme() == "file" {
            let path = url.to_file_path().ok()?;
            if path.exists() && is_image_file_path(path.as_path()) {
                return Some(path);
            }
        }
    }

    let path = Path::new(text);
    if path.is_absolute() && path.exists() && is_image_file_path(path) {
        return Some(path.to_path_buf());
    }

    None
}

/// 从剪贴板读取非图像文件路径
fn non_image_file_from_clipboard(
    native_file_url_path: Option<&std::path::PathBuf>,
    file_list: Option<&[std::path::PathBuf]>,
    text: Option<&str>,
) -> Option<std::path::PathBuf> {
    // 优先从 native_file_url 获取
    if let Some(path) = native_file_url_path {
        if path.exists() && !is_image_file_path(path.as_path()) {
            return Some(path.clone());
        }
    }

    // 从 file_list 获取第一个非图像文件
    if let Some(paths) = file_list {
        for path in paths {
            if path.exists() && !is_image_file_path(path.as_path()) {
                return Some(path.clone());
            }
        }
    }

    // 从文本解析文件路径
    file_path_from_text(text)
}

/// 从文本解析文件路径（支持 file:// URL 和绝对路径）
fn file_path_from_text(text: Option<&str>) -> Option<std::path::PathBuf> {
    let text = text.map(str::trim).filter(|text| !text.is_empty())?;

    if let Ok(url) = Url::parse(text) {
        if url.scheme() == "file" {
            let path = url.to_file_path().ok()?;
            if path.exists() && !is_image_file_path(path.as_path()) {
                return Some(path);
            }
        }
    }

    let path = Path::new(text);
    if path.is_absolute() && path.exists() && !is_image_file_path(path) {
        return Some(path.to_path_buf());
    }

    None
}

fn should_prefer_image_over_text(image_available: bool) -> bool {
    image_available
}

fn should_spawn_image_generation(
    existing_item: Option<&clipboard_core::storage::models::ClipboardItem>,
) -> bool {
    match existing_item {
        Some(item) => item.file_path.is_none() || item.thumbnail_path.is_none(),
        None => true,
    }
}

fn build_pending_image_item(
    hash: String,
    file_name: String,
    img: &arboard::ImageData,
    source_app: Option<String>,
) -> ClipboardItemCreate {
    ClipboardItemCreate {
        content_type: ContentType::Image,
        content: None,
        preview: Some(format!("{}×{}", img.width, img.height)),
        content_hash: hash,
        file_name: Some(file_name),
        file_size: Some(img.bytes.len() as i64),
        file_path: None,
        thumbnail_path: None,
        source_app,
    }
}

fn build_pending_file_image_item(
    hash: String,
    file_name: String,
    file_size: Option<i64>,
    dimensions: Option<(u32, u32)>,
    source_app: Option<String>,
) -> ClipboardItemCreate {
    ClipboardItemCreate {
        content_type: ContentType::Image,
        content: None,
        preview: dimensions.map(|(width, height)| format!("{}×{}", width, height)),
        content_hash: hash,
        file_name: Some(file_name),
        file_size,
        file_path: None,
        thumbnail_path: None,
        source_app,
    }
}

fn build_pending_file_item(
    hash: String,
    file_name: String,
    file_size: Option<i64>,
    source_app: Option<String>,
) -> ClipboardItemCreate {
    let preview = file_size
        .map(|size| {
            format!(
                "{} ({})",
                file_name,
                clipboard_core::clipboard::file_store::format_file_size(size)
            )
        })
        .unwrap_or_else(|| file_name.clone());

    ClipboardItemCreate {
        content_type: ContentType::File,
        content: None,
        preview: Some(preview),
        content_hash: hash,
        file_name: Some(file_name),
        file_size,
        file_path: None,
        thumbnail_path: None,
        source_app,
    }
}

fn spawn_file_copy(
    conn: Arc<Mutex<rusqlite::Connection>>,
    files_dir: std::path::PathBuf,
    app_handle: tauri::AppHandle,
    hash: String,
    source_path: std::path::PathBuf,
) {
    std::thread::spawn(move || {
        log::debug!(
            "开始后台复制文件: hash={}, source={}",
            hash,
            source_path.to_string_lossy()
        );

        let stored_path = clipboard_core::clipboard::file_store::copy_file_to_store(
            &files_dir,
            &hash,
            &source_path,
        );
        let stored_path = match stored_path {
            Ok(path) => path,
            Err(err) => {
                log::warn!("复制文件失败: {}", err);
                return;
            }
        };

        let file_size = std::fs::metadata(&source_path)
            .ok()
            .map(|meta| meta.len() as i64);

        log::debug!("文件复制完成: {}", stored_path);

        // 更新数据库中的文件路径
        let conn = conn.lock().unwrap();
        if let Err(err) =
            repository::attach_file_assets_by_hash(&conn, &hash, &stored_path, file_size)
        {
            log::warn!("更新文件路径失败: {}", err);
        }
        drop(conn);

        let _ = app_handle.emit("clipboard-updated", ());
    });
}

fn hash_file_identity(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let metadata = std::fs::metadata(path)?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    Ok(hasher::hash_text(&format!(
        "file:{}:{}:{}",
        canonical_path.to_string_lossy(),
        metadata.len(),
        modified
    )))
}

enum ImageAssetSource {
    File(std::path::PathBuf),
    Rgba {
        bytes: Vec<u8>,
        width: u32,
        height: u32,
    },
}

fn spawn_image_asset_generation(
    conn: Arc<Mutex<rusqlite::Connection>>,
    state: Arc<MonitorState>,
    images_dir: std::path::PathBuf,
    app_handle: tauri::AppHandle,
    hash: String,
    source: ImageAssetSource,
) {
    std::thread::spawn(move || {
        let original_path = match &source {
            ImageAssetSource::File(path) => {
                log::debug!(
                    "开始后台生成图片资源: hash={}, source={}",
                    hash,
                    path.to_string_lossy()
                );
                clipboard_core::clipboard::image_store::copy_original_image(
                    &images_dir,
                    &hash,
                    path,
                )
            }
            ImageAssetSource::Rgba {
                bytes,
                width,
                height,
            } => {
                log::debug!(
                    "开始后台生成图片资源: hash={}, size={}x{}",
                    hash,
                    width,
                    height
                );
                clipboard_core::clipboard::image_store::save_original_png(
                    &images_dir,
                    &hash,
                    bytes,
                    *width,
                    *height,
                )
            }
        };
        let original_path = match original_path {
            Ok(path) => path,
            Err(err) => {
                log::warn!("保存图片原图失败: {}", err);
                state.finish_image_generation(&hash);
                return;
            }
        };
        log::debug!("原图保存完成: {}", original_path);

        let preview_result = match &source {
            ImageAssetSource::File(path) => {
                clipboard_core::clipboard::image_store::save_preview_from_file(
                    &images_dir,
                    &hash,
                    path,
                )
            }
            ImageAssetSource::Rgba {
                bytes,
                width,
                height,
            } => clipboard_core::clipboard::image_store::save_preview_png(
                &images_dir,
                &hash,
                bytes,
                *width,
                *height,
            ),
        };
        let preview_path = match preview_result {
            Ok(path) => Some(path),
            Err(err) => {
                log::warn!("保存图片预览图失败: {}", err);
                Some(original_path.clone())
            }
        };
        log::debug!("预览图保存完成: {:?}", preview_path);

        let file_size = std::fs::metadata(&original_path)
            .ok()
            .map(|meta| meta.len() as i64);

        let updated = {
            let conn = conn.lock().unwrap();
            repository::attach_image_assets_by_hash(
                &conn,
                &hash,
                original_path,
                preview_path,
                file_size,
            )
        };

        match updated {
            Ok(Some(_)) => {
                log::debug!("图片资源路径补写完成: {}", hash);
                state.finish_image_generation(&hash);
                let _ = app_handle.emit("clipboard-updated", ());
            }
            Ok(None) => {
                state.finish_image_generation(&hash);
                log::debug!("图片占位记录已不存在，跳过资源补写: {}", hash);
            }
            Err(err) => {
                state.finish_image_generation(&hash);
                log::warn!("补写图片资源路径失败: {}", err);
            }
        }
    });
}

fn check_clipboard(
    conn: &Arc<Mutex<rusqlite::Connection>>,
    state: &Arc<MonitorState>,
    images_dir: &std::path::Path,
    files_dir: &std::path::Path,
    app_handle: &tauri::AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    let change_count = super::native_macos::clipboard_change_count().ok().flatten();
    if state.has_processed_change_count(change_count) {
        return Ok(());
    }

    // 读取配置的最大项目数
    let max_items: u32 = {
        let db_conn = conn.lock().unwrap();
        crate::storage::database::get_setting(&db_conn, "max_items")
            .ok()
            .flatten()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5000)
    };

    // 获取当前最前面的应用名称
    let source_app = super::native_macos::get_frontmost_app_name();

    // 快速读取剪切板内容，立即释放锁
    let (
        text,
        image_data,
        file_list,
        native_file_url_path,
        text_image_path,
        file_image_path,
        non_image_file_path,
    ) = {
        let mut clipboard_guard = state.clipboard.lock().map_err(|e| e.to_string())?;
        if clipboard_guard.is_none() {
            *clipboard_guard = Some(Clipboard::new()?);
        }
        let clipboard = clipboard_guard.as_mut().unwrap();

        // 先检测文本（macOS 上 get_image() 在某些场景会失败，文本优先更稳定）
        let text: Option<String> = clipboard.get_text().ok().filter(|t| !t.is_empty());

        let native_file_url_path: Option<std::path::PathBuf> =
            super::native_macos::read_file_url_from_clipboard()
                .ok()
                .flatten();

        // 获取文件列表（用于提取原始文件名和降级读取图片）
        let file_list: Option<Vec<std::path::PathBuf>> = clipboard.get().file_list().ok();

        // 检测非图像文件路径
        let non_image_file_path: Option<std::path::PathBuf> = non_image_file_from_clipboard(
            native_file_url_path.as_ref(),
            file_list.as_deref(),
            text.as_deref(),
        );

        let file_image_path: Option<std::path::PathBuf> = native_file_url_path
            .as_ref()
            .filter(|path| is_image_file_path(path.as_path()) && path.exists())
            .cloned()
            .or_else(|| {
                first_image_path_from_file_list(file_list.as_deref()).map(|path| path.to_path_buf())
            })
            .or_else(|| image_path_from_text(text.as_deref()));

        let (image, text_image_path) = if file_image_path.is_some() || non_image_file_path.is_some()
        {
            // 如果已经识别为图像文件或非图像文件，跳过图像数据读取
            (None, Option::<std::path::PathBuf>::None)
        } else {
            let raw_image = clipboard.get_image().ok();

            let native_image = if raw_image.is_none() {
                super::native_macos::read_image_from_clipboard()
                    .ok()
                    .flatten()
            } else {
                None
            };

            let image = raw_image.or_else(|| {
                native_image.as_ref().map(|image| arboard::ImageData {
                    bytes: image.rgba.clone().into(),
                    width: image.width as usize,
                    height: image.height as usize,
                })
            });

            (image, None)
        };

        let text = if file_image_path.is_some()
            || non_image_file_path.is_some()
            || should_prefer_image_over_text(image.is_some())
        {
            None
        } else {
            text
        };

        if let Some(path) = file_image_path.as_ref() {
            log::debug!("从文件路径识别图片: {}", path.to_string_lossy());
        }

        (
            text,
            image,
            file_list,
            native_file_url_path,
            text_image_path,
            file_image_path,
            non_image_file_path,
        )
    };
    // clipboard_guard 在此已释放

    if let Some(text) = text {
        let (content_type, normalized_text) = parser::detect_and_normalize_content(&text);
        let hash = hasher::hash_text(&normalized_text);
        if !state.should_skip(&hash) {
            let preview = parser::generate_preview(&normalized_text, 100);
            let item = ClipboardItemCreate {
                content_type,
                content: Some(normalized_text),
                preview: Some(preview),
                content_hash: hash,
                file_name: None,
                file_size: None,
                file_path: None,
                thumbnail_path: None,
                source_app: source_app.clone(),
            };

            let db_conn = conn.lock().unwrap();
            match repository::upsert_item(&db_conn, &item, max_items)? {
                repository::UpsertResult::Inserted { deleted_resources } => {
                    for (fp, tp) in deleted_resources {
                        if !tp.is_empty() {
                            // 图片资源
                            if let Err(e) =
                                clipboard_core::clipboard::image_store::delete_images(&fp, &tp)
                            {
                                log::warn!("清理图片文件失败: {}", e);
                            }
                        } else {
                            // 文件资源
                            if let Err(e) = clipboard_core::clipboard::file_store::delete_file(&fp)
                            {
                                log::warn!("清理文件失败: {}", e);
                            }
                        }
                    }
                    info!("保存新的剪切板项目: {:?}", item.preview);
                    let _ = app_handle.emit("clipboard-updated", ());
                }
                repository::UpsertResult::Updated => {
                    info!("更新已有剪切板项目: {:?}", item.preview);
                    let _ = app_handle.emit("clipboard-updated", ());
                }
            }
        }
    } else if let Some(image_path) = file_image_path {
        let hash = hash_file_identity(&image_path)?;
        if !state.should_skip(&hash) {
            let file_name = file_name_from_path(&image_path)
                .unwrap_or_else(|| "clipboard_image.png".to_string());
            let file_size = std::fs::metadata(&image_path)
                .ok()
                .map(|meta| meta.len() as i64);
            let dimensions = image::image_dimensions(&image_path).ok();
            let item = build_pending_file_image_item(
                hash.clone(),
                file_name,
                file_size,
                dimensions,
                source_app.clone(),
            );

            let db_conn = conn.lock().unwrap();
            let existing_item = repository::get_item_by_hash(&db_conn, &hash)?;
            let needs_asset_generation = should_spawn_image_generation(existing_item.as_ref());
            match repository::upsert_item(&db_conn, &item, max_items)? {
                repository::UpsertResult::Inserted { deleted_resources } => {
                    for (fp, tp) in deleted_resources {
                        if !tp.is_empty() {
                            // 图片资源
                            if let Err(e) =
                                clipboard_core::clipboard::image_store::delete_images(&fp, &tp)
                            {
                                log::warn!("清理图片文件失败: {}", e);
                            }
                        } else {
                            // 文件资源
                            if let Err(e) = clipboard_core::clipboard::file_store::delete_file(&fp)
                            {
                                log::warn!("清理文件失败: {}", e);
                            }
                        }
                    }
                    log::debug!("插入图片文件占位记录: {}", image_path.to_string_lossy());
                }
                repository::UpsertResult::Updated => {
                    log::debug!("更新图片文件占位记录: {}", image_path.to_string_lossy());
                }
            }
            drop(db_conn);
            let _ = app_handle.emit("clipboard-updated", ());
            if !needs_asset_generation {
                state.mark_change_count_processed(change_count);
                return Ok(());
            }
            let current_item = {
                let db_conn = conn.lock().unwrap();
                repository::get_item_by_hash(&db_conn, &hash)?
            };
            if !should_spawn_image_generation(current_item.as_ref()) {
                state.mark_change_count_processed(change_count);
                return Ok(());
            }
            if !state.begin_image_generation(&hash) {
                state.mark_change_count_processed(change_count);
                return Ok(());
            }
            spawn_image_asset_generation(
                Arc::clone(conn),
                Arc::clone(state),
                images_dir.to_path_buf(),
                app_handle.clone(),
                hash,
                ImageAssetSource::File(image_path),
            );
        }
    } else if let Some(file_path) = non_image_file_path {
        // 处理非图像文件
        let hash = hash_file_identity(&file_path)?;
        if !state.should_skip(&hash) {
            let file_name =
                file_name_from_path(&file_path).unwrap_or_else(|| "unknown_file".to_string());
            let file_size = std::fs::metadata(&file_path)
                .ok()
                .map(|meta| meta.len() as i64);
            let item =
                build_pending_file_item(hash.clone(), file_name, file_size, source_app.clone());

            let db_conn = conn.lock().unwrap();
            match repository::upsert_item(&db_conn, &item, max_items)? {
                repository::UpsertResult::Inserted { deleted_resources } => {
                    for (fp, tp) in deleted_resources {
                        if !tp.is_empty() {
                            // 图片资源
                            if let Err(e) =
                                clipboard_core::clipboard::image_store::delete_images(&fp, &tp)
                            {
                                log::warn!("清理图片文件失败: {}", e);
                            }
                        } else {
                            // 文件资源
                            if let Err(e) = clipboard_core::clipboard::file_store::delete_file(&fp)
                            {
                                log::warn!("清理文件失败: {}", e);
                            }
                        }
                    }
                    log::debug!("插入文件占位记录: {}", file_path.to_string_lossy());
                }
                repository::UpsertResult::Updated => {
                    log::debug!("更新文件占位记录: {}", file_path.to_string_lossy());
                }
            }
            drop(db_conn);
            let _ = app_handle.emit("clipboard-updated", ());

            // 后台复制文件
            spawn_file_copy(
                Arc::clone(conn),
                files_dir.to_path_buf(),
                app_handle.clone(),
                hash,
                file_path,
            );
        }
    } else if let Some(img) = image_data {
        let hash = hasher::hash_bytes(&img.bytes);
        if !state.should_skip(&hash) {
            let file_name = file_list
                .as_ref()
                .and_then(|paths| paths.first())
                .and_then(|p| file_name_from_path(p.as_path()))
                .or_else(|| {
                    native_file_url_path
                        .as_deref()
                        .and_then(file_name_from_path)
                })
                .or_else(|| text_image_path.as_deref().and_then(file_name_from_path))
                .unwrap_or_else(|| "clipboard_image.png".to_string());

            let item = build_pending_image_item(hash.clone(), file_name, &img, source_app.clone());

            let db_conn = conn.lock().unwrap();
            let existing_item = repository::get_item_by_hash(&db_conn, &hash)?;
            let needs_asset_generation = should_spawn_image_generation(existing_item.as_ref());
            match repository::upsert_item(&db_conn, &item, max_items)? {
                repository::UpsertResult::Inserted { deleted_resources } => {
                    for (fp, tp) in deleted_resources {
                        if !tp.is_empty() {
                            // 图片资源
                            if let Err(e) =
                                clipboard_core::clipboard::image_store::delete_images(&fp, &tp)
                            {
                                log::warn!("清理图片文件失败: {}", e);
                            }
                        } else {
                            // 文件资源
                            if let Err(e) = clipboard_core::clipboard::file_store::delete_file(&fp)
                            {
                                log::warn!("清理文件失败: {}", e);
                            }
                        }
                    }
                    log::debug!("插入图片占位记录: {}×{}", img.width, img.height);
                }
                repository::UpsertResult::Updated => {
                    log::debug!("更新图片占位记录: {}×{}", img.width, img.height);
                }
            }
            drop(db_conn);
            let _ = app_handle.emit("clipboard-updated", ());
            if !needs_asset_generation {
                state.mark_change_count_processed(change_count);
                return Ok(());
            }
            let current_item = {
                let db_conn = conn.lock().unwrap();
                repository::get_item_by_hash(&db_conn, &hash)?
            };
            if !should_spawn_image_generation(current_item.as_ref()) {
                state.mark_change_count_processed(change_count);
                return Ok(());
            }
            if !state.begin_image_generation(&hash) {
                state.mark_change_count_processed(change_count);
                return Ok(());
            }
            spawn_image_asset_generation(
                Arc::clone(conn),
                Arc::clone(state),
                images_dir.to_path_buf(),
                app_handle.clone(),
                hash,
                ImageAssetSource::Rgba {
                    bytes: img.bytes.to_vec(),
                    width: img.width as u32,
                    height: img.height as u32,
                },
            );
        }
    }

    state.mark_change_count_processed(change_count);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_应该跳过重复内容() {
        let state = MonitorState::new();
        assert!(!state.should_skip("hash1"));
        assert!(state.should_skip("hash1"));
        assert!(!state.should_skip("hash2"));
    }

    #[test]
    fn test_图片文件名文本应优先识别为图片() {
        assert!(should_prefer_image_over_text(true));
    }

    #[test]
    fn test_存在图片数据时应优先按图片处理() {
        assert!(should_prefer_image_over_text(true));
    }

    #[test]
    fn test_大写扩展名图片路径也应识别为图片() {
        assert!(is_image_file_path(Path::new("/tmp/IMG_0900_副本.PNG")));
    }

    #[test]
    fn test_文件列表降级读取应支持大写扩展名图片() {
        let file_list = vec![PathBuf::from("/tmp/IMG_0900_副本.PNG")];

        assert_eq!(
            first_image_path_from_file_list(Some(file_list.as_slice()))
                .map(|path| path.to_string_lossy().to_string()),
            Some("/tmp/IMG_0900_副本.PNG".to_string()),
        );
    }

    #[test]
    fn test_文件URL文本应解析成图片路径() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let image_path = temp_dir.path().join("finder-copy.png");
        std::fs::write(&image_path, b"not-used").unwrap();

        let url = format!("file://{}", image_path.to_string_lossy());
        assert_eq!(image_path_from_text(Some(&url)), Some(image_path));
    }

    #[test]
    fn test_图片首次入库应为无路径占位记录() {
        let item = build_pending_image_item(
            "pending-hash".to_string(),
            "image0.png".to_string(),
            &arboard::ImageData {
                bytes: std::borrow::Cow::Owned(vec![0u8; 16]),
                width: 2,
                height: 2,
            },
            None,
        );

        assert_eq!(item.content_type, ContentType::Image);
        assert_eq!(item.file_name.as_deref(), Some("image0.png"));
        assert_eq!(item.preview.as_deref(), Some("2×2"));
        assert_eq!(item.file_path, None);
        assert_eq!(item.thumbnail_path, None);
    }

    #[test]
    fn test_已有完整资源的图片不应重复生成() {
        let complete_item = clipboard_core::storage::models::ClipboardItem {
            id: 1,
            content_type: ContentType::Image,
            content: None,
            preview: Some("2×2".to_string()),
            content_hash: "same-image".to_string(),
            file_name: Some("same-image.png".to_string()),
            file_size: Some(16),
            file_path: Some("/tmp/same-image.png".to_string()),
            thumbnail_path: Some("/tmp/same-image_preview.png".to_string()),
            source_app: None,
            is_pinned: false,
            is_favorite: false,
            created_at: "2026-06-01 00:00:00".to_string(),
            updated_at: "2026-06-01 00:00:00".to_string(),
        };

        assert!(!should_spawn_image_generation(Some(&complete_item)));
        assert!(should_spawn_image_generation(None));
    }

    #[test]
    fn test_进行中的图片任务应跳过重复调度() {
        let state = MonitorState::new();
        assert!(state.begin_image_generation("same-image"));
        assert!(state.should_skip("same-image"));
        state.finish_image_generation("same-image");
        assert!(!state.should_skip("same-image"));
    }

    #[test]
    fn test_剪贴板变更号只在处理成功后标记() {
        let state = MonitorState::new();

        assert!(!state.has_processed_change_count(Some(10)));
        assert!(!state.has_processed_change_count(Some(10)));
        state.mark_change_count_processed(Some(10));
        assert!(state.has_processed_change_count(Some(10)));
        assert!(!state.has_processed_change_count(Some(11)));
        assert!(!state.has_processed_change_count(None));
    }
}
