# ClipBoard Manager 文件复制支持实现计划

## Context

当前项目只支持文本和图像的剪贴板监控，不支持非图像文件（如 PDF、文档、zip 等）。用户复制文件时，剪贴板监控器会静默忽略这些内容。

数据库 schema、前端组件已为文件类型做好准备，但后端监控器和写回逻辑缺失。

## 实现方案

### 1. 新增文件存储模块

**文件**: `src-tauri/src/clipboard/file_store.rs`（新建）

参考 `image_store.rs` 的模式，实现文件复制和清理：

```rust
// 复制文件到应用数据目录，保留原始扩展名
pub fn copy_file_to_store(files_dir: &Path, hash: &str, source_path: &Path) -> Result<String, Error>

// 删除存储的文件
pub fn delete_file(file_path: &str) -> Result<(), Error>

// 格式化文件大小（B/KB/MB/GB）
pub fn format_file_size(bytes: i64) -> String
```

在 `src-tauri/src/clipboard/mod.rs` 中注册：`pub mod file_store;`

### 2. 修改剪贴板监控器

**文件**: `src-tauri/src/clipboard/monitor.rs`

#### 2.1 新增辅助函数

```rust
// 从剪贴板读取非图像文件路径（优先级：native_file_url > file_list > text）
fn non_image_file_from_clipboard(
    native_file_url_path: Option<&Path>,
    file_list: Option<&[PathBuf]>,
    text: Option<&str>,
) -> Option<PathBuf>

// 从文本解析文件路径（支持 file:// URL 和绝对路径）
fn file_path_from_text(text: Option<&str>) -> Option<PathBuf>
```

#### 2.2 修改 `check_clipboard` 函数

在现有的 `if/else if` 链中添加文件处理分支：

```
if let Some(text) = text { ... }
else if let Some(image_path) = file_image_path { ... }
else if let Some(file_path) = non_image_file_path { /* 新增：文件处理 */ }
else if let Some(img) = image_data { ... }
```

文件处理分支逻辑：
1. 使用 `hash_file_identity()` 生成去重哈希（复用现有函数）
2. 创建 `ClipboardItemCreate` 记录（`ContentType::File`，`file_path=None`）
3. 调用 `repository::upsert_item()` 插入占位记录
4. 触发 `app.emit("clipboard-updated")` 通知前端
5. 调用 `spawn_file_copy()` 后台复制文件

#### 2.3 新增 `spawn_file_copy` 函数

参考 `spawn_image_asset_generation()` 的模式：

```rust
fn spawn_file_copy(
    conn: Arc<Mutex<rusqlite::Connection>>,
    state: Arc<MonitorState>,
    files_dir: PathBuf,
    app_handle: tauri::AppHandle,
    hash: String,
    source_path: PathBuf,
)
```

后台线程：复制文件 → 更新数据库 `file_path` → 触发前端刷新

#### 2.4 修改函数签名

- `start_monitoring` 增加 `files_dir: PathBuf` 参数
- `check_clipboard` 增加 `files_dir: PathBuf` 参数

### 3. 修改仓储层

**文件**: `src-tauri/src/storage/repository.rs`

#### 3.1 新增函数

```rust
// 补写文件资源路径（类似 attach_image_assets_by_hash，但无 thumbnail_path）
pub fn attach_file_assets_by_hash(
    conn: &Connection,
    content_hash: &str,
    file_path: &str,
    file_size: Option<i64>,
) -> Result<Option<i64>, rusqlite::Error>

// 获取所有文件类型的存储路径（用于清理）
pub fn get_all_file_items(conn: &Connection) -> Result<Vec<String>, rusqlite::Error>
```

#### 3.2 修改现有函数

- `delete_oldest`: 返回删除的文件路径用于清理
- `clear_all`: 同时清理文件类型记录

### 4. 修改剪贴板写回命令

**文件**: `src-tauri/src/commands/clipboard.rs`

#### 4.1 修改 `copy_to_clipboard`

将第 140-142 行的错误分支替换为实际实现：

```rust
ContentType::File => {
    if let Some(ref file_path) = item.file_path {
        crate::clipboard::native_macos::write_file_to_clipboard(file_path)?;
    } else {
        return Err("文件尚未保存，无法复制".to_string());
    }
}
```

#### 4.2 修改 `delete_clipboard_item`

添加文件类型清理逻辑：

```rust
if item.content_type == ContentType::File {
    if let Some(ref fp) = item.file_path {
        if let Err(e) = crate::clipboard::file_store::delete_file(fp) {
            log::warn!("清理文件失败: {}", e);
        }
    }
}
```

#### 4.3 修改 `clear_all_history`

同时清理文件类型记录的存储文件。

### 5. 新增原生 macOS 函数

**文件**: `src-tauri/src/clipboard/native_macos.rs`

```rust
// 将文件写回系统剪贴板（使用 NSPasteboard 的文件 URL 方式）
#[cfg(target_os = "macos")]
pub fn write_file_to_clipboard(file_path: &str) -> Result<bool, Box<dyn std::error::Error>>

// 非 macOS 平台的桩函数
#[cfg(not(target_os = "macos"))]
pub fn write_file_to_clipboard(_file_path: &str) -> Result<bool, Box<dyn std::error::Error>>
```

实现要点：
- 使用 `NSPasteboard` 的 `writeObjects:` 方法
- 创建 `NSURL` 对象表示文件
- 写入 `NSURL` 数组到剪贴板

### 6. 修改应用入口

**文件**: `src-tauri/src/lib.rs`

第 46 行后添加：

```rust
let files_dir = app.path().app_data_dir().expect("无法获取应用数据目录").join("files");
```

第 51 行的 `start_monitoring` 调用增加 `files_dir` 参数。

### 7. 前端适配（可选增强）

当前前端已基本就绪，可选择性增强：

**文件**: `src/components/PreviewPanel/index.tsx`

- 文件项 `filePath` 为 null 时显示"复制中..."提示
- 增强文件大小格式化（支持 MB/GB）

## 数据流

```
用户在 Finder 复制文件
  ↓
check_clipboard 检测到非图像文件路径
  ↓
hash_file_identity 生成去重哈希
  ↓
插入 ContentType::File 占位记录（file_path=None）
  ↓
前端立即显示文件图标、文件名、文件大小
  ↓
后台线程 spawn_file_copy 复制文件到 <app_data>/files/
  ↓
更新数据库 file_path 字段
  ↓
用户点击文件项 → copy_to_clipboard
  ↓
write_file_to_clipboard 写入系统剪贴板
```

## 边界情况处理

1. **大文件**: 后台异步复制，UI 立即响应
2. **源文件被删除/移动**: 文件已复制到应用目录，不受影响
3. **多文件复制**: v1 只处理第一个非图像文件（后续可扩展）
4. **文件尚未复制完成**: `copy_to_clipboard` 检查 `file_path` 是否为 null

## 关键文件

- `src-tauri/src/clipboard/file_store.rs`（新建）
- `src-tauri/src/clipboard/monitor.rs`（修改）
- `src-tauri/src/clipboard/native_macos.rs`（修改）
- `src-tauri/src/commands/clipboard.rs`（修改）
- `src-tauri/src/storage/repository.rs`（修改）
- `src-tauri/src/lib.rs`（修改）
- `src-tauri/src/clipboard/mod.rs`（修改）

## 验证方案

### 单元测试

1. `file_store.rs` 测试：复制文件、删除文件、格式化大小
2. `monitor.rs` 测试：非图像文件路径检测、文件 URL 解析
3. `repository.rs` 测试：文件记录插入和资源补写

### 手动测试

1. 从 Finder 复制 PDF → 验证出现在剪贴板历史，显示文件图标、文件名、文件大小
2. 点击文件项 → 验证粘贴到 Finder 窗口或应用
3. 复制相同文件两次 → 验证无重复记录
4. 删除剪贴板条目 → 验证存储文件被清理
5. 复制文本后再复制文件 → 验证两种类型正常共存

### 运行测试

```bash
cd src-tauri && cargo test
cd .. && npm run tauri dev
```
