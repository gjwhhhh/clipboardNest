use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Emitter, Manager, State, Wry,
};

#[cfg(mobile)]
use tauri::plugin::PluginHandle;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.clipboard.plugin";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_clipboard_monitor);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardContent {
    pub text: Option<String>,
    pub content_type: String,
}

#[cfg(mobile)]
#[derive(Debug, Deserialize)]
struct TextResponse {
    text: Option<String>,
}

#[cfg(mobile)]
#[derive(Debug, Deserialize)]
struct ImageResponse {
    data_url: Option<String>,
}

pub struct ClipboardMonitor {
    app: AppHandle<Wry>,
    is_monitoring: Mutex<bool>,
    last_content: Mutex<Option<ClipboardContent>>,
    #[cfg(mobile)]
    mobile_plugin_handle: PluginHandle<Wry>,
}

impl ClipboardMonitor {
    fn new(app: AppHandle<Wry>, #[cfg(mobile)] mobile_plugin_handle: PluginHandle<Wry>) -> Self {
        Self {
            app,
            is_monitoring: Mutex::new(false),
            last_content: Mutex::new(None),
            #[cfg(mobile)]
            mobile_plugin_handle,
        }
    }

    fn set_monitoring(&self, enabled: bool) -> Result<(), String> {
        let mut is_monitoring = self.is_monitoring.lock().map_err(|e| e.to_string())?;
        *is_monitoring = enabled;
        Ok(())
    }

    fn emit_clipboard_update(&self, content: ClipboardContent) -> Result<(), String> {
        let mut last_content = self.last_content.lock().map_err(|e| e.to_string())?;
        *last_content = Some(content.clone());
        self.app
            .emit("clipboard-updated", content)
            .map_err(|e| e.to_string())
    }

    #[cfg(not(mobile))]
    fn last_text(&self) -> Result<Option<String>, String> {
        let last_content = self.last_content.lock().map_err(|e| e.to_string())?;
        Ok(last_content.as_ref().and_then(|c| c.text.clone()))
    }
}

#[tauri::command]
fn start_monitoring(state: State<'_, ClipboardMonitor>) -> Result<(), String> {
    state.set_monitoring(true)?;

    #[cfg(mobile)]
    state
        .mobile_plugin_handle
        .run_mobile_plugin::<()>("startMonitoring", ())
        .map_err(|e| e.to_string())?;

    log::info!("开始监听剪切板");
    Ok(())
}

#[tauri::command]
fn stop_monitoring(state: State<'_, ClipboardMonitor>) -> Result<(), String> {
    state.set_monitoring(false)?;

    #[cfg(mobile)]
    state
        .mobile_plugin_handle
        .run_mobile_plugin::<()>("stopMonitoring", ())
        .map_err(|e| e.to_string())?;

    log::info!("停止监听剪切板");
    Ok(())
}

#[tauri::command]
fn get_text(state: State<'_, ClipboardMonitor>) -> Result<Option<String>, String> {
    #[cfg(mobile)]
    {
        let response = state
            .mobile_plugin_handle
            .run_mobile_plugin::<TextResponse>("getText", ())
            .map_err(|e| e.to_string())?;
        return Ok(response.text);
    }

    #[cfg(not(mobile))]
    state.last_text()
}

#[tauri::command]
fn set_text(state: State<'_, ClipboardMonitor>, text: String) -> Result<(), String> {
    #[cfg(mobile)]
    state
        .mobile_plugin_handle
        .run_mobile_plugin::<()>("setText", text.clone())
        .map_err(|e| e.to_string())?;

    state.emit_clipboard_update(ClipboardContent {
        text: Some(text),
        content_type: "text".to_string(),
    })
}

#[tauri::command]
fn get_image(state: State<'_, ClipboardMonitor>) -> Result<Option<String>, String> {
    #[cfg(mobile)]
    {
        let response = state
            .mobile_plugin_handle
            .run_mobile_plugin::<ImageResponse>("getImage", ())
            .map_err(|e| e.to_string())?;
        return Ok(response.data_url);
    }

    #[cfg(not(mobile))]
    let _ = state;
    #[cfg(not(mobile))]
    Ok(None)
}

#[tauri::command]
fn set_image(state: State<'_, ClipboardMonitor>, data_url: String) -> Result<(), String> {
    #[cfg(mobile)]
    state
        .mobile_plugin_handle
        .run_mobile_plugin::<()>("setImage", data_url)
        .map_err(|e| e.to_string())?;

    #[cfg(not(mobile))]
    let _ = state;
    #[cfg(not(mobile))]
    let _ = data_url;

    Ok(())
}

#[tauri::command]
fn update_clipboard_content(
    state: State<'_, ClipboardMonitor>,
    content: ClipboardContent,
) -> Result<(), String> {
    state.emit_clipboard_update(content)
}

pub fn init() -> TauriPlugin<Wry> {
    Builder::<Wry>::new("clipboard-monitor")
        .invoke_handler(tauri::generate_handler![
            start_monitoring,
            stop_monitoring,
            get_text,
            set_text,
            get_image,
            set_image,
            update_clipboard_content,
        ])
        .setup(|app, api| {
            #[cfg(not(mobile))]
            let _ = api;

            #[cfg(target_os = "android")]
            let mobile_plugin_handle =
                api.register_android_plugin(PLUGIN_IDENTIFIER, "ClipboardMonitorPlugin")?;

            #[cfg(target_os = "ios")]
            let mobile_plugin_handle = api.register_ios_plugin(init_plugin_clipboard_monitor)?;

            app.manage(ClipboardMonitor::new(
                app.app_handle().clone(),
                #[cfg(mobile)]
                mobile_plugin_handle,
            ));
            Ok(())
        })
        .build()
}
