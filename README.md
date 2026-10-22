<div align="center">
  <img src="public/favicon.svg" width="72" height="72" alt="AIQuota" />
  <h1>AIQuota</h1>
  <p>Your AI subscriptions and API balances, in one place.</p>
  <p><strong>English</strong> · <a href="README.zh-CN.md">简体中文</a></p>
  <p><a href="https://github.com/Maxthonzx/AIQuota/releases/latest">Download</a> · <a href="docs/USER_GUIDE.md">User guide</a> · <a href="CHANGELOG.md">Changelog</a> · <a href="https://github.com/Maxthonzx/AIQuota/issues">Issues</a></p>
</div>

AIQuota is a local desktop app for tracking AI subscription quotas, API balances, and reset times. Connect the services you use, remove the ones you do not, and arrange your workspace your way.

## Download

Get **v1.5.1** from [GitHub Releases](https://github.com/Maxthonzx/AIQuota/releases/latest). Choose the package that matches your computer:

| Platform | Download | Requirements |
| --- | --- | --- |
| Mac with Apple Silicon (M series) | [DMG](https://github.com/Maxthonzx/AIQuota/releases/download/v1.5.1/AIQuota-v1.5.1-macos-apple-silicon.dmg) | macOS 13.3+, ARM64 |
| Mac with Intel processor | [DMG](https://github.com/Maxthonzx/AIQuota/releases/download/v1.5.1/AIQuota-v1.5.1-macos-intel.dmg) | macOS 13.3+, x86_64 |
| Windows 11 | [Setup EXE](https://github.com/Maxthonzx/AIQuota/releases/download/v1.5.1/AIQuota-v1.5.1-windows-x64-setup.exe) · [MSI](https://github.com/Maxthonzx/AIQuota/releases/download/v1.5.1/AIQuota-v1.5.1-windows-x64.msi) | Windows 11 x64, Microsoft WebView2 |

Mac `.app.zip` archives, source code, and `SHA256SUMS.txt` are also available on the release page. Windows ARM64 and Linux packages are not provided. Installers are currently unsigned; macOS builds are not notarized. See [installation and troubleshooting](docs/TROUBLESHOOTING.md) for verification and system prompts.

## What you can do

- **Bring subscriptions and API services together.** Track Kimi Code, ChatGPT/Codex, Google AI Pro through Antigravity, OpenCode Go, Cursor, GLM Coding Plan, and DeepSeek.
- **Connect custom providers.** Configure a GET/JSON quota endpoint, authentication, and field mappings to display a balance, used percentage, or used/total quota.
- **Keep the dashboard yours.** Add or remove providers, search and filter the list, and reorder services by dragging or using the up/down buttons. Your order survives restarts.
- **Know when quota returns.** View reset times in your computer's time zone, alongside a countdown. Receive a system notification when an exhausted quota becomes available again.
- **Choose your appearance.** Follow the system theme by default, or choose light or dark mode.
- **Keep credentials under your control.** Manually entered keys use Windows Credential Manager or macOS Keychain. Existing CLI/IDE sign-ins are read without modifying their files.

The application interface is currently in Simplified Chinese. Documentation is available in English and Simplified Chinese.

## Get started

1. Install the package for your operating system and processor.
2. Open AIQuota. On first launch, detected local connections are added automatically.
3. Choose **Add provider / 添加供应商**, then select a preset or custom provider.
4. Connect using the supported local sign-in or the service's appropriate key, then save.
5. Use **Providers / 供应商** to configure, remove, or reorder services. Use **Settings / 设置** for appearance, refresh intervals, and notifications.

| Provider | Connection | What is displayed |
| --- | --- | --- |
| ChatGPT / Codex | Local Codex CLI ChatGPT sign-in | Codex subscription quota windows |
| Kimi Code | Local Kimi Code configuration or a Coding subscription key; additional keyed accounts supported | Subscription quota windows |
| Google AI Pro | Signed-in, running Antigravity and its local language service | Available Gemini/Antigravity quota |
| OpenCode Go | Active local Go sign-in or a Go subscription key | Go quota windows |
| Cursor | Local Cursor IDE sign-in | Subscription usage and quota information |
| GLM Coding Plan | Coding Plan package key | Plan quota windows |
| DeepSeek | DeepSeek API key | API account balances |
| Custom | HTTPS GET endpoint with Bearer, API-key header, or no authentication | Your mapped balance or quota fields |

Subscription credentials and general API keys are not interchangeable. A custom provider needs a **quota/balance endpoint**; a chat-completions URL alone does not provide account usage. The [user guide](docs/USER_GUIDE.md) includes examples and removal behavior.

## Privacy and security

AIQuota does not operate a credential collection server and does not include analytics or telemetry. Authentication is sent to the provider you query; custom providers receive requests at the endpoint you configure.

Saved keys are not returned to the renderer or stored in browser localStorage. Non-secret preferences and connection metadata stay on your computer. Never put a key in a query URL or attach credentials, account databases, or unredacted logs to an issue.

Read the [security policy](SECURITY.md) and [privacy details](docs/USER_GUIDE.md#privacy-and-storage). Some provider integrations use non-public endpoints that can change. Use only accounts you own or are authorized to access, subject to each service's terms. This project is not affiliated with the listed providers.

## Documentation

| Topic | English | 简体中文 |
| --- | --- | --- |
| Overview and downloads | [README](README.md) | [项目说明](README.zh-CN.md) |
| Setup and everyday use | [User guide](docs/USER_GUIDE.md) | [使用指南](docs/USER_GUIDE.zh-CN.md) |
| Installation and connection issues | [Troubleshooting](docs/TROUBLESHOOTING.md) | [常见问题](docs/TROUBLESHOOTING.zh-CN.md) |
| Development and packaging | [Development](docs/DEVELOPMENT.md) | [开发与打包](docs/DEVELOPMENT.zh-CN.md) |
| Release history | [Changelog](CHANGELOG.md) | [更新日志](CHANGELOG.zh-CN.md) |
| Contributing | [Contributing](CONTRIBUTING.md) | [参与贡献](CONTRIBUTING.zh-CN.md) |
| Security | [Security policy](SECURITY.md) | [安全说明](SECURITY.zh-CN.md) |
| Dependency licenses | [Third-party notices](THIRD_PARTY_NOTICES.md) | [第三方许可说明](THIRD_PARTY_NOTICES.zh-CN.md) |

## Build from source

Install Node.js 24, pnpm 11, Rust, and the [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/), then run:

```sh
pnpm install --frozen-lockfile
pnpm test
pnpm build
cargo test --locked --manifest-path src-tauri/Cargo.toml --lib
pnpm exec tauri dev
```

See the [development guide](docs/DEVELOPMENT.md) for native credential-store tests and release commands. GitHub Actions tests and packages Apple Silicon, Intel macOS, and Windows x64 separately.

## License and acknowledgments

AIQuota is licensed under [MIT](LICENSE). Dependency licenses and original copyright notices are preserved in [third-party notices](THIRD_PARTY_NOTICES.md) and `third-party/licenses/`, including in the installers.

The provider-directory and connection flows were informed by [CC Switch](https://github.com/farion1231/cc-switch) and [Magpie](https://github.com/yetone/magpie). Provider names and trademarks belong to their respective owners.
