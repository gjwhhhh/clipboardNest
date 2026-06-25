# 安全策略

## 报告安全问题

如果你发现安全问题，请不要在公开 Issue 中披露可利用细节。请优先通过 GitHub Security Advisory 或维护者公开联系方式进行私下报告。

报告时请尽量包含：

- 受影响的平台和版本
- 复现步骤
- 可能的影响范围
- 你已经尝试过的缓解方式

## 敏感信息

本项目不应该提交以下内容：

- `.env`、本地配置和真实账号信息
- Apple 证书、`.p12`、provisioning profile、私钥
- API token、访问密钥、Webhook URL
- 构建产物和依赖目录，例如 `node_modules/`、`target/`、`dist/`
