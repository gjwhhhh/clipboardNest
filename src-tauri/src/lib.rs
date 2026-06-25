mod clipboard;
mod commands;
mod hotkey;
mod storage;
mod tray;
mod window;

use commands::clipboard::DbState;
use log::info;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub fn run() {
    // 初始化日志，默认关闭，通过 RUST_LOG 环境变量开启（如 RUST_LOG=info）
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("off")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let db_path = app
                .path()
                .app_data_dir()
                .expect("无法获取应用数据目录")
                .join("clipboard.db");
            let conn = storage::database::initialize(&db_path).expect("无法初始化数据库");
            storage::database::ensure_platform_defaults(&conn, hotkey::default_hotkey())
                .expect("无法设置默认配置");

            let initial_hotkey = storage::database::get_setting(&conn, "hotkey")
                .ok()
                .flatten()
                .unwrap_or_else(|| hotkey::default_hotkey().to_string());

            let db_conn = Arc::new(Mutex::new(conn));
            app.manage(DbState(db_conn.clone()));

            // 创建监控器状态并注册到 Tauri
            let monitor_state = Arc::new(clipboard::monitor::MonitorState::new());
            app.manage(commands::clipboard::MonitorStateWrapper(
                monitor_state.clone(),
            ));

            // 启动时清理过期数据
            {
                let db_conn_guard = db_conn.lock().unwrap();
                let retention_days: u32 =
                    storage::database::get_setting(&db_conn_guard, "retention_days")
                        .ok()
                        .flatten()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(30);
                let max_items: u32 = storage::database::get_setting(&db_conn_guard, "max_items")
                    .ok()
                    .flatten()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(5000);

                // 按时间清理
                if let Ok(deleted) =
                    storage::repository::delete_old_items(&db_conn_guard, retention_days)
                {
                    for (fp, tp) in deleted {
                        if !tp.is_empty() {
                            let _ = clipboard_core::clipboard::image_store::delete_images(&fp, &tp);
                        } else {
                            let _ = clipboard_core::clipboard::file_store::delete_file(&fp);
                        }
                    }
                }

                // 按数量清理
                if let Ok(deleted) =
                    storage::repository::delete_excess_items(&db_conn_guard, max_items)
                {
                    for (fp, tp) in deleted {
                        if !tp.is_empty() {
                            let _ = clipboard_core::clipboard::image_store::delete_images(&fp, &tp);
                        } else {
                            let _ = clipboard_core::clipboard::file_store::delete_file(&fp);
                        }
                    }
                }
            }

            // 启动剪切板监控，传入共享状态
            info!("启动剪切板监控...");
            let app_data_dir = app.path().app_data_dir().expect("无法获取应用数据目录");
            let images_dir = app_data_dir.join("images");
            let files_dir = app_data_dir.join("files");
            let monitor_state_clone = monitor_state.clone();
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(clipboard::monitor::start_monitoring(
                    db_conn,
                    500,
                    monitor_state_clone,
                    images_dir,
                    files_dir,
                    app_handle,
                ));
            });

            // 创建快捷键管理器并注册到 Tauri
            let hotkey_mgr = Arc::new(hotkey::HotkeyManager::new(&initial_hotkey));
            app.manage(commands::settings::HotkeyManagerWrapper(hotkey_mgr));

            tray::create_tray(app)?;

            // 注册全局快捷键
            if let Err(e) = hotkey::register_hotkey(app.handle(), &initial_hotkey) {
                log::warn!("快捷键注册失败: {}", e);
            }

            info!("应用启动完成");
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
            commands::clipboard::hide_window,
            commands::clipboard::get_image_data_url,
            commands::settings::get_settings,
            commands::settings::update_setting,
            commands::settings::update_hotkey,
        ])
        .run(tauri::generate_context!())
        .expect("运行 tauri 应用程序时出错");
}
