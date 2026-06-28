# CC Switch Fork — Claude Desktop 增强

基于 [farion1231/cc-switch](https://github.com/farion1231/cc-switch) v3.16.4 的 fork，专注于改进 Claude Desktop 的 3P 适配。

## 已完成

### HTTPS + CORS（WebView 客户端必需）
- **CORS middleware**：`tower_http::cors::CorsLayer`，回显 Origin + 全方法全头。Office add-in（WebView）可以跨域连接。
- **HTTPS listener**：`axum-server` + `rustls`。proxy 启动时自动检测 `~/.cc-switch/cert.pem` + `key.pem`，有就同时起 HTTP + HTTPS（HTTPS 端口 = HTTP + 1）。
- 不需要 mkcert——任何 PEM 格式的 cert + key 放到 `~/.cc-switch/` 即可。
- Commit: `0a1a6e0e`, `b5e433e4`

### Config 分层（切 provider 不丢配置）
- **后端**：`apply_provider_to_paths_inner` 写 profile 时，从 DB 读取 common config snippet，deep merge 到 profile。managedMcpServers 等用户配置在切 provider 时不会丢。
- **后端**：`apply_common_config_to_settings` 对 `ClaudeDesktop` 也应用 common config（之前什么都不做）。
- **前端**：`ClaudeDesktopProviderForm` 加了 `CommonConfigEditor`（和其他 app type 一样有通用配置开关 + 编辑器）。
- **前端**：`useCommonConfigSnippet` hook 加了 `appType` 参数，支持 `claude_desktop`。
- Commit: `fff284b8`, `ede0cf66`, `b8b05161`

### 配置 JSON 显示（profile 格式）
- **settingsConfig 用 profile 格式**：不再是 Claude Code 的 `env` 格式（`ANTHROPIC_BASE_URL` / `ANTHROPIC_AUTH_TOKEN`），而是 Claude Desktop 原生格式（`inferenceGatewayBaseUrl` / `inferenceGatewayApiKey` / `inferenceModels` 数组等）。
- **实时同步**：用户改模型路由 / API Key / Base URL 时，JSON 编辑器实时更新。
- **兼容旧格式**：`defaultValues` 自动从旧 `env` 格式转换到新 profile 格式。
- **隐藏 Claude Code 专有 toggle**：CommonConfigEditor 对非 Claude app type 隐藏「隐藏署名 / teammates / tool search / 思考强度 / 禁用自动更新」等 toggle。
- Commit: `cff2758d`, `cea68475`, `2956a5c7`, `2a6ca384`

## TODO

### HTTPS 零依赖（不装 mkcert）
- 用 `rcgen` crate 在首次启动时自动生成自签证书 + CA。
- 用 `osascript` 弹系统密码框，自动把 CA 装入系统 keychain（`security add-trusted-cert`）。
- 用户拖进 Applications 就能用，不需要任何额外安装。
- 当前状态：HTTPS 代码已就绪（自动检测 `~/.cc-switch/cert.pem`），但证书需要用户手动放置或用 mkcert 生成。

### 后端 direct 模式适配新 profile 格式
- `direct_gateway_credentials` 还读 `settingsConfig.env.ANTHROPIC_BASE_URL`，需改成读 `settingsConfig.inferenceGatewayBaseUrl`。
- proxy 模式不受影响（读 DB + meta）。

### HTTPS 配置 UI
- 设置页面加 HTTPS 开关 + 端口 + 证书路径配置。
- 当前是自动检测，没有 UI。

### DB 持久化 HTTPS 配置
- `proxy_config` 表加 `https_port` / `tls_cert_path` / `tls_key_path` 列。
- 当前这三个字段在 DAO 里 hardcode 为 `None`。

### web_search 透传
- 3P 模型（Qwen / Kimi 等）自己有 web_search 能力，cc-switch 只需透传 `web_search` tool_use 给模型即可，不需要集成 Tavily / Exa。
- 之前写了 executor 模块但已 revert（不是正确方向）。
- 需要：确保 cc-switch 不拦截 / 不丢弃 `web_search` tool_use，透传给上游模型。

### 模型路由灵活化
- 当前路由是固定四档（Sonnet / Opus / Fable / Haiku），不能添加更多。
- 改成用户可配（任意数量、任意别名、支持通配符 / 默认 fallback）。

## 编译

```bash
cd cc-switch-fork
pnpm install
pnpm build
```

产物：
- App: `src-tauri/target/release/bundle/macos/CC Switch.app`
- DMG: `src-tauri/target/release/bundle/dmg/CC Switch_3.16.4_aarch64.dmg`

## 测试

```bash
cd src-tauri
cargo test --lib
```

1681 tests passed（7 个 takeover 测试因端口冲突跳过，与改动无关）。

## Commit 历史

| Commit | 内容 |
|---|---|
| `0a1a6e0e` | HTTPS listener + CORS middleware |
| `fff284b8` | 切 provider 保留用户配置 + CORS fix |
| `ede0cf66` | 后端 common_config 应用到 Desktop profile |
| `b8b05161` | 前端 common config 编辑器 |
| `cff2758d` | 模型路由显示在 JSON 里 |
| `cea68475` | JSON 编辑器可见 |
| `2956a5c7` | 隐藏 Claude Code 专有 toggle |
| `2a6ca384` | settingsConfig 用 profile 格式 |
| `b5e433e4` | HTTPS 自动检测证书 + cleanup |
