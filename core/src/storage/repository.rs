// 仓库 CRUD 操作
use super::models::{ClipboardItem, ClipboardItemCreate, ContentType};
use rusqlite::{params, Connection};
use std::str::FromStr;

pub enum UpsertResult {
    Inserted {
        deleted_resources: Vec<(String, String)>,
    },
    Updated,
}

pub fn insert_item(conn: &Connection, item: &ClipboardItemCreate) -> Result<i64, rusqlite::Error> {
    conn.execute(
        "INSERT INTO clipboard_items (content_type, content, preview, content_hash, file_name, file_size, file_path, thumbnail_path, source_app)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            item.content_type.as_str(),
            item.content,
            item.preview,
            item.content_hash,
            item.file_name,
            item.file_size,
            item.file_path,
            item.thumbnail_path,
            item.source_app,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_item(conn: &Connection, id: i64) -> Result<Option<ClipboardItem>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, content_type, content, preview, content_hash, file_name, file_size, file_path, thumbnail_path, source_app, is_pinned, is_favorite, created_at, updated_at
         FROM clipboard_items WHERE id = ?1"
    )?;

    let result = stmt.query_row(params![id], |row| {
        Ok(ClipboardItem {
            id: row.get(0)?,
            content_type: ContentType::from_str(&row.get::<_, String>(1)?)
                .ok()
                .unwrap_or(ContentType::Text),
            content: row.get(2)?,
            preview: row.get(3)?,
            content_hash: row.get(4)?,
            file_name: row.get(5)?,
            file_size: row.get(6)?,
            file_path: row.get(7)?,
            thumbnail_path: row.get(8)?,
            source_app: row.get(9)?,
            is_pinned: row.get(10)?,
            is_favorite: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    });

    match result {
        Ok(item) => Ok(Some(item)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn get_history(
    conn: &Connection,
    content_type: Option<ContentType>,
    limit: u32,
    offset: u32,
) -> Result<Vec<ClipboardItem>, rusqlite::Error> {
    let (query, params_vec): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match content_type {
        Some(ct) => (
            "SELECT id, content_type, content, preview, content_hash, file_name, file_size, file_path, thumbnail_path, source_app, is_pinned, is_favorite, created_at, updated_at
             FROM clipboard_items
             WHERE content_type = ?1
             ORDER BY is_pinned DESC, updated_at DESC, id DESC
             LIMIT ?2 OFFSET ?3",
            vec![
                Box::new(ct.as_str()) as Box<dyn rusqlite::types::ToSql>,
                Box::new(limit),
                Box::new(offset),
            ],
        ),
        None => (
            "SELECT id, content_type, content, preview, content_hash, file_name, file_size, file_path, thumbnail_path, source_app, is_pinned, is_favorite, created_at, updated_at
             FROM clipboard_items
             ORDER BY is_pinned DESC, updated_at DESC, id DESC
             LIMIT ?1 OFFSET ?2",
            vec![
                Box::new(limit) as Box<dyn rusqlite::types::ToSql>,
                Box::new(offset),
            ],
        ),
    };

    let mut stmt = conn.prepare(query)?;
    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        params_vec.iter().map(|p| p.as_ref()).collect();

    let items = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(ClipboardItem {
                id: row.get(0)?,
                content_type: ContentType::from_str(&row.get::<_, String>(1)?)
                    .ok()
                    .unwrap_or(ContentType::Text),
                content: row.get(2)?,
                preview: row.get(3)?,
                content_hash: row.get(4)?,
                file_name: row.get(5)?,
                file_size: row.get(6)?,
                file_path: row.get(7)?,
                thumbnail_path: row.get(8)?,
                source_app: row.get(9)?,
                is_pinned: row.get(10)?,
                is_favorite: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(items)
}

/// 搜索剪切板项目
/// 优先使用 FTS5 全文索引，降级到 LIKE 查询
pub fn search_items(
    conn: &Connection,
    query: &str,
    limit: u32,
) -> Result<Vec<ClipboardItem>, rusqlite::Error> {
    // 构建 FTS5 查询语句
    let fts_query = query
        .split_whitespace()
        .map(|w| format!("\"{}\"", w.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ");

    let stmt = conn.prepare(
        "SELECT c.id, c.content_type, c.content, c.preview, c.content_hash,
                c.file_name, c.file_size, c.file_path, c.thumbnail_path, c.source_app,
                c.is_pinned, c.is_favorite, c.created_at, c.updated_at
         FROM clipboard_items c
         INNER JOIN clipboard_items_fts f ON c.id = f.rowid
         WHERE clipboard_items_fts MATCH ?1
         ORDER BY c.is_pinned DESC, c.updated_at DESC, c.id DESC
         LIMIT ?2",
    );

    match stmt {
        Ok(mut stmt) => {
            let result = stmt
                .query_map(rusqlite::params![fts_query, limit], |row| {
                    Ok(ClipboardItem {
                        id: row.get(0)?,
                        content_type: ContentType::from_str(&row.get::<_, String>(1)?)
                            .ok()
                            .unwrap_or(ContentType::Text),
                        content: row.get(2)?,
                        preview: row.get(3)?,
                        content_hash: row.get(4)?,
                        file_name: row.get(5)?,
                        file_size: row.get(6)?,
                        file_path: row.get(7)?,
                        thumbnail_path: row.get(8)?,
                        source_app: row.get(9)?,
                        is_pinned: row.get(10)?,
                        is_favorite: row.get(11)?,
                        created_at: row.get(12)?,
                        updated_at: row.get(13)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>();

            match result {
                Ok(items) if !items.is_empty() => Ok(items),
                _ => search_items_like(conn, query, limit),
            }
        }
        Err(_) => search_items_like(conn, query, limit),
    }
}

/// 降级搜索：使用 LIKE 查询
fn search_items_like(
    conn: &Connection,
    query: &str,
    limit: u32,
) -> Result<Vec<ClipboardItem>, rusqlite::Error> {
    let search_pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT id, content_type, content, preview, content_hash, file_name, file_size, file_path, thumbnail_path, source_app, is_pinned, is_favorite, created_at, updated_at
         FROM clipboard_items
         WHERE preview LIKE ?1 OR file_name LIKE ?1
         ORDER BY is_pinned DESC, updated_at DESC, id DESC
         LIMIT ?2"
    )?;

    let items = stmt
        .query_map(rusqlite::params![search_pattern, limit], |row| {
            Ok(ClipboardItem {
                id: row.get(0)?,
                content_type: ContentType::from_str(&row.get::<_, String>(1)?)
                    .ok()
                    .unwrap_or(ContentType::Text),
                content: row.get(2)?,
                preview: row.get(3)?,
                content_hash: row.get(4)?,
                file_name: row.get(5)?,
                file_size: row.get(6)?,
                file_path: row.get(7)?,
                thumbnail_path: row.get(8)?,
                source_app: row.get(9)?,
                is_pinned: row.get(10)?,
                is_favorite: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(items)
}

pub fn delete_item(conn: &Connection, id: i64) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM clipboard_items WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn set_pinned(conn: &Connection, id: i64, pinned: bool) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE clipboard_items SET is_pinned = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![pinned, id],
    )?;
    Ok(())
}

pub fn set_favorite(conn: &Connection, id: i64, favorite: bool) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE clipboard_items SET is_favorite = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![favorite, id],
    )?;
    Ok(())
}

pub fn get_item_by_hash(
    conn: &Connection,
    content_hash: &str,
) -> Result<Option<ClipboardItem>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, content_type, content, preview, content_hash, file_name, file_size, file_path, thumbnail_path, source_app, is_pinned, is_favorite, created_at, updated_at
         FROM clipboard_items
         WHERE content_hash = ?1
         ORDER BY updated_at DESC, id DESC
         LIMIT 1"
    )?;

    let result = stmt.query_row(params![content_hash], |row| {
        Ok(ClipboardItem {
            id: row.get(0)?,
            content_type: ContentType::from_str(&row.get::<_, String>(1)?)
                .ok()
                .unwrap_or(ContentType::Text),
            content: row.get(2)?,
            preview: row.get(3)?,
            content_hash: row.get(4)?,
            file_name: row.get(5)?,
            file_size: row.get(6)?,
            file_path: row.get(7)?,
            thumbnail_path: row.get(8)?,
            source_app: row.get(9)?,
            is_pinned: row.get(10)?,
            is_favorite: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    });

    match result {
        Ok(item) => Ok(Some(item)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// 检查内容哈希是否已存在
pub fn is_duplicate(conn: &Connection, content_hash: &str) -> Result<bool, rusqlite::Error> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM clipboard_items WHERE content_hash = ?1",
        params![content_hash],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

fn update_item_by_hash(
    conn: &Connection,
    existing: &ClipboardItem,
    item: &ClipboardItemCreate,
) -> Result<i64, rusqlite::Error> {
    let preview = item.preview.clone().or_else(|| existing.preview.clone());
    let file_name = item
        .file_name
        .clone()
        .or_else(|| existing.file_name.clone());
    let file_size = item.file_size.or(existing.file_size);
    let file_path = item
        .file_path
        .clone()
        .or_else(|| existing.file_path.clone());
    let thumbnail_path = item
        .thumbnail_path
        .clone()
        .or_else(|| existing.thumbnail_path.clone());
    let source_app = item
        .source_app
        .clone()
        .or_else(|| existing.source_app.clone());

    conn.execute(
        "UPDATE clipboard_items
         SET content_type = ?1,
             content = ?2,
             preview = ?3,
             file_name = ?4,
             file_size = ?5,
             file_path = ?6,
             thumbnail_path = ?7,
             source_app = ?8,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = ?9",
        params![
            item.content_type.as_str(),
            item.content,
            preview,
            file_name,
            file_size,
            file_path,
            thumbnail_path,
            source_app,
            existing.id,
        ],
    )?;

    Ok(existing.id)
}

pub fn attach_image_assets_by_hash(
    conn: &Connection,
    content_hash: &str,
    file_path: String,
    preview_path: Option<String>,
    file_size: Option<i64>,
) -> Result<Option<i64>, rusqlite::Error> {
    let Some(existing) = get_item_by_hash(conn, content_hash)? else {
        return Ok(None);
    };

    conn.execute(
        "UPDATE clipboard_items
         SET file_path = ?1,
             thumbnail_path = ?2,
             file_size = COALESCE(?3, file_size)
         WHERE id = ?4",
        params![file_path, preview_path, file_size, existing.id],
    )?;

    Ok(Some(existing.id))
}

pub fn clear_all(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM clipboard_items", [])?;
    conn.execute(
        "DELETE FROM sqlite_sequence WHERE name = 'clipboard_items'",
        [],
    )?;
    Ok(())
}

/// 获取所有图片项目的文件路径（用于清理文件）
pub fn get_all_image_items(conn: &Connection) -> Result<Vec<(String, String)>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT file_path, thumbnail_path FROM clipboard_items
         WHERE content_type = 'image'
         AND file_path IS NOT NULL AND thumbnail_path IS NOT NULL",
    )?;

    let items = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    Ok(items)
}

/// 获取所有文件类型的存储路径（用于清理）
pub fn get_all_file_items(conn: &Connection) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT file_path FROM clipboard_items
         WHERE content_type = 'file'
         AND file_path IS NOT NULL",
    )?;

    let items = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    Ok(items)
}

/// 补写文件资源路径（类似 attach_image_assets_by_hash，但无 thumbnail_path）
pub fn attach_file_assets_by_hash(
    conn: &Connection,
    content_hash: &str,
    file_path: &str,
    file_size: Option<i64>,
) -> Result<Option<i64>, rusqlite::Error> {
    let Some(existing) = get_item_by_hash(conn, content_hash)? else {
        return Ok(None);
    };

    conn.execute(
        "UPDATE clipboard_items
         SET file_path = ?1,
             file_size = COALESCE(?2, file_size)
         WHERE id = ?3",
        params![file_path, file_size, existing.id],
    )?;

    Ok(Some(existing.id))
}

pub fn get_item_count(conn: &Connection) -> Result<i64, rusqlite::Error> {
    conn.query_row("SELECT COUNT(*) FROM clipboard_items", [], |row| row.get(0))
}

pub fn delete_oldest(
    conn: &Connection,
    count: u32,
) -> Result<Vec<(String, String)>, rusqlite::Error> {
    // 先查询即将删除的图片项的文件路径
    let image_paths = {
        let mut stmt = conn.prepare(
            "SELECT file_path, thumbnail_path FROM clipboard_items
             WHERE id IN (SELECT id FROM clipboard_items WHERE is_pinned = FALSE ORDER BY updated_at ASC, id ASC LIMIT ?1)
             AND content_type = 'image'
             AND file_path IS NOT NULL AND thumbnail_path IS NOT NULL"
        )?;
        let rows = stmt.query_map(params![count], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        rows.filter_map(|r| r.ok()).collect::<Vec<_>>()
    };

    // 查询即将删除的文件类型项的路径
    let file_paths = {
        let mut stmt = conn.prepare(
            "SELECT file_path FROM clipboard_items
             WHERE id IN (SELECT id FROM clipboard_items WHERE is_pinned = FALSE ORDER BY updated_at ASC, id ASC LIMIT ?1)
             AND content_type = 'file'
             AND file_path IS NOT NULL"
        )?;
        let rows = stmt.query_map(params![count], |row| row.get::<_, String>(0))?;
        rows.filter_map(|r| r.ok()).collect::<Vec<_>>()
    };

    conn.execute(
        "DELETE FROM clipboard_items WHERE id IN (SELECT id FROM clipboard_items WHERE is_pinned = FALSE ORDER BY updated_at ASC, id ASC LIMIT ?1)",
        params![count],
    )?;

    // 合并图片路径和文件路径返回
    let mut all_paths = image_paths;
    for fp in file_paths {
        all_paths.push((fp, String::new()));
    }

    Ok(all_paths)
}

pub fn upsert_item(
    conn: &Connection,
    item: &ClipboardItemCreate,
    max_items: u32,
) -> Result<UpsertResult, rusqlite::Error> {
    if let Some(existing) = get_item_by_hash(conn, &item.content_hash)? {
        update_item_by_hash(conn, &existing, item)?;
        return Ok(UpsertResult::Updated);
    }

    insert_item(conn, item)?;

    let count = get_item_count(conn)?;
    let mut deleted_resources = Vec::new();
    if count > max_items as i64 {
        let excess = (count - max_items as i64) as u32;
        deleted_resources = delete_oldest(conn, excess)?;
    }

    Ok(UpsertResult::Inserted { deleted_resources })
}

/// 删除超过指定天数的非置顶记录，返回被删除项目的文件路径
pub fn delete_old_items(
    conn: &Connection,
    days: u32,
) -> Result<Vec<(String, String)>, rusqlite::Error> {
    let cutoff = chrono::Local::now() - chrono::Duration::days(days as i64);
    let cutoff_str = cutoff.format("%Y-%m-%d %H:%M:%S").to_string();

    // 查询即将删除的图片资源路径
    let image_paths = {
        let mut stmt = conn.prepare(
            "SELECT file_path, thumbnail_path FROM clipboard_items
             WHERE is_pinned = FALSE AND updated_at < ?1
             AND content_type = 'image'
             AND file_path IS NOT NULL AND thumbnail_path IS NOT NULL",
        )?;
        let rows = stmt.query_map(params![cutoff_str], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        rows.filter_map(|r| r.ok()).collect::<Vec<_>>()
    };

    // 查询即将删除的文件资源路径
    let file_paths = {
        let mut stmt = conn.prepare(
            "SELECT file_path FROM clipboard_items
             WHERE is_pinned = FALSE AND updated_at < ?1
             AND content_type = 'file'
             AND file_path IS NOT NULL",
        )?;
        let rows = stmt.query_map(params![cutoff_str], |row| row.get::<_, String>(0))?;
        rows.filter_map(|r| r.ok()).collect::<Vec<_>>()
    };

    // 删除过期记录
    conn.execute(
        "DELETE FROM clipboard_items WHERE is_pinned = FALSE AND updated_at < ?1",
        params![cutoff_str],
    )?;

    // 合并返回
    let mut all_paths = image_paths;
    for fp in file_paths {
        all_paths.push((fp, String::new()));
    }

    Ok(all_paths)
}

/// 删除超过最大数量的非置顶记录，返回被删除项目的文件路径
pub fn delete_excess_items(
    conn: &Connection,
    max_items: u32,
) -> Result<Vec<(String, String)>, rusqlite::Error> {
    let count = get_item_count(conn)?;
    if count <= max_items as i64 {
        return Ok(Vec::new());
    }
    let excess = (count - max_items as i64) as u32;
    delete_oldest(conn, excess)
}

pub fn vacuum_database(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch("VACUUM")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL;").unwrap();
        database::run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_插入和获取项目() {
        let conn = setup_test_db();
        let item = ClipboardItemCreate {
            content_type: ContentType::Text,
            content: Some("你好，世界！".to_string()),
            preview: Some("你好，世界！".to_string()),
            content_hash: "abc123".to_string(),
            file_name: None,
            file_size: None,
            file_path: None,
            thumbnail_path: None,
            source_app: Some("终端".to_string()),
        };

        let id = insert_item(&conn, &item).unwrap();
        let retrieved = get_item(&conn, id).unwrap().unwrap();

        assert_eq!(retrieved.id, id);
        assert_eq!(retrieved.content, Some("你好，世界！".to_string()));
        assert_eq!(retrieved.content_type, ContentType::Text);
    }

    #[test]
    fn test_获取历史记录降序排列() {
        let conn = setup_test_db();

        for i in 0..3 {
            let item = ClipboardItemCreate {
                content_type: ContentType::Text,
                content: Some(format!("项目 {}", i)),
                preview: Some(format!("项目 {}", i)),
                content_hash: format!("hash{}", i),
                file_name: None,
                file_size: None,
                file_path: None,
                thumbnail_path: None,
                source_app: None,
            };
            insert_item(&conn, &item).unwrap();
        }

        let items = get_history(&conn, None, 100, 0).unwrap();
        assert_eq!(items.len(), 3);
        // 验证返回了所有项目
        let contents: Vec<Option<String>> = items.iter().map(|i| i.content.clone()).collect();
        assert!(contents.contains(&Some("项目 0".to_string())));
        assert!(contents.contains(&Some("项目 1".to_string())));
        assert!(contents.contains(&Some("项目 2".to_string())));
    }

    #[test]
    fn test_搜索项目() {
        let conn = setup_test_db();

        for (content, hash) in vec![
            ("你好世界", "hash1"),
            ("测试内容", "hash2"),
            ("你好再次", "hash3"),
        ] {
            let item = ClipboardItemCreate {
                content_type: ContentType::Text,
                content: Some(content.to_string()),
                preview: Some(content.to_string()),
                content_hash: hash.to_string(),
                file_name: None,
                file_size: None,
                file_path: None,
                thumbnail_path: None,
                source_app: None,
            };
            insert_item(&conn, &item).unwrap();
        }

        let results = search_items(&conn, "你好", 100).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_删除项目() {
        let conn = setup_test_db();
        let item = ClipboardItemCreate {
            content_type: ContentType::Text,
            content: Some("删除我".to_string()),
            preview: Some("删除我".to_string()),
            content_hash: "del123".to_string(),
            file_name: None,
            file_size: None,
            file_path: None,
            thumbnail_path: None,
            source_app: None,
        };

        let id = insert_item(&conn, &item).unwrap();
        delete_item(&conn, id).unwrap();
        assert!(get_item(&conn, id).unwrap().is_none());
    }

    #[test]
    fn test_置顶项目() {
        let conn = setup_test_db();
        let item = ClipboardItemCreate {
            content_type: ContentType::Text,
            content: Some("置顶我".to_string()),
            preview: Some("置顶我".to_string()),
            content_hash: "pin123".to_string(),
            file_name: None,
            file_size: None,
            file_path: None,
            thumbnail_path: None,
            source_app: None,
        };

        let id = insert_item(&conn, &item).unwrap();
        set_pinned(&conn, id, true).unwrap();

        let retrieved = get_item(&conn, id).unwrap().unwrap();
        assert!(retrieved.is_pinned);
    }

    #[test]
    fn test_按_hash_查找项目() {
        let conn = setup_test_db();
        let item = ClipboardItemCreate {
            content_type: ContentType::Text,
            content: Some("重复内容".to_string()),
            preview: Some("重复内容".to_string()),
            content_hash: "dup123".to_string(),
            file_name: None,
            file_size: None,
            file_path: None,
            thumbnail_path: None,
            source_app: None,
        };

        insert_item(&conn, &item).unwrap();
        assert!(get_item_by_hash(&conn, "dup123").unwrap().is_some());
        assert!(get_item_by_hash(&conn, "other_hash").unwrap().is_none());
    }

    #[test]
    fn test_插入后自动清理超出限制() {
        let conn = setup_test_db();

        for i in 0..5 {
            let item = ClipboardItemCreate {
                content_type: ContentType::Text,
                content: Some(format!("内容 {}", i)),
                preview: Some(format!("内容 {}", i)),
                content_hash: format!("hash_cleanup_{}", i),
                file_name: None,
                file_size: None,
                file_path: None,
                thumbnail_path: None,
                source_app: None,
            };
            insert_item(&conn, &item).unwrap();
        }

        let new_item = ClipboardItemCreate {
            content_type: ContentType::Text,
            content: Some("新内容".to_string()),
            preview: Some("新内容".to_string()),
            content_hash: "hash_cleanup_new".to_string(),
            file_name: None,
            file_size: None,
            file_path: None,
            thumbnail_path: None,
            source_app: None,
        };
        match upsert_item(&conn, &new_item, 3).unwrap() {
            UpsertResult::Inserted { .. } => {}
            UpsertResult::Updated => panic!("新内容不应命中更新"),
        }

        let count = get_item_count(&conn).unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_清理不删除置顶项() {
        let conn = setup_test_db();

        for i in 0..3 {
            let item = ClipboardItemCreate {
                content_type: ContentType::Text,
                content: Some(format!("内容 {}", i)),
                preview: Some(format!("内容 {}", i)),
                content_hash: format!("hash_pin_{}", i),
                file_name: None,
                file_size: None,
                file_path: None,
                thumbnail_path: None,
                source_app: None,
            };
            let id = insert_item(&conn, &item).unwrap();
            if i == 0 {
                set_pinned(&conn, id, true).unwrap();
            }
        }

        let new_item = ClipboardItemCreate {
            content_type: ContentType::Text,
            content: Some("新内容".to_string()),
            preview: Some("新内容".to_string()),
            content_hash: "hash_pin_new".to_string(),
            file_name: None,
            file_size: None,
            file_path: None,
            thumbnail_path: None,
            source_app: None,
        };
        match upsert_item(&conn, &new_item, 2).unwrap() {
            UpsertResult::Inserted { .. } => {}
            UpsertResult::Updated => panic!("新内容不应命中更新"),
        }

        let items = get_history(&conn, None, 100, 0).unwrap();
        assert!(items.iter().any(|i| i.is_pinned));
    }

    #[test]
    fn test_重复图片更新文件名并前移到顶部() {
        let conn = setup_test_db();

        let first = ClipboardItemCreate {
            content_type: ContentType::Image,
            content: None,
            preview: Some("100×100".to_string()),
            content_hash: "same-image".to_string(),
            file_name: Some("old.png".to_string()),
            file_size: Some(100),
            file_path: Some("/tmp/old.png".to_string()),
            thumbnail_path: Some("/tmp/old_200.png".to_string()),
            source_app: Some("Finder".to_string()),
        };
        let second = ClipboardItemCreate {
            content_type: ContentType::Text,
            content: Some("other".to_string()),
            preview: Some("other".to_string()),
            content_hash: "other".to_string(),
            file_name: None,
            file_size: None,
            file_path: None,
            thumbnail_path: None,
            source_app: Some("Terminal".to_string()),
        };

        insert_item(&conn, &first).unwrap();
        insert_item(&conn, &second).unwrap();

        conn.execute(
            "UPDATE clipboard_items SET updated_at = '2026-01-01 00:00:00' WHERE content_hash = 'same-image'",
            [],
        ).unwrap();
        conn.execute(
            "UPDATE clipboard_items SET updated_at = '2026-01-02 00:00:00' WHERE content_hash = 'other'",
            [],
        ).unwrap();

        let duplicate = ClipboardItemCreate {
            content_type: ContentType::Image,
            content: None,
            preview: Some("100×100".to_string()),
            content_hash: "same-image".to_string(),
            file_name: Some("new-name.png".to_string()),
            file_size: Some(100),
            file_path: Some("/tmp/new.png".to_string()),
            thumbnail_path: Some("/tmp/new_200.png".to_string()),
            source_app: Some("Finder".to_string()),
        };

        let result = upsert_item(&conn, &duplicate, 10).unwrap();
        assert!(matches!(result, UpsertResult::Updated));
        assert_eq!(get_item_count(&conn).unwrap(), 2);

        let updated = get_item_by_hash(&conn, "same-image").unwrap().unwrap();
        assert_eq!(updated.file_name.as_deref(), Some("new-name.png"));
        assert_eq!(updated.file_path.as_deref(), Some("/tmp/new.png"));

        let items = get_history(&conn, None, 10, 0).unwrap();
        assert_eq!(items.first().unwrap().content_hash, "same-image");
    }

    #[test]
    fn test_图片可先插入无文件路径的占位记录() {
        let conn = setup_test_db();

        let pending = ClipboardItemCreate {
            content_type: ContentType::Image,
            content: None,
            preview: Some("2196×1440".to_string()),
            content_hash: "pending-image".to_string(),
            file_name: Some("image0.png".to_string()),
            file_size: Some(2196 * 1440 * 4),
            file_path: None,
            thumbnail_path: None,
            source_app: Some("Finder".to_string()),
        };

        let result = upsert_item(&conn, &pending, 10).unwrap();
        assert!(matches!(result, UpsertResult::Inserted { .. }));

        let item = get_item_by_hash(&conn, "pending-image").unwrap().unwrap();
        assert_eq!(item.file_path, None);
        assert_eq!(item.thumbnail_path, None);
        assert_eq!(item.file_name.as_deref(), Some("image0.png"));
    }

    #[test]
    fn test_图片资源补写应更新同一条记录() {
        let conn = setup_test_db();

        let pending = ClipboardItemCreate {
            content_type: ContentType::Image,
            content: None,
            preview: Some("2196×1440".to_string()),
            content_hash: "attach-image".to_string(),
            file_name: Some("image0.png".to_string()),
            file_size: Some(64),
            file_path: None,
            thumbnail_path: None,
            source_app: Some("Finder".to_string()),
        };

        let result = upsert_item(&conn, &pending, 10).unwrap();
        assert!(matches!(result, UpsertResult::Inserted { .. }));
        let original_id = get_item_by_hash(&conn, "attach-image").unwrap().unwrap().id;

        let updated_id = attach_image_assets_by_hash(
            &conn,
            "attach-image",
            "/tmp/attach-image.png".to_string(),
            Some("/tmp/attach-image_preview.png".to_string()),
            Some(128),
        )
        .unwrap()
        .unwrap();

        assert_eq!(updated_id, original_id);
        assert_eq!(get_item_count(&conn).unwrap(), 1);

        let item = get_item_by_hash(&conn, "attach-image").unwrap().unwrap();
        assert_eq!(item.id, original_id);
        assert_eq!(item.file_path.as_deref(), Some("/tmp/attach-image.png"));
        assert_eq!(
            item.thumbnail_path.as_deref(),
            Some("/tmp/attach-image_preview.png")
        );
        assert_eq!(item.file_size, Some(128));
    }

    #[test]
    fn test_图片资源补写不应改变历史顺序() {
        let conn = setup_test_db();

        let pending = ClipboardItemCreate {
            content_type: ContentType::Image,
            content: None,
            preview: Some("2196×1440".to_string()),
            content_hash: "top-image".to_string(),
            file_name: Some("image0.png".to_string()),
            file_size: Some(64),
            file_path: None,
            thumbnail_path: None,
            source_app: Some("Finder".to_string()),
        };
        let other = ClipboardItemCreate {
            content_type: ContentType::Text,
            content: Some("other".to_string()),
            preview: Some("other".to_string()),
            content_hash: "other-image-top".to_string(),
            file_name: None,
            file_size: None,
            file_path: None,
            thumbnail_path: None,
            source_app: Some("Terminal".to_string()),
        };

        insert_item(&conn, &pending).unwrap();
        insert_item(&conn, &other).unwrap();

        conn.execute(
            "UPDATE clipboard_items SET updated_at = '2026-01-01 00:00:00' WHERE content_hash = 'top-image'",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE clipboard_items SET updated_at = '2026-01-02 00:00:00' WHERE content_hash = 'other-image-top'",
            [],
        )
        .unwrap();

        attach_image_assets_by_hash(
            &conn,
            "top-image",
            "/tmp/top-image.png".to_string(),
            Some("/tmp/top-image_preview.png".to_string()),
            Some(128),
        )
        .unwrap();

        let items = get_history(&conn, None, 10, 0).unwrap();
        assert_eq!(items.first().unwrap().content_hash, "other-image-top");

        let item = get_item_by_hash(&conn, "top-image").unwrap().unwrap();
        assert_eq!(item.updated_at, "2026-01-01 00:00:00");
    }

    #[test]
    fn test_清空历史后重置自增序列() {
        let conn = setup_test_db();
        let item = ClipboardItemCreate {
            content_type: ContentType::Text,
            content: Some("reset".to_string()),
            preview: Some("reset".to_string()),
            content_hash: "reset-1".to_string(),
            file_name: None,
            file_size: None,
            file_path: None,
            thumbnail_path: None,
            source_app: None,
        };

        let first_id = insert_item(&conn, &item).unwrap();
        assert_eq!(first_id, 1);
        clear_all(&conn).unwrap();
        vacuum_database(&conn).unwrap();

        let second = ClipboardItemCreate {
            content_type: ContentType::Text,
            content: Some("reset2".to_string()),
            preview: Some("reset2".to_string()),
            content_hash: "reset-2".to_string(),
            file_name: None,
            file_size: None,
            file_path: None,
            thumbnail_path: None,
            source_app: None,
        };
        let second_id = insert_item(&conn, &second).unwrap();
        assert_eq!(second_id, 1);
    }
}
