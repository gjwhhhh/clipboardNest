use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{
    plugin::{Builder, TauriPlugin},
    Emitter, Manager, State, Wry,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardContent {
    pub text: Option<String>,
    pub content_type: String,
}

pub struct MonitorState {
    is_monitoring: Mutex<bool>,
    last_content: Mutex<Option<ClipboardContent>>,
    app_handle: Mutex<Option<tauri::AppHandle<Wry>>>,
}

impl Default for MonitorState {
    fn default() -> Self {
        Self {
            is_monitoring: Mutex::new(false),
            last_content: Mutex::new(None),
            app_handle: Mutex::new(None),
        }
    }
}

impl MonitorState {
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置 AppHandle
    pub fn set_app_handle(&self, handle: tauri::AppHandle<Wry>) {
        let mut app_handle = self.app_handle.lock().unwrap();
        *app_handle = Some(handle);
    }

    /// 发送剪切板更新事件
    pub fn emit_clipboard_update(&self, content: ClipboardContent) {
        let mut last_content = self.last_content.lock().unwrap();
        *last_content = Some(content.clone());

        if let Ok(guard) = self.app_handle.lock() {
            if let Some(app_handle) = guard.as_ref() {
                let _ = app_handle.emit("clipboard-updated", content);
            }
        }
    }
}

#[tauri::command]
fn start_monitoring(state: State<'_, Arc<MonitorState>>) -> Result<(), String> {
    let mut is_monitoring = state.is_monitoring.lock().map_err(|e| e.to_string())?;
    *is_monitoring = true;
    log::info!("开始监听剪切板");
    Ok(())
}

#[tauri::command]
fn stop_monitoring(state: State<'_, Arc<MonitorState>>) -> Result<(), String> {
    let mut is_monitoring = state.is_monitoring.lock().map_err(|e| e.to_string())?;
    *is_monitoring = false;
    log::info!("停止监听剪切板");
    Ok(())
}

#[tauri::command]
fn get_text(state: State<'_, Arc<MonitorState>>) -> Result<Option<String>, String> {
    let last_content = state.last_content.lock().map_err(|e| e.to_string())?;
    Ok(last_content.as_ref().and_then(|c| c.text.clone()))
}

#[tauri::command]
fn update_clipboard_content(
    state: State<'_, Arc<MonitorState>>,
    content: ClipboardContent,
) -> Result<(), String> {
    state.emit_clipboard_update(content);
    Ok(())
}

pub fn init() -> TauriPlugin<Wry> {
    Builder::<Wry>::new("clipboard-monitor")
        .invoke_handler(tauri::generate_handler![
            start_monitoring,
            stop_monitoring,
            get_text,
            update_clipboard_content,
        ])
        .setup(|app, _api| {
            let state = Arc::new(MonitorState::new());
            state.set_app_handle(app.app_handle().clone());
            app.manage(state);
            Ok(())
        })
        .build()
}
