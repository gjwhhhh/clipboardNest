# Clipboard Nest

English | [中文](#中文)

Clipboard Nest is a lightweight, local-first clipboard history manager built with Tauri, React, Rust, and SQLite. It helps you keep recent text, rich text, image, and file clipboard entries searchable and ready to reuse.

The desktop app is the primary target today. macOS has the most complete clipboard integration, while Windows and Linux packaging are supported through the build scripts and GitHub Actions workflows. Mobile-related code is still experimental.

## Features

- Local clipboard history for text, rich text fallback, images, and copied files.
- Fast search and type filters for browsing previous clipboard entries.
- Pin, favorite, delete, and copy actions for history items.
- Global shortcut support for quickly opening the desktop window.
- System tray integration for desktop workflows.
- SQLite storage with migrations and local retention controls.
- Cross-platform bundle scripts for macOS, Windows, and Linux.

## Tech Stack

| Layer | Technology |
| --- | --- |
| App shell | Tauri 2 |
| Backend | Rust |
| Frontend | React, TypeScript |
| Styling | Tailwind CSS |
| State | Zustand |
| Storage | SQLite, rusqlite |
| Tests | Cargo test, Vitest |

## Project Status

Clipboard Nest is in early open-source development. The core desktop clipboard workflow is usable, but APIs, UI details, packaging behavior, and mobile support may still change before a stable release.

## Requirements

- Node.js 20 or later
- Rust stable
- Platform dependencies required by Tauri 2
- macOS, Windows, or Linux for desktop development

For Linux packaging, install the WebKitGTK/AppIndicator dependencies used by Tauri. The GitHub Actions workflow in `.github/workflows/build-desktop-bundles.yml` is the best reference for CI packages.

## Getting Started

Install dependencies:

```bash
npm ci
```

Run the desktop app in development mode:

```bash
npm run start
```

Run the frontend only:

```bash
npm run dev
```

Run checks:

```bash
npm run build
npm run test:frontend
cargo test --workspace
```

## Packaging

Build the current platform:

```bash
npm run build:release
```

Build platform-specific bundles:

```bash
npm run build:macos
npm run build:windows
npm run build:linux
```

Bundle output locations:

| Platform | Output |
| --- | --- |
| macOS | `src-tauri/target/release/bundle/macos/`, `src-tauri/target/release/bundle/dmg/` |
| Windows | `src-tauri/target/release/bundle/msi/`, `src-tauri/target/release/bundle/nsis/` |
| Linux | `src-tauri/target/release/bundle/deb/`, `src-tauri/target/release/bundle/appimage/`, `src-tauri/target/release/bundle/rpm/` |

macOS signing and notarization are not configured with private credentials in this repository. See `docs/macos-signing-guide.md` for the expected local or CI environment variables.

## Repository Layout

```text
.
├── core/                  # Shared Rust clipboard and storage logic
├── src/                   # Desktop React frontend
├── src-tauri/             # Desktop Tauri application
├── src-mobile/            # Experimental mobile frontend
├── src-mobile-tauri/      # Experimental mobile Tauri application
├── scripts/               # Build helper scripts
├── docs/                  # Project notes and packaging guides
└── .github/workflows/     # CI and release workflows
```

## Privacy and Security

Clipboard data can contain sensitive information. Clipboard Nest stores history locally and the repository does not include certificates, private keys, Apple signing identities, or production secrets.

Please do not commit local databases, `.env` files, signing certificates, generated bundles, or dependency directories. See `SECURITY.md` for security reporting guidance.

## Contributing

Contributions are welcome. Please read `CONTRIBUTING.md` before opening a pull request.

Recommended checks before submitting changes:

```bash
npm run build
npm run test:frontend
cargo test --workspace
```

## License

Clipboard Nest is licensed under the Apache License 2.0. See `LICENSE` for details.

---

# 中文

[English](#clipboard-nest) | 中文

Clipboard Nest 是一个轻量、本地优先的剪切板历史管理工具，基于 Tauri、React、Rust 和 SQLite 构建。它可以记录文字、富文本降级内容、图片和复制的文件，并让这些历史内容可以被快速搜索和再次使用。

当前项目以桌面端为主要目标。macOS 的剪切板集成最完整，Windows 和 Linux 通过构建脚本及 GitHub Actions 工作流支持打包。移动端相关代码仍处于实验阶段。

## 功能特性

- 记录文本、富文本降级内容、图片和复制文件的剪切板历史。
- 支持搜索和类型筛选，方便浏览历史内容。
- 支持置顶、收藏、删除和再次复制历史项。
- 支持全局快捷键快速呼出桌面窗口。
- 支持系统托盘，适合桌面常驻使用。
- 使用 SQLite 本地存储，并包含数据库迁移和保留策略。
- 提供 macOS、Windows、Linux 的跨平台打包脚本。

## 技术栈

| 层级 | 技术 |
| --- | --- |
| 应用壳 | Tauri 2 |
| 后端 | Rust |
| 前端 | React, TypeScript |
| 样式 | Tailwind CSS |
| 状态管理 | Zustand |
| 存储 | SQLite, rusqlite |
| 测试 | Cargo test, Vitest |

## 项目状态

Clipboard Nest 目前处于早期开源开发阶段。桌面端核心剪切板流程已经可用，但 API、界面细节、打包行为和移动端支持在稳定版发布前仍可能调整。

## 环境要求

- Node.js 20 或更高版本
- Rust stable
- Tauri 2 所需的系统依赖
- macOS、Windows 或 Linux 桌面开发环境

Linux 打包需要安装 Tauri 所需的 WebKitGTK/AppIndicator 依赖。CI 依赖可参考 `.github/workflows/build-desktop-bundles.yml`。

## 快速开始

安装依赖：

```bash
npm ci
```

启动桌面端开发模式：

```bash
npm run start
```

只启动前端：

```bash
npm run dev
```

运行检查：

```bash
npm run build
npm run test:frontend
cargo test --workspace
```

## 打包

构建当前平台：

```bash
npm run build:release
```

按平台构建安装包：

```bash
npm run build:macos
npm run build:windows
npm run build:linux
```

产物位置：

| 平台 | 产物目录 |
| --- | --- |
| macOS | `src-tauri/target/release/bundle/macos/`, `src-tauri/target/release/bundle/dmg/` |
| Windows | `src-tauri/target/release/bundle/msi/`, `src-tauri/target/release/bundle/nsis/` |
| Linux | `src-tauri/target/release/bundle/deb/`, `src-tauri/target/release/bundle/appimage/`, `src-tauri/target/release/bundle/rpm/` |

仓库不会包含任何私有签名凭据。macOS 签名和公证需要在本地或 CI 环境中自行配置，详见 `docs/macos-signing-guide.md`。

## 项目结构

```text
.
├── core/                  # 共享 Rust 剪切板和存储逻辑
├── src/                   # 桌面端 React 前端
├── src-tauri/             # 桌面端 Tauri 应用
├── src-mobile/            # 实验性移动端前端
├── src-mobile-tauri/      # 实验性移动端 Tauri 应用
├── scripts/               # 构建辅助脚本
├── docs/                  # 项目说明和打包文档
└── .github/workflows/     # CI 和发布工作流
```

## 隐私与安全

剪切板内容可能包含敏感信息。Clipboard Nest 将历史记录存储在本地，仓库中不包含证书、私钥、Apple 签名身份或生产环境密钥。

请不要提交本地数据库、`.env` 文件、签名证书、生成的安装包或依赖目录。安全问题报告方式见 `SECURITY.md`。

## 贡献

欢迎参与贡献。提交 Pull Request 前请先阅读 `CONTRIBUTING.md`。

建议在提交前运行：

```bash
npm run build
npm run test:frontend
cargo test --workspace
```

## 许可证

Clipboard Nest 基于 Apache License 2.0 开源，详见 `LICENSE`。
