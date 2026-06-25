// 设置命令
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

use super::clipboard::DbState;

pub struct HotkeyManagerWrapper(pub Arc<crate::hotkey::HotkeyManager>);

#[tauri::command]
pub fn get_settings(state: State<'_, DbState>) -> Result<HashMap<String, String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;

    let settings = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(settings)
}

#[tauri::command]
pub fn update_setting(state: State<'_, DbState>, key: String, value: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_hotkey(
    state: State<'_, DbState>,
    hotkey_mgr: State<'_, HotkeyManagerWrapper>,
    app_handle: tauri::AppHandle,
    new_hotkey: String,
) -> Result<(), String> {
    let new_hotkey = new_hotkey.trim().to_string();

    // 验证快捷键格式
    crate::hotkey::parse_hotkey(&new_hotkey).map_err(|e| format!("无效的快捷键格式: {}", e))?;

    let old_hotkey = hotkey_mgr.0.current();

    // 重新注册
    crate::hotkey::reregister_hotkey(&app_handle, &old_hotkey, &new_hotkey)
        .map_err(|e| format!("注册快捷键失败: {}", e))?;

    // 更新内存状态
    hotkey_mgr.0.update(&new_hotkey);

    // 持久化到数据库
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('hotkey', ?1)",
        rusqlite::params![new_hotkey],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database;

    /// 创建内存测试数据库并执行迁移
    fn setup_test_db() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        database::run_migrations(&conn).unwrap();
        // 需要调用 ensure_platform_defaults 来插入默认快捷键
        database::ensure_platform_defaults(&conn, "Cmd+Shift+V").unwrap();
        conn
    }

    /// 通过底层 SQL 查询获取设置，等同于 get_settings 命令的逻辑
    fn query_settings(conn: &rusqlite::Connection) -> Result<HashMap<String, String>, String> {
        let mut stmt = conn
            .prepare("SELECT key, value FROM settings")
            .map_err(|e| e.to_string())?;
        let settings = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        Ok(settings)
    }

    /// 通过底层 SQL 写入设置，等同于 update_setting 命令的逻辑
    fn upsert_setting(conn: &rusqlite::Connection, key: &str, value: &str) -> Result<(), String> {
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    #[test]
    fn test_获取默认设置() {
        let conn = setup_test_db();
        let settings = query_settings(&conn).unwrap();

        // 验证迁移脚本中定义的默认设置
        assert_eq!(settings.get("retention_days").unwrap(), "30");
        assert_eq!(settings.get("max_items").unwrap(), "5000");
        assert_eq!(settings.get("poll_interval_ms").unwrap(), "500");
        assert_eq!(settings.get("launch_at_login").unwrap(), "true");
        assert_eq!(settings.get("theme").unwrap(), "system");
        // hotkey 由 ensure_platform_defaults 插入
        assert!(settings.get("hotkey").is_some());
    }

    #[test]
    fn test_更新设置() {
        let conn = setup_test_db();

        // 更新 retention_days 从 "30" 改为 "60"
        let result = upsert_setting(&conn, "retention_days", "60");
        assert!(result.is_ok());

        // 验证更新后的值
        let settings = query_settings(&conn).unwrap();
        assert_eq!(settings.get("retention_days").unwrap(), "60");
    }

    #[test]
    fn test_更新并读取多个设置() {
        let conn = setup_test_db();

        // 连续更新多个设置
        upsert_setting(&conn, "retention_days", "90").unwrap();
        upsert_setting(&conn, "max_items", "10000").unwrap();
        upsert_setting(&conn, "theme", "dark").unwrap();

        let settings = query_settings(&conn).unwrap();
        assert_eq!(settings.get("retention_days").unwrap(), "90");
        assert_eq!(settings.get("max_items").unwrap(), "10000");
        assert_eq!(settings.get("theme").unwrap(), "dark");
        // 未修改的设置应保持默认值
        assert_eq!(settings.get("poll_interval_ms").unwrap(), "500");
    }

    #[test]
    fn test_插入新设置键() {
        let conn = setup_test_db();

        // INSERT OR REPLACE 可以插入新的键
        let result = upsert_setting(&conn, "custom_key", "custom_value");
        assert!(result.is_ok());

        let settings = query_settings(&conn).unwrap();
        assert_eq!(settings.get("custom_key").unwrap(), "custom_value");
    }

    #[test]
    fn test_更新设置键覆盖旧值() {
        let conn = setup_test_db();

        // 先写入一个值
        upsert_setting(&conn, "my_key", "value_a").unwrap();
        // 再次写入同一个键，应覆盖旧值
        upsert_setting(&conn, "my_key", "value_b").unwrap();

        let settings = query_settings(&conn).unwrap();
        assert_eq!(settings.get("my_key").unwrap(), "value_b");
        // 确保只有一个同键条目（INSERT OR REPLACE 语义）
        assert_eq!(settings.len(), 7); // 5 个迁移默认设置 + 1 个 hotkey + 1 个自定义
    }

    #[test]
    fn test_数据库迁移幂等性() {
        let conn = setup_test_db();
        // 多次运行迁移不应出错
        database::run_migrations(&conn).unwrap();
        database::run_migrations(&conn).unwrap();

        let settings = query_settings(&conn).unwrap();
        assert_eq!(settings.get("retention_days").unwrap(), "30");
    }
}
