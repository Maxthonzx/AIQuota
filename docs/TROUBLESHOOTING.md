# Troubleshooting

**English** · [简体中文](TROUBLESHOOTING.zh-CN.md) · [Home](../README.md)

## Which download should I choose?

On a Mac, use **Apple menu → About This Mac**. A chip named Apple M1/M2/M3 or another M-series chip needs the **Apple Silicon** package. A processor named Intel needs the **Intel** package. The current release requires macOS 13.3 or newer.

For Windows 11, use **Settings → System → About** and check **System type**. The provided EXE/MSI contains the x64 application. A native Windows ARM64 build is not provided. The EXE is the usual interactive installer; MSI is an alternative for standard Windows installation tooling.

## Verify a downloaded file

Download `SHA256SUMS.txt` from the same release, then calculate the checksum of the package you selected:

```sh
# macOS
shasum -a 256 AIQuota-v1.5.1-macos-apple-silicon.dmg
```

```powershell
# Windows PowerShell
Get-FileHash .\AIQuota-v1.5.1-windows-x64-setup.exe -Algorithm SHA256
```

Compare the result with the line for that exact filename in the manifest. A checksum identifies matching release files; it is not a publisher signature.

## macOS or Windows shows a security prompt

Current downloads are unsigned; macOS releases are not notarized. Confirm the repository, release filename, and checksum first. On a Mac you control, macOS may offer **Privacy & Security → Open Anyway** after an attempted launch. Windows SmartScreen may offer **More info**. Continue only if you trust the download; organizational policy can prevent running an unsigned application. Do not disable system protection globally to install AIQuota.

## The window is blank on Windows

Make sure Microsoft WebView2 is installed and current, then restart the app. The installer can obtain/update the runtime when necessary; that step needs a network connection. Keep Windows 11 updated. AIQuota requires WebView2 111 or newer.

## A provider says it is not connected

Open its configuration and confirm the required connection type in the [user guide](USER_GUIDE.md). A ChatGPT subscription sign-in is different from an OpenAI API key; GLM Coding Plan needs its package key. Login shortcuts open the appropriate local CLI/application, and you must finish its own sign-in flow before refreshing AIQuota.

For Antigravity, keep the signed-in IDE and its language service running. Windows detection uses PowerShell/CIM and does not require WMIC. A missing language service, an expired IDE sign-in, or a changed provider endpoint can still prevent a query. Repeatedly refreshing cannot repair an expired credential.

For OpenCode Go, ensure the Go credential you intend to use is active in OpenCode, or choose manual Key mode. AIQuota does not switch the active local account.

## A custom provider returns a mapping or network error

- Use the exact final HTTPS quota URL; redirects are deliberately not followed.
- Check that the endpoint accepts GET and returns JSON smaller than 1 MB.
- Verify Bearer versus API-key header authentication and the selected header name.
- Check dot-separated paths against the actual response. A missing or non-numeric field is an error, not zero usage.
- Percentages must be 0–100; ratios need conversion by the endpoint. Used/total mode requires a positive total.
- Do not paste a chat-completions URL or a key into the URL field.

Share only an invented, reduced JSON example when asking for help. Never include the actual Key, headers, or raw account response.

## Safe storage or migration fails

Unlock macOS Keychain or check Windows Credential Manager. AIQuota preserves old plaintext settings if migration fails, verification disagrees, or data conflicts. Do not delete the old file or sign-in files to silence the error. Restart after resolving the native storage issue. A failed write is never intentionally replaced by plaintext storage.

Downgrading to 1.3 or earlier does not restore compatibility with the new native-store keys; the older app may need keys re-entered. External CLI/IDE sign-ins are separate.

## A reset time or notification looks wrong

Check your computer's date, time zone, and automatic time settings. The app uses provider-reported reset instants and local daylight-saving rules. A provider can change its reset schedule, and a countdown reaching zero is not proof that the remote quota has recovered.

Recovery notifications need the app to be running, an observed exhausted-to-available transition, system permission, and system notifications not being suppressed. Send a test reminder from settings. macOS Focus and Windows Do Not Disturb can suppress banners.

## Report a reproducible problem

Include app version, operating-system version, processor architecture, provider name, and steps to reproduce. Use [Issues](https://github.com/Maxthonzx/AIQuota/issues) for ordinary bugs and the [security policy](../SECURITY.md) for sensitive reports. Remove keys, account identifiers, machine paths, and personal information before attaching files or screenshots.
