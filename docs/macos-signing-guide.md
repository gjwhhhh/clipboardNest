# macOS 应用签名与分发指南

## 问题现象

打包的 macOS app 在其他电脑上无法打开，但在开发者电脑上可以运行。

**根本原因：**
- 当前应用使用 `adhoc` 签名（`Signature=adhoc`）
- 没有设置 `TeamIdentifier`
- 没有进行 Apple 公证（Notarization）

macOS 10.15+ 要求应用必须：
1. 使用有效的开发者证书签名
2. 经过 Apple 公证（Notarization）
3. 才能在其他电脑上正常运行（无需用户手动操作）

---

## 方案一：临时打开方式（无需开发者账号）

用户拿到应用后，可以通过以下方式手动运行：

### 方式一：右键打开（最简单）
1. 右键点击 `.app` 文件
2. 选择"打开"
3. 在弹出的对话框中再次点击"打开"

### 方式二：系统偏好设置
1. 尝试双击打开应用（会被阻止）
2. 打开"系统偏好设置 > 安全性与隐私 > 通用"
3. 底部会显示"xxx 已被阻止"，点击"仍要打开"

### 方式三：命令行解除隔离（推荐）
```bash
# 如果双击提示"无法打开"且没有"安全性与隐私"选项
xattr -cr ~/Downloads/剪切板管理器.app

# 如果仍然无法打开，重新签名
codesign --force --deep --sign - ~/Downloads/剪切板管理器.app

# 打开应用
open ~/Downloads/剪切板管理器.app
```

### 一键脚本
```bash
xattr -cr ~/Downloads/剪切板管理器.app && codesign --force --deep --sign - ~/Downloads/剪切板管理器.app && open ~/Downloads/剪切板管理器.app
```

**注意：** 这些方式需要用户每次手动操作，体验较差。

---

## 方案二：使用 Apple 开发者证书签名（推荐）

### 前置条件
- Apple 开发者账号（$99/年）
- Apple Developer ID Application 证书（在 Apple Developer 后台下载）
- 创建 App-Specific Password（在 appleid.apple.com 生成）

### 步骤

#### 1. 下载并安装证书
- 登录 [Apple Developer 后台](https://developer.apple.com/account/resources/certificates)
- 下载 "Developer ID Application" 证书
- 双击安装到钥匙串

#### 2. 设置环境变量

在终端中设置以下环境变量（或创建 `.env` 文件）：

```bash
# 设置签名身份（格式："Developer ID Application: 你的名字 (TEAM_ID)"）
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"

# 设置证书密码（base64 编码的 .p12 文件）
export APPLE_CERTIFICATE="base64-encoded-certificate.p12"
export APPLE_CERTIFICATE_PASSWORD="certificate-password"

# 设置 Apple ID 和 App-Specific Password
export APPLE_ID="your-apple-id@example.com"
export APPLE_PASSWORD="app-specific-password"
export APPLE_TEAM_ID="YOUR_TEAM_ID"
```

#### 3. 在 Tauri 配置中添加签名配置

编辑 `src-tauri/tauri.conf.json`，在 `bundle` 字段中添加 `macOS` 配置：

```json
{
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "resources/icons/32x32.png",
      "resources/icons/128x128.png",
      "resources/icons/128x128@2x.png",
      "resources/icons/icon.icns",
      "resources/icons/icon.ico"
    ],
    "macOS": {
      "signingIdentity": null,
      "minimumSystemVersion": "10.15"
    }
  }
}
```

**注意：** `signingIdentity` 设为 `null` 表示使用环境变量 `APPLE_SIGNING_IDENTITY` 的值。

#### 4. 重新打包

```bash
npm run build:macos
```

#### 5. 验证签名

```bash
codesign -dvv /Applications/剪切板管理器.app
# 应该看到：
# Signature=iid
# TeamIdentifier=YOUR_TEAM_ID
```

---

## 方案三：创建 .env 文件存储证书信息（可选）

在项目根目录创建 `.env` 文件（不要提交到 git）：

```env
APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
APPLE_CERTIFICATE="base64-encoded-certificate.p12"
APPLE_CERTIFICATE_PASSWORD="certificate-password"
APPLE_ID="your-apple-id@example.com"
APPLE_PASSWORD="app-specific-password"
APPLE_TEAM_ID="YOUR_TEAM_ID"
```

**注意：** 确保 `.env` 文件在 `.gitignore` 中，不要提交到版本库。

---

## 常见问题

### Q: 为什么我自己电脑可以运行，其他电脑不行？
A: macOS 会信任本地构建的应用（有 adhoc 签名），但对从网络下载或从其他电脑传来的应用会进行 Gatekeeper 检查，要求有有效的开发者签名。

### Q: 没有开发者账号就无法分发应用吗？
A: 可以分发，但用户需要手动操作（右键打开或在系统偏好设置中允许）。体验较差，且每次安装都需要手动操作。

### Q: 公证（Notarization）是什么？
A: 公证是 Apple 的安全检查流程，确保应用没有恶意软件。从 macOS 10.15 开始，分发的应用必须经过公证才能在其他电脑上正常运行。

### Q: 如何获取 App-Specific Password？
A: 登录 [appleid.apple.com](https://appleid.apple.com)，在"安全"部分生成 App-Specific Password。

---

## 参考资源

- Tauri macOS 签名文档: https://tauri.app/distribute/distribute-signing/
- Apple 公证文档: https://developer.apple.com/documentation/xcode/notarizing_macos_software_before_distribution
- Apple Developer ID 证书: https://developer.apple.com/account/resources/certificates
