# User guide

**English** · [简体中文](USER_GUIDE.zh-CN.md) · [Home](../README.md)

## Install and upgrade

Download the appropriate Apple Silicon, Intel, or Windows 11 x64 package from [Releases](https://github.com/Maxthonzx/AIQuota/releases/latest). On a Mac, open the DMG and drag AIQuota to Applications. On Windows, close AIQuota and run the EXE or MSI installer. Install a newer version over the existing installation; the application identifier and native credential-store service remain stable.

The application uses the system webview. macOS 13.3 or newer is required; Windows requires WebView2 111 or newer. The Windows installer can obtain/update WebView2 when necessary, which requires internet access. Downloads are currently unsigned and macOS packages are not notarized; see [troubleshooting](TROUBLESHOOTING.md).

## Add a provider

On the first launch, AIQuota adds providers whose existing local credentials it detects. Thereafter, your saved provider list is authoritative: removing everything leaves an empty dashboard rather than adding the presets again.

Choose **Add provider / 添加供应商**, search the directory, and select a preset. Give the connection a display name and use the supported connection method:

- **Codex:** sign in to ChatGPT using the local Codex CLI. An OpenAI API key cannot replace this subscription sign-in.
- **Kimi Code:** use the local Kimi Code configuration or enter a Coding subscription key. Additional keyed accounts can be added from its configuration panel after the provider is saved.
- **Antigravity:** install and sign in to Antigravity, and keep its language service running.
- **OpenCode Go:** use the active local Go account or enter its Go subscription key. AIQuota does not switch OpenCode's active account.
- **Cursor:** use the locally signed-in Cursor IDE account.
- **GLM:** use the Coding Plan package key, not a general pay-as-you-go API key.
- **DeepSeek:** enter a DeepSeek platform API key to query balances.

Keys are password fields with a visibility toggle. An existing saved key is never shown again; leave the field empty to keep it. A save error leaves the form open and does not report success. After a successful save, AIQuota queries the current quota. A network or authentication error appears on the provider card and can be resolved by reconfiguring it.

## Configure a custom provider

Choose **Custom provider / 自定义供应商**. Multiple custom connections may use the same service with different names or credentials.

1. Enter a display name and choose **Subscription** or **API**. This category affects filtering; it does not change the remote API.
2. Enter a complete quota-query URL. The endpoint must accept GET and return JSON. HTTPS is required, except for loopback HTTP such as `http://127.0.0.1:8080/quota`.
3. Select Bearer authentication, an API-key header, or no authentication. Header mode supports `x-api-key`, `api-key`, `x-goog-api-key`, and `Authorization`. Bearer mode adds the `Bearer ` prefix; header mode sends the key exactly as entered.
4. Select a display mode and map the response fields.

| Mode | Required fields | Meaning |
| --- | --- | --- |
| Balance | Value path | Available balance; unit is optional |
| Used percentage | Value path | A number from 0 through 100, not a ratio from 0 through 1 |
| Used / total | Value path and total path | Used amount must be non-negative; total must be greater than zero |

For this response:

```json
{
  "data": {
    "balance": "12.50",
    "used": 25,
    "total": 100,
    "reset_at": "2026-11-02T08:53:16Z"
  }
}
```

Use `data.balance` for a balance, or `data.used` and `data.total` for usage. An optional reset-time path can be `data.reset_at`. Numeric strings are supported. Array elements use numeric segments, for example `data.items.0.used`. Paths are simple dot-separated field names, not scripts or full JSONPath expressions.

Reset values may be ISO date/time strings or epoch seconds/milliseconds. Include a time-zone offset or `Z` in date/time strings for an unambiguous instant. AIQuota displays it using your computer's local time zone.

A model chat URL is not a balance API. Endpoints that require POST, OAuth refresh flows, request signing, or JavaScript extraction need a separate provider adapter and are not supported by this generic form. Responses are limited to 1 MB and redirects are not followed. Put credentials in the authentication field, never in the URL.

## Arrange or remove services

Drag a provider's grip on the overview or provider list, use the up/down buttons in the provider list, or focus the grip and press the up/down keys. The same persisted order is used in both views. A provider's additional accounts stay together.

Open a provider's configuration and choose **Remove / 移除**, then confirm:

- Removing a built-in provider hides its related cards and stops quota queries. AIQuota's saved keys and the external app's sign-in remain available if you add it again.
- Removing a custom provider also deletes that connection's saved key. Re-adding it requires configuring it again.
- Removing an additional Kimi account deletes its saved account/key entry from AIQuota. It does not change external Kimi CLI sign-ins.

In-flight results from a removed or changed connection are ignored. Error cards can also be collapsed without removing the service; a successful result displays its quota again.

## Appearance, reset times, and notifications

**Settings / 设置** offers automatic, light, and dark appearance. Automatic is the default and responds to system theme changes. Manual choices survive a restart.

Reset times use your computer's current time zone, including local daylight-saving rules, and appear as `YYYY.MM.DD HH:mm:ss` alongside a countdown. If the provider does not supply a valid reset time, no time is invented. Use the timestamp as provider-reported information; a countdown reaching zero does not guarantee that a remote service has already refreshed its quota.

Automatic refresh can run every 1, 2, 5, or 10 minutes. Recovery notifications require system permission and are sent when an observed exhausted quota becomes available again, rather than merely because the countdown reaches zero. You can send a test notification from settings. AIQuota must be running to refresh and notify.

## Privacy and storage

| Data | Storage or destination |
| --- | --- |
| Manually entered keys | Windows Credential Manager or macOS Keychain; custom connections use individual entries |
| Existing CLI/IDE credentials | Read from the corresponding local application; AIQuota does not rewrite those sign-in files |
| Theme, provider order, names, URLs, field mappings, notification preferences | Local webview storage on your computer |
| Authentication and quota requests | The corresponding provider, or the custom endpoint you configured |
| Antigravity query | Its local language-service endpoint on loopback |

AIQuota has no analytics or telemetry collection service. A saved key is not returned to the renderer. The key you type is sent to the native backend when saved; query requests execute in that backend.

Legacy plaintext AIQuota settings are removed only after a native-store write and successful read-back verification. If storage is locked, unavailable, corrupt, or in conflict, the original settings are retained and an error is shown. Resolve the storage problem rather than deleting the original file. Versions 1.3 and earlier cannot read the newer native-store keys after a downgrade.

For reporting problems without exposing credentials, see [Security](../SECURITY.md).
