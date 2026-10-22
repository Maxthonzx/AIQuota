# 常见问题

[English](TROUBLESHOOTING.md) · **简体中文** · [首页](../README.zh-CN.md)

## 应该下载哪个版本？

Mac 打开「苹果菜单 → 关于本机」。芯片为 Apple M1/M2/M3 等 M 系列的，选择 **Apple Silicon**；处理器为 Intel 的，选择 **Intel**。当前版本要求 macOS 13.3 及以上。

Windows 11 打开「设置 → 系统 → 系统信息」，查看「系统类型」。提供的 EXE/MSI 包含 x64 应用，暂不提供原生 Windows ARM64。一般用户可选 EXE，MSI 适合标准 Windows 安装工具。

## 如何校验安装包？

从同一 Release 下载 `SHA256SUMS.txt`，计算所选包的 SHA-256：

```sh
# macOS
shasum -a 256 AIQuota-v1.5.1-macos-apple-silicon.dmg
```

```powershell
# Windows PowerShell
Get-FileHash .\AIQuota-v1.5.1-windows-x64-setup.exe -Algorithm SHA256
```

与清单中相同文件名的那一行比对。校验和可确认文件与发布产物一致，但不等同于发布者签名。

## macOS 或 Windows 提示安全警告

当前包未签名，macOS 包未公证。请先核对仓库、文件名和校验和。在自己控制的 Mac 上，尝试启动后，系统可能在「隐私与安全性」提供「仍要打开」；Windows SmartScreen 可能提供「更多信息」。仅在信任下载内容时继续，组织策略可能不允许运行未签名应用。不要为安装 AIQuota 全局关闭系统保护。

## Windows 打开后白屏

确认 Microsoft WebView2 已安装并更新，再重启应用。安装程序可在需要时获取或更新运行时，此步骤要求联网；同时保持 Windows 11 更新。AIQuota 要求 WebView2 111 及以上。

## 供应商显示未连接

打开配置，按[使用指南](USER_GUIDE.zh-CN.md)确认连接方式。ChatGPT 订阅登录不同于 OpenAI API Key；GLM Coding Plan 需要套餐 Key。快捷登录只负责打开本机 CLI/应用，仍需在其界面完成登录后再刷新 AIQuota。

Antigravity 需要已登录的 IDE 和语言服务保持运行。Windows 检测使用 PowerShell/CIM，不依赖 WMIC。但语言服务未启动、IDE 登录过期或远端接口变化仍可能导致失败；反复刷新不能修复过期凭据。

OpenCode Go 请确认希望使用的 Go 凭据已在 OpenCode 中选为当前账号，或切换为手动 Key。AIQuota 不会替你更改本机当前账号。

## 自定义供应商出现字段或网络错误

- 使用最终的 HTTPS 额度 URL；应用不会自动跟随重定向。
- 确认接口接受 GET，返回 JSON 且小于 1 MB。
- 核对 Bearer/API Key Header 方式和 Header 名称。
- 按实际响应检查点号字段路径；缺失或非数字字段会报错，不会当作零用量。
- 百分比必须为 0–100，0–1 比例需由接口转换；已用量/总额度模式要求总量大于零。
- 不要把聊天地址或 Key 填进 URL。

求助时仅分享缩减后的虚构 JSON，不要提供真实 Key、认证头或原始账号响应。

## 安全存储或迁移失败

解锁 macOS 钥匙串，或检查 Windows 凭据管理器。迁移失败、读回不一致或数据冲突时，AIQuota 会保留旧明文配置。不要删除旧文件或外部登录文件来消除提示；解决系统存储问题后重启。写入失败不会主动回退为明文保存。

回退到 1.3 及更早版本时，旧应用不能读取新安全存储中的 Key，可能需要重新输入；外部 CLI/IDE 登录状态独立保存。

## 重置时间或提醒不符合预期

检查电脑日期、时区和自动校时。应用使用供应商提供的重置时刻，并按当地夏令时规则换算。供应商可能调整重置计划，倒计时归零不代表远端额度已恢复。

恢复提醒要求应用运行、实际观察到耗尽到可用的变化、已获得系统权限，且通知未被系统抑制。可在设置中发送测试通知；macOS 专注模式或 Windows 请勿打扰可能隐藏横幅。

## 如何反馈可复现问题？

提供应用版本、系统版本、处理器架构、供应商名称和复现步骤。普通问题提交到 [Issues](https://github.com/Maxthonzx/AIQuota/issues)，敏感问题按[安全说明](../SECURITY.zh-CN.md)处理。附件和截图应先移除 Key、账号标识、个人机器路径及其他隐私信息。
