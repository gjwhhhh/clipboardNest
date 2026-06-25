# 桌面端待完成功能清单

> 基于当前桌面端实现整理，用于跟踪开源后仍待完善的功能。

## 已完成功能（16/19 任务）

| 任务 | 状态 | 备注 |
|------|------|------|
| 项目初始化 | ✅ | 采用 workspace 架构，core 独立 |
| Tailwind CSS | ✅ | 含深色模式 CSS 变量 |
| SQLite 数据库 | ✅ | 3 个迁移 + FTS5 |
| 仓库 CRUD | ✅ | 扩展了 upsert、FTS5、清理等 |
| 内容解析器 | ✅ | 含 RTF 降级 |
| 剪切板监控 | ✅ | 原生 macOS、异步资源生成 |
| Tauri 命令 | ✅ | 含 hide_window、update_hotkey 等 |
| TypeScript 类型 | ✅ | |
| Zustand 状态管理 | ✅ | |
| 自定义 Hooks | ✅ | |
| 列表组件 | ✅ | |
| 搜索栏 | ✅ | |
| 预览面板 | ✅ | |
| 设置面板 | ✅ | |
| 主应用组件 | ✅ | Alfred 风格布局 |
| 全局快捷键 | ✅ | 含动态更新 |

## 未完成功能（3/19 任务）

### 1. 国际化 (i18n)

**计划要求：**
- 安装 `i18next` + `react-i18next`
- 创建 `src/locales/en.json`、`src/locales/zh.json`
- 所有 UI 文本使用 `useTranslation()` 钩子

**当前状态：**
- 未安装 i18next 包
- 不存在 locales 目录
- 所有 UI 文本硬编码中文

**工作量：** 中等（2-3 天）

---

### 2. 通用组件 (Button.tsx, Modal.tsx)

**计划要求：**
- 创建 `src/components/common/Button.tsx`
- 创建 `src/components/common/Modal.tsx`

**当前状态：**
- 不存在 `src/components/common/` 目录
- 各组件内联实现按钮和弹窗

**工作量：** 小（1 天）

---

### 3. 性能优化

**计划要求：**
- 剪切板监控防抖优化
- SQLite 查询优化验证
- 5000+ 项目性能测试

**当前状态：**
- 未进行系统性性能测试
- 虚拟滚动已实现但未优化

**工作量：** 中等（2-3 天）

## 部分完成功能（设置项未生效）

### 1. 开机自启 (`launchAtLogin`)

**问题：** 设置值仅保存在数据库，未调用系统 API

**修复方案：**
- 集成 `tauri-plugin-autostart` 插件
- 在 `update_setting` 命令中调用 `autostart.enable()`/`autostart.disable()`
- 启动时根据设置启用/禁用

**工作量：** 小（0.5 天）

---

### 2. 主题切换 (`theme`)

**问题：** 仅跟随系统偏好，手动选择不生效

**修复方案：**
- 创建 `useTheme` Hook，读取设置中的 theme 值
- 通过 `document.documentElement.classList` 切换 `dark` 类
- 更新 `tailwind.config.js` 为 `darkMode: "class"`

**工作量：** 小（0.5 天）

---

### 3. 轮询间隔 (`pollIntervalMs`)

**问题：** 监控器启动时硬编码 500ms，不读取设置

**修复方案：**
- 启动时从数据库读取 `poll_interval_ms`
- 传入 `start_monitoring` 函数

**工作量：** 极小（0.5 小时）

---

### 4. 快捷键编辑 UI

**问题：** 需手动输入字符串，不支持按键捕获

**修复方案：**
- 添加键盘事件监听，自动捕获按键组合
- 格式化为 `Cmd+Shift+V` 格式

**工作量：** 小（1 天）

---

### 5. 窗口定位

**问题：** 窗口居中显示而非跟随鼠标

**修复方案：**
- 恢复 `position_near_mouse` 实现
- 使用 macOS `NSEvent.mouseLocation` 获取鼠标位置

**工作量：** 小（0.5 天）

## 优先级建议

| 优先级 | 功能 | 原因 |
|--------|------|------|
| P0 | 轮询间隔修复 | 一行代码改动 |
| P0 | 主题切换修复 | 用户可见功能缺失 |
| P1 | 开机自启 | 用户期望功能 |
| P1 | 窗口定位 | 体验问题 |
| P2 | 快捷键编辑 UI | 可用手动输入替代 |
| P2 | 国际化 | 非核心功能 |
| P3 | 通用组件 | 可复用性改进 |
| P3 | 性能优化 | 当前可接受 |
