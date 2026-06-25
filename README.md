# ClipBoard Manager - macOS 剪切板管理工具

## 项目简介

一款 macOS 桌面剪切板管理工具，能够记录文字、图片、小文件的剪切历史，通过快捷键快速呼出选择历史记录。

## 技术选型

| 类别 | 选择 | 说明 |
|------|------|------|
| 框架 | Tauri 2.0 | Rust 后端 + Web 前端，性能好体积小 |
| 前端 | React + TypeScript | 生态成熟，开发效率高 |
| 样式 | Tailwind CSS | 原子化 CSS，快速开发 |
| 状态管理 | Zustand | 轻量级状态管理 |
| 数据库 | SQLite | 本地存储，轻量高效 |
| 分发 | Tauri 桌面安装包 | 按平台生成对应安装产物 |

## 主要功能

- ✅ 文件类型剪切板支持（复制/粘贴文件）
- ✅ 来源应用检测（显示内容来源）
- ✅ 富文本降级处理（RTF 自动转为纯文本）
- ✅ 动态快捷键配置（支持字母、数字、功能键）

---

## 打包说明

### 本机打包命令

- `npm run build:macos`：在 macOS 上生成 `.app` 和 `.dmg`
- `npm run build:windows`：统一入口；在 Windows 主机上生成 `.msi` 和 `NSIS .exe`，在 macOS 上自动切到 `NSIS .exe` 交叉打包
- `npm run build:windows:host`：只给 Windows 主机用，生成 `.msi` 和 `NSIS .exe`
- `npm run build:windows:cross`：在 macOS 上交叉生成 Windows `NSIS .exe`
- `npm run build:windows:nsis-cross`：`build:windows:cross` 的兼容别名
- `npm run build:linux`：在 Linux 上生成 `.deb`、`.AppImage`、`.rpm`

### 为什么在 macOS 上看不到 Windows 安装包

- `npm run build:release` 等价于 `tauri build`，默认只构建当前平台支持的 bundle
- macOS 本机构建不会自动产出 Windows `.msi`
- 现在直接执行 `npm run build:windows`，会在 macOS 上自动走交叉打包，不再误用主机构建参数
- 如果要稳定产出完整跨平台安装包，使用 `.github/workflows/build-desktop-bundles.yml` 在三种 runner 上分别构建

### 产物目录

- macOS：`src-tauri/target/release/bundle/macos/`、`src-tauri/target/release/bundle/dmg/`
- Windows：`src-tauri/target/release/bundle/msi/`、`src-tauri/target/release/bundle/nsis/`
- Linux：`src-tauri/target/release/bundle/deb/`、`src-tauri/target/release/bundle/appimage/`、`src-tauri/target/release/bundle/rpm/`

---

## 项目结构

```
clipboard_proj/
├── src-tauri/                    # Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs               # 入口
│   │   ├── lib.rs                # 应用入口
│   │   ├── clipboard/            # 剪切板模块
│   │   │   ├── mod.rs            # 剪切板模块入口
│   │   │   ├── monitor.rs        # 剪切板监控（轮询 changeCount）
│   │   │   ├── parser.rs         # 内容类型检测与解析
│   │   │   ├── hasher.rs         # 内容哈希（SHA-256）
│   │   │   ├── image_store.rs    # 图片存储（原图 + 预览图）
│   │   │   ├── file_store.rs     # 文件存储
│   │   │   └── native_macos.rs   # macOS 原生剪切板操作
│   │   ├── storage/              # 数据存储模块
│   │   │   ├── mod.rs
│   │   │   ├── database.rs       # 数据库初始化与迁移
│   │   │   ├── models.rs         # 数据模型
│   │   │   └── repository.rs     # CRUD 操作
│   │   ├── hotkey/               # 全局快捷键管理
│   │   │   └── mod.rs
│   │   ├── tray/                 # 系统托盘
│   │   │   └── mod.rs
│   │   ├── window/               # 窗口管理
│   │   │   └── mod.rs
│   │   └── commands/             # Tauri 命令接口
│   │       ├── mod.rs
│   │       ├── clipboard.rs      # 剪切板相关命令
│   │       └── settings.rs       # 设置相关命令
│   ├── migrations/
│   └── resources/
├── src/                          # React 前端
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/
│   │   ├── ClipboardList/        # 历史列表
│   │   ├── SearchBar/            # 搜索栏
│   │   ├── PreviewPanel/         # 预览面板
│   │   └── SettingsPanel/        # 设置面板
│   ├── hooks/
│   ├── stores/
│   ├── types/
│   └── utils/
├── package.json
├── tsconfig.json
├── vite.config.ts
└── index.html
```

---

## 核心功能

### 1. 剪切板监控

**实现方式：** 定时轮询 NSPasteboard.changeCount

- 轮询间隔：500ms（窗口隐藏时 1s，可用户自定义）
- 检测内容类型：纯文本、富文本(RTF)、图片(PNG/TIFF/JPEG/GIF/WebP)、文件路径
- 图片存储为本地文件，记录路径
- 去重处理：相同内容不重复记录

**性能说明：**
- change_count 是轻量操作（读取整数属性，微秒级）
- 只有内容变化时才真正读取剪切板
- Maccy、Paste 等主流应用均采用此方案

### 2. 数据存储

**SQLite 表结构：**

```sql
CREATE TABLE clipboard_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type TEXT NOT NULL,          -- 'text' | 'richtext' | 'image' | 'file'
    content TEXT,                        -- 文本内容 / 文件路径
    preview TEXT,                        -- 预览文本（前100字符）
    content_hash TEXT,                   -- 内容哈希（用于去重）
    file_name TEXT,                      -- 文件名
    file_size INTEGER,                   -- 文件大小（字节）
    file_path TEXT,                      -- 存储路径
    thumbnail_path TEXT,                 -- 预览图路径（图片类型）
    source_app TEXT,                     -- 来源应用
    is_pinned BOOLEAN DEFAULT FALSE,
    is_favorite BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX idx_clipboard_created_at ON clipboard_items(created_at);
CREATE INDEX idx_clipboard_content_type ON clipboard_items(content_type);
CREATE INDEX idx_clipboard_content_hash ON clipboard_items(content_hash);
```

**数据库配置：**
- 启用 WAL（Write-Ahead Logging）模式，提升并发性能和崩溃恢复能力
- 启用外键约束
- 设置 busy_timeout 为 5000ms

### 3. 全局快捷键

- 默认：`Cmd + Shift + V`（可自定义）
- 触发：显示/隐藏主窗口
- 窗口定位：跟随鼠标所在的显示器，在鼠标位置附近显示

### 4. 系统托盘

- 显示应用图标
- 右键菜单：显示历史、设置、退出
- 左键点击：显示/隐藏窗口

### 5. 前端 UI

**主窗口（400x500px）：**
- 搜索栏：实时搜索历史（后端 SQLite 查询）
- 分类筛选：全部/文本/图片/文件
- 历史列表：虚拟滚动，图片项显示图标与元信息
- 预览面板：选中项详细预览
- 操作按钮：复制、删除、置顶、收藏

**UI 特性：**
- 毛玻璃背景效果
- 深色/浅色模式跟随系统
- 键盘导航（上下选择，回车确认）
- 流畅动画过渡

---

## 数据生命周期管理

### 保留策略

- **默认保留天数：** 30 天（用户可在设置中调整）
- **最大记录数量：** 5000 条
- **清理机制：** 满足任一条件时自动清理：
  1. 记录超过 30 天 → 删除最旧的记录
  2. 记录超过 5000 条 → 删除最旧的记录
  3. 清理时同步删除关联的图片文件和预览图

### 存储空间管理

- 应用启动时执行一次清理检查
- 每次新增记录后检查是否触发清理
- 删除记录时同步删除对应的图片文件和预览图文件

---

## 图片存储方案

### 存储路径

```
~/Library/Application Support/clipboard-manager/
└── images/
    ├── {hash}.png          # 原图
    └── {hash}_preview.png  # 预览图
```

### 存储规则

- **大小限制：** 单张图片最大 20MB，超过则跳过不记录
- **支持格式：** PNG、TIFF、JPEG、GIF、WebP
- **入库方式：** 图片先以占位记录写入数据库，再后台异步生成原图与预览图
- **预览图：** 生成单张 PNG 预览图用于右侧预览，列表不再读取图片缩略图
- **命名方式：** 使用内容哈希作为文件名，避免重复写入
- **清理联动：** 删除历史记录时，同步删除对应的原图和预览图

---

## 去重与防抖

### 内容去重

- **文本类型：** 计算内容 SHA-256 哈希，相同哈希视为重复
- **图片类型：** 计算图片文件 SHA-256 哈希，相同哈希视为重复
- **文件路径：** 路径相同视为重复

### 防抖机制

- 相同内容在 1 秒内连续复制，只记录第一次
- 实现方式：记录最近一次的内容哈希和时间戳，1 秒内相同哈希跳过

---

## macOS 系统权限

### 所需权限

| 权限 | 用途 | 是否必须 |
|------|------|----------|
| 辅助功能（Accessibility） | 读取其他应用的剪切板内容 | 是 |
| 通知权限 | 显示操作反馈通知 | 否 |

### 权限引导

- 首次启动时检测辅助功能权限
- 未授权时弹窗引导用户到"系统设置 → 隐私与安全 → 辅助功能"中添加应用
- 提供"打开系统设置"按钮，一键跳转
- 权限被拒绝时显示提示页面，说明权限用途

### 沙盒配置

- 应用不启用 App Sandbox（沙盒模式下无法监控其他应用的剪切板）
- 使用 `com.apple.security.automation.apple-events` 权限

---

## 错误处理

### 数据库异常

| 场景 | 处理方式 |
|------|----------|
| 数据库文件损坏 | 尝试使用 `PRAGMA integrity_check` 检测，执行 `REINDEX` 修复；修复失败则备份损坏文件，重建新数据库 |
| 数据库锁定（busy） | 使用 WAL 模式 + busy_timeout，超时后提示用户重启应用 |
| 磁盘空间不足 | 检测写入失败，提示用户清理空间，暂停剪切板记录 |

### 图片文件异常

| 场景 | 处理方式 |
|------|----------|
| 图片文件丢失/损坏 | 列表保留文本记录，预览区标记为"文件不可用" |
| 预览图生成失败 | 回退使用原图进行预览 |
| 图片超过大小限制 | 静默跳过，不记录该条剪切板内容 |

### 剪切板内容异常

| 场景 | 处理方式 |
|------|----------|
| 内容类型无法识别 | 静默跳过，不记录 |
| 剪切板访问被拒绝 | 记录错误日志，等待下次轮询重试 |
| 富文本解析失败 | 降级为纯文本记录 |

### 应用崩溃恢复

- 使用 SQLite WAL 模式，崩溃后自动恢复未提交的事务
- 应用重启时检查数据库完整性
- 清理孤立的图片文件（数据库中无对应记录的图片）

---

## 快捷键冲突处理

### 注册失败处理

1. 应用启动时尝试注册全局快捷键
2. 注册失败（被其他应用占用）时：
   - 弹窗提示用户："快捷键 `Cmd+Shift+V` 已被其他应用占用"
   - 提供"打开设置修改快捷键"按钮
   - 应用仍可正常使用，只是快捷键不可用，可通过系统托盘打开

### 自定义快捷键

- 用户可在设置中自定义快捷键
- 修改时实时尝试注册，失败则提示冲突
- 支持的修饰键：Cmd、Shift、Ctrl、Option
- 至少需要一个修饰键 + 一个普通键

---

## 多显示器行为

- 窗口显示在当前鼠标所在的显示器上
- 窗口位置靠近鼠标，但不遮挡鼠标光标
- 如果窗口超出屏幕边缘，自动调整位置确保完全可见

---

## 设置面板

### 设置项清单

| 设置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| 全局快捷键 | 快捷键选择 | `Cmd+Shift+V` | 点击后按下新的快捷键组合 |
| 历史保留天数 | 滑块/输入框 | 30 天 | 范围：7-365 天 |
| 最大记录数量 | 下拉选择 | 5000 条 | 可选：1000/3000/5000/10000 |
| 开机自启 | 开关 | 开启 | 登录时自动启动应用 |
| 深色模式 | 选择 | 跟随系统 | 可选：跟随系统/浅色/深色 |
| 清除所有历史 | 按钮 | - | 点击后二次确认，清除所有记录和图片 |

---

## 性能指标

### 响应要求

| 指标 | 目标 |
|------|------|
| 列表滚动帧率 | ≥ 60fps（5000 条记录） |
| 搜索响应时间 | < 100ms |
| 应用启动时间 | < 1s |
| 内存占用 | < 100MB |
| 空闲 CPU 占用 | < 1% |
| 窗口显示延迟 | < 200ms（快捷键触发到窗口可见） |

### 搜索策略

- 使用后端 SQLite 查询，支持 `LIKE` 模糊匹配
- 搜索字段：preview（预览文本）、file_name（文件名）
- 搜索结果限制：最多返回 200 条
- 前端使用防抖（300ms），避免频繁查询

### 虚拟滚动

- 使用虚拟滚动渲染列表，仅渲染可视区域内的条目
- 可视区域外预留上下各 5 个条目的缓冲
- 每个列表项高度固定，支持快速定位

---

## UI 语言

- 支持中文和英文双语
- 默认跟随系统语言
- 用户可在设置中手动切换
- 语言资源文件存放在 `src/locales/` 目录

---

## 开发阶段

### Phase 1: 项目初始化 (1-2天)

- [x] 初始化 Tauri + React + TypeScript 项目
- [x] 配置 Rust 和 Node 依赖
- [x] 搭建目录结构
- [x] 配置 Tailwind CSS、ESLint

### Phase 2: 核心后端 (3-4天)

- [x] SQLite 数据库初始化和迁移
- [x] 数据模型和 Repository 实现
- [x] 剪切板监控模块
- [x] Tauri 命令接口

### Phase 3: 系统集成 (2-3天)

- [x] 全局快捷键注册
- [x] 系统托盘功能
- [x] 窗口管理（无边框、失焦隐藏）

### Phase 4: 前端 UI (3-4天)

- [x] 主窗口布局
- [x] 剪切板列表（虚拟滚动）
- [x] 搜索和过滤功能
- [x] 预览面板
- [x] 设置面板

### Phase 5: 打磨优化 (2-3天)

- [x] 性能优化（虚拟滚动、懒加载）
- [x] 动画效果和音效反馈
- [x] 错误处理和空状态
- [x] 批量操作和导出功能

### Phase 6: 打包分发 (1-2天)

- [x] 应用图标设计
- [x] 构建 macOS / Windows / Linux 安装包
- [x] 功能和性能测试
- [x] 使用文档编写

**总计：约 2-3 周**

---

## 依赖清单

### Rust (Cargo.toml)

```toml
[package]
name = "clipboard-manager"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-global-shortcut = "2"
tauri-plugin-shell = "2"
arboard = "3"
rusqlite = { version = "0.31", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
image = "0.25"
uuid = { version = "1", features = ["v4"] }
tokio = { version = "1", features = ["full"] }
log = "0.4"
env_logger = "0.11"
sha2 = "0.10"
```

### Node (package.json)

```json
{
  "name": "clipboard-manager",
  "private": true,
  "version": "0.1.0",
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-global-shortcut": "^2.0.0",
    "react": "^18.3.0",
    "react-dom": "^18.3.0",
    "zustand": "^4.5.0",
    "lucide-react": "^0.400.0",
    "react-virtuoso": "^4.7.0",
    "i18next": "^23.10.0",
    "react-i18next": "^14.1.0"
  },
  "devDependencies": {
    "@types/react": "^18.3.0",
    "@types/react-dom": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0",
    "tailwindcss": "^3.4.0",
    "typescript": "^5.4.0",
    "vite": "^5.4.0"
  }
}
```

---

## 测试策略

- Rust 单元测试：`cargo test`，数据库操作、内容解析、去重逻辑、哈希计算
- 前端单元测试：`npm run test:frontend`，使用 Vitest + happy-dom
- 前端组件测试：使用 @testing-library/react

### 测试覆盖目标

- 核心模块（clipboard、storage）：≥ 80% 代码覆盖率
- 前端组件：关键路径覆盖
- 边界条件：数据库满、权限拒绝、文件丢失等异常场景

---

## 验证方案

### 功能验证
1. 启动应用，复制文本/图片/文件，验证记录
2. `Cmd+Shift+V` 呼出窗口
3. 搜索历史记录
4. 点击历史项验证复制
5. 测试置顶、收藏、删除
6. 系统托盘功能

### 性能验证
1. 5000+ 条历史，列表滚动流畅（≥ 60fps）
2. 内存占用 < 100MB
3. 空闲 CPU < 1%
4. 搜索响应 < 100ms

### 打包验证
1. 在 macOS 上执行 `npm run build:macos`，确认 `.app` 和 `.dmg`
2. 在 Windows Runner 上执行 `npm run build:windows` 或 `npm run build:windows:host`，确认 `.msi` 和 `NSIS .exe`
3. 在 Linux Runner 上执行 `npm run build:linux`，确认 `.deb`、`.AppImage`、`.rpm`
4. 安装后正常运行
5. 开机自启正常

---

## 后续扩展

- [ ] iCloud 同步
- [ ] 剪切板内容编辑
- [ ] 快捷短语管理
- [ ] 多设备同步
- [ ] AI 智能分类
- [ ] 数据导出/导入（JSON 格式）

## 许可证

本项目基于 Apache License 2.0 开源，详见 [LICENSE](LICENSE)。
