-- 创建 FTS5 全文索引
CREATE VIRTUAL TABLE IF NOT EXISTS clipboard_items_fts USING fts5(
    preview,
    file_name,
    content='clipboard_items',
    content_rowid='id'
);

-- 填充现有数据
INSERT INTO clipboard_items_fts(rowid, preview, file_name)
    SELECT id, preview, file_name FROM clipboard_items;

-- 触发器：插入时同步
CREATE TRIGGER IF NOT EXISTS clipboard_items_ai AFTER INSERT ON clipboard_items BEGIN
    INSERT INTO clipboard_items_fts(rowid, preview, file_name)
        VALUES (new.id, new.preview, new.file_name);
END;

-- 触发器：删除时同步
CREATE TRIGGER IF NOT EXISTS clipboard_items_ad AFTER DELETE ON clipboard_items BEGIN
    INSERT INTO clipboard_items_fts(clipboard_items_fts, rowid, preview, file_name)
        VALUES ('delete', old.id, old.preview, old.file_name);
END;

-- 触发器：更新时同步
CREATE TRIGGER IF NOT EXISTS clipboard_items_au AFTER UPDATE ON clipboard_items BEGIN
    INSERT INTO clipboard_items_fts(clipboard_items_fts, rowid, preview, file_name)
        VALUES ('delete', old.id, old.preview, old.file_name);
    INSERT INTO clipboard_items_fts(rowid, preview, file_name)
        VALUES (new.id, new.preview, new.file_name);
END;
