# Changelog

**English** · [简体中文](CHANGELOG.zh-CN.md)

Dates use the maintainer's time zone (Asia/Shanghai). The history below describes product releases; it is independent of the public repository's clean Git history.

## [1.5.1] — 2026-10-22

### Changed

- Prepared a clean-history public repository with MIT licensing and retained third-party legal notices.
- Made English the default README and added matching Simplified Chinese documentation, user guides, troubleshooting, contributor guidance, security information, and release notes.
- Split macOS downloads into native Apple Silicon and Intel packages, with explicit architecture checks in CI.
- Added clearly named Windows 11 x64 EXE/MSI packages and SHA-256 checksums for downloadable artifacts.
- Set the macOS minimum to 13.3 to match the webview features used by the frontend, and require a sufficiently recent Windows WebView2 runtime.
- Consolidated release automation into one workflow that tests all three targets before publishing.

## 1.5.0 — 2026-10-22

### Added

- Overview, provider management, and settings pages with a preset directory and a unified connection form.
- Custom subscription/API providers using GET/JSON balance and quota endpoints, Bearer or API-key header authentication, and configurable field paths.
- Provider removal, searching, filtering, and persistent drag/up/down ordering.
- Manual Kimi Code and OpenCode Go keys, alongside supported local sign-ins.
- Key visibility controls, safe-storage feedback, and visible configuration errors.

### Fixed

- Stale queries cannot restore a removed provider or overwrite a newer configuration.
- Drag completion remains reliable when a card changes during a refresh.
- Kimi account labels are preserved on early request failures.

## 1.4.1 — 2026-10-22

- Added absolute reset times in the computer's current time zone, for example `2026.11.02 16:53:16`, while retaining countdowns.
- Supported ISO timestamps, epoch seconds/milliseconds, and local daylight-saving-time conversion.

## 1.4.0 — 2026-10-22

- Added automatic, light, and dark appearance modes with persistent preferences and live system-theme updates.
- Moved manually entered credentials to native operating-system storage, with verified migration from legacy plaintext settings.
- Restricted login helpers to exact backend-owned targets and enabled the production content security policy.
- Updated TLS certificate-validation dependencies and added MIT licensing, privacy information, and dependency notices.

## 1.3.0 — 2026-10-22

- Prevented Windows background process discovery and refresh commands from opening console windows or stealing focus.
- Kept deliberately opened login terminals visible.

## Earlier development

- Replaced Windows WMIC process discovery with PowerShell/CIM and corrected PID/port parsing.
- Established local integrations for the supported providers and desktop packaging.

[1.5.1]: https://github.com/Maxthonzx/AIQuota/releases/tag/v1.5.1
