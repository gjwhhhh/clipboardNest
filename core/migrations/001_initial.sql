-- 剪切板管理器数据库迁移

CREATE TABLE IF NOT EXISTS clipboard_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type TEXT NOT NULL CHECK(content_type IN ('text', 'richtext', 'image', 'file')),
    content TEXT,
    preview TEXT,
    content_hash TEXT NOT NULL,
    file_name TEXT,
    file_size INTEGER,
    file_path TEXT,
    thumbnail_path TEXT,
    source_app TEXT,
    is_pinned BOOLEAN DEFAULT FALSE,
    is_favorite BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_clipboard_created_at ON clipboard_items(created_at);
CREATE INDEX IF NOT EXISTS idx_clipboard_content_type ON clipboard_items(content_type);
CREATE INDEX IF NOT EXISTS idx_clipboard_content_hash ON clipboard_items(content_hash);

-- 默认设置
INSERT OR IGNORE INTO settings (key, value) VALUES
    ('retention_days', '30'),
    ('max_items', '5000'),
    ('poll_interval_ms', '500'),
    ('launch_at_login', 'true'),
    ('theme', 'system');
