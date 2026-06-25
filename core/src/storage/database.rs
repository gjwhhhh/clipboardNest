// SQLite 数据库模块（core 版本，无 Tauri 依赖）
use rusqlite::Connection;
use std::path::Path;

const MIGRATIONS: &[(&str, &str)] = &[
    (
        "001_initial.sql",
        include_str!("../../migrations/001_initial.sql"),
    ),
    (
        "002_fts5.sql",
        include_str!("../../migrations/002_fts5.sql"),
    ),
    (
        "003_updated_at_index.sql",
        include_str!("../../migrations/003_updated_at_index.sql"),
    ),
];

/// 初始化数据库：打开连接、设置 PRAGMA、运行迁移
pub fn initialize(db_path: &Path) -> Result<Connection, Box<dyn std::error::Error>> {
    // 确保父目录存在
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    conn.execute_batch("PRAGMA busy_timeout=5000;")?;
    run_migrations(&conn)?;
    Ok(conn)
}

/// 运行数据库迁移
/// 按内嵌顺序应用所有迁移
pub fn run_migrations(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            name TEXT PRIMARY KEY,
            applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
    )?;

    for (name, sql) in MIGRATIONS {
        let already_applied: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM _migrations WHERE name = ?1",
            rusqlite::params![name],
            |row| row.get(0),
        )?;

        if !already_applied {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO _migrations (name) VALUES (?1)",
                rusqlite::params![name],
            )?;
            log::info!("已应用迁移: {}", name);
        }
    }

    Ok(())
}

/// 确保默认设置存在
/// `default_hotkey`：平台特定的默认快捷键字符串（如 "Cmd+Shift+V"）
pub fn ensure_platform_defaults(
    conn: &Connection,
    default_hotkey: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('hotkey', ?1)",
        rusqlite::params![default_hotkey],
    )?;

    #[cfg(not(target_os = "macos"))]
    conn.execute(
        "UPDATE settings SET value = ?1 WHERE key = 'hotkey' AND value = 'Cmd+Shift+V'",
        rusqlite::params![default_hotkey],
    )?;

    Ok(())
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query(rusqlite::params![key])?;

    match rows.next()? {
        Some(row) => row.get(0).map(Some),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_初始化创建表() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 验证 clipboard_items 表存在
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='clipboard_items'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // 验证 settings 表存在
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='settings'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_默认设置已插入() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let value: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key='retention_days'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(value, "30");
    }

    #[test]
    fn test_默认快捷键写入() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        ensure_platform_defaults(&conn, "Cmd+Shift+V").unwrap();

        let value = get_setting(&conn, "hotkey").unwrap().unwrap();
        assert_eq!(value, "Cmd+Shift+V");
    }

    #[test]
    fn test_initialize_创建数据库文件() {
        let tmp = tempfile::TempDir::new().unwrap();
        let db_path = tmp.path().join("test.db");
        let conn = initialize(&db_path).unwrap();

        // 验证表存在
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='clipboard_items'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
