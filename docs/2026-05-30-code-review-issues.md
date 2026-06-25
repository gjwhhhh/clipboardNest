# 代码审查问题记录

> 审查时间: 2026-05-30
> 审查范围: 全项目代码（Rust 后端 + React 前端）

## 🔴 严重问题

### BUG-01: copy_to_clipboard 触发监控器重复检测
- **位置**: `src-tauri/src/commands/clipboard.rs:46`, `src-tauri/src/clipboard/monitor.rs`
- **描述**: 从历史中复制内容写入系统剪切板后，监控器 500ms 后检测到变化，虽然 `is_duplicate` 能阻止重复插入，但会导致不必要的 DB 查询和 `updated_at` 时间戳更新
- **影响**: 性能浪费，时间戳被错误更新

### BUG-02: 监控器锁争用
- **位置**: `src-tauri/src/clipboard/monitor.rs:72-76`
- **描述**: `clipboard` 锁在 `check_clipboard` 整个执行期间被持有，包括可能阻塞的 `clipboard.get_text()` 调用。前端 `copy_to_clipboard` 也需要访问剪切板
- **影响**: 前端操作可能卡顿

### BUG-03: 窗口定位硬编码为 1920x1080
- **位置**: `src-tauri/src/window/mod.rs:8-9`
- **描述**: 屏幕尺寸写死，非此分辨率的显示器上窗口位置偏移。`position_near_mouse` 未被调用
- **影响**: 多显示器/非标准分辨率用户窗口位置错误

### BUG-04: 数据库无上限清理机制
- **位置**: `src-tauri/src/storage/repository.rs:191`
- **描述**: `delete_oldest` 存在但从未被调用，数据库会无限增长
- **影响**: 长期使用后 SQLite 文件过大，性能下降

### BUG-05: 全局快捷键无法动态修改
- **位置**: `src-tauri/src/hotkey/mod.rs`
- **描述**: 快捷键启动时注册一次，设置面板的"更改"按钮是空壳
- **影响**: 用户无法自定义快捷键

## 🟡 中等问题

### BUG-06: 搜索无全文索引
- **位置**: `src-tauri/src/storage/repository.rs:119`
- **描述**: `WHERE preview LIKE ?1 OR file_name LIKE ?1` 全表扫描
- **影响**: 数据量大时搜索变慢

### BUG-07: useKeyboard 拦截搜索框快捷键
- **位置**: `src/hooks/useKeyboard.ts:31-34`
- **描述**: `Cmd+Backspace` 在搜索框聚焦时会被拦截，导致意外触发删除
- **影响**: 用户在搜索框清空内容时触发删除操作

### BUG-08: 复制操作无视觉反馈
- **位置**: `src/components/ClipboardList/ClipboardItem.tsx`
- **描述**: 双击或点击复制按钮后无任何提示
- **影响**: 用户不确定是否复制成功

### BUG-09: 错误状态不会自动消失且不可见
- **位置**: `src/stores/clipboardStore.ts`
- **描述**: 错误写入 `error` 字段但 UI 中不展示，无超时清除
- **影响**: 操作失败时用户完全无感知

### BUG-10: useEffect 缺少依赖
- **位置**: `src/components/SettingsPanel/index.tsx:16`
- **描述**: `fetchSettings` 未加入依赖数组
- **影响**: ESLint 警告

### BUG-11: 系统托盘图标缺失
- **位置**: `src-tauri/src/tray/mod.rs`
- **描述**: `TrayIconBuilder` 没调 `.icon()`，macOS 上托盘显示空白
- **影响**: 托盘无图标，用户难以识别

### BUG-12: 点击记录后预览面板无法退出（严重体验问题）
- **位置**: `src/components/ClipboardList/index.tsx`, `src/components/PreviewPanel/index.tsx`, `src/App.tsx`
- **描述**: 当前交互流程：点击列表项 → 底部弹出预览面板 → 无法退回。预览面板占据了列表空间，用户被困在预览状态。这与 Alfred 等成熟剪切板工具的交互模式差距很大
- **影响**: 核心交互体验严重受损，用户无法高效浏览和选择历史记录
- **期望行为（Alfred 风格）**:
  1. 左右分栏布局：左侧列表（截断标题），右侧实时预览
  2. 鼠标 hover 左侧列表项时，右侧自动显示预览内容
  3. 点击列表项 → 复制到剪切板 + 自动隐藏应用窗口
  4. 移除底部预览面板和双击复制的交互
