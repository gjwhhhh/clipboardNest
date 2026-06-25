# 贡献指南

感谢你愿意参与 ClipBoard Manager 的开发。

## 开发环境

- Node.js 20+
- Rust stable
- Tauri 2.x 相关系统依赖

安装前端依赖：

```bash
npm ci
cd src-mobile && npm ci
```

运行桌面端：

```bash
npm run start
```

运行移动端前端：

```bash
cd src-mobile
npm run dev
```

## 提交前检查

```bash
npm run build
npm run test:frontend
cargo test --workspace
```

如果只修改某个子项目，可以运行更小范围的检查，但请在 PR 中说明验证范围。

## 代码风格

- Rust 代码遵循 `cargo fmt` 和 `cargo clippy` 的常规建议。
- 前端使用 React + TypeScript + Tailwind CSS。
- 非必要不做大范围格式化，避免无关 diff。

## macOS 签名

仓库不包含任何 Apple 证书、私钥或签名身份。发布者需要在自己的环境中配置签名和公证所需的环境变量。
