mod clipboard_monitor;
mod commands;
pub mod sync;

use std::sync::{Arc, Mutex};
use tauri::{Listener, Manager};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_monitor::init())
        .setup(|app| {
            // 初始化数据库
            let db_path = app
                .path()
                .app_data_dir()
                .expect("无法获取应用数据目录")
                .join("clipboard.db");

            let conn =
                clipboard_core::storage::database::initialize(&db_path).expect("无法初始化数据库");

            // 设置默认配置
            clipboard_core::storage::database::ensure_platform_defaults(&conn, "Cmd+V")
                .expect("无法设置默认配置");

            // 管理数据库连接
            let db_conn = Arc::new(Mutex::new(conn));
            app.manage(commands::clipboard::DbState(db_conn.clone()));

            // 启动剪切板监控
            let monitor_state = Arc::new(clipboard_monitor::MonitorState::new());
            app.manage(clipboard_monitor::MonitorStateWrapper(
                monitor_state.clone(),
            ));

            // 监听剪切板更新事件
            let db_conn_clone = db_conn.clone();
            let monitor_state_clone = monitor_state.clone();

            app.listen("clipboard-updated", move |event: tauri::Event| {
                if let Ok(content) = serde_json::from_str::<
                    tauri_plugin_clipboard_monitor::ClipboardContent,
                >(event.payload())
                {
                    if let Some(text) = content.text {
                        let _ = clipboard_monitor::handle_clipboard_content(
                            &db_conn_clone,
                            &monitor_state_clone,
                            &text,
                        );
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::clipboard::get_clipboard_history,
            commands::clipboard::search_clipboard,
            commands::clipboard::copy_to_clipboard,
            commands::clipboard::delete_clipboard_item,
            commands::clipboard::pin_clipboard_item,
            commands::clipboard::favorite_clipboard_item,
            commands::clipboard::clear_all_history,
            commands::settings::get_settings,
            commands::settings::update_setting,
        ])
        .run(tauri::generate_context!())
        .expect("运行移动端应用时出错");
}
