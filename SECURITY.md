# Security and privacy

**English** · [简体中文](SECURITY.zh-CN.md) · [Home](README.md)

## Report a vulnerability

Use the repository's **Security → Report a vulnerability** feature for private reports. Do not open a public issue containing a key, token, cookie, authentication JSON, account database, real account screenshot, or unredacted log. If private reporting is unavailable, open an issue requesting a private contact method without including exploit details or sensitive data.

Describe the affected version and platform, impact, and a minimal reproduction using invented values. Report provider account problems to the corresponding provider; AIQuota cannot change provider entitlements.

## Supported versions

Security fixes target the latest release. Downloads are currently unsigned, and macOS packages are not notarized. Compare downloads with the release's SHA-256 manifest. Build and install only code you trust: this desktop process can read the current user's provider credentials.

## Data handling

- AIQuota runs locally and has no analytics, telemetry, or credential collection service.
- Authentication is sent to the queried provider or your configured custom endpoint. Credentials therefore do not remain exclusively on the machine during authorized API requests.
- Manually entered keys are stored in macOS Keychain / Windows Credential Manager. Saved keys are not returned to the renderer or written to localStorage. Non-secret connection metadata and preferences use local webview storage.
- External CLI/IDE sign-in files are read without being rewritten. Legacy plaintext AIQuota settings are removed only after verified migration to native storage. Failures or conflicts preserve the old settings and report an error; there is no plaintext fallback.
- Removing a built-in provider stops queries and preserves credentials for re-adding it. Removing a custom connection deletes its stored key. See the [user guide](docs/USER_GUIDE.md) for details.

## Runtime boundaries

The backend accepts only exact, backend-owned login helper targets. The renderer cannot supply an arbitrary shell command or application. Production CSP restricts scripts, renderer network destinations, and embedded content.

Custom quota adapters accept HTTPS GET/JSON endpoints; loopback HTTP is allowed for local services. Responses are limited to 1 MB, redirects are not followed, scripts are not evaluated, and credential-bearing network errors or response bodies are not displayed. Only configure an endpoint you intend to authorize, and do not put credentials in URLs.

Antigravity's local integration accepts its self-signed certificate on loopback. That exception does not apply to remote provider endpoints. Some provider APIs are non-public and may change independently of AIQuota.

## Repository and release hygiene

Do not commit credentials, `.env` files, sign-in JSON, local databases, private screenshots, or machine-specific logs. Keep original dependency copyright and license notices; those are legal attribution, not secrets to redact.

The release workflow scans reachable Git history for secrets, tests the frontend and native backend, exercises isolated native credential entries, verifies packaged CPU architectures, and publishes checksums. These checks reduce known risks but are not proof that every possible vulnerability or secret is absent. Never point tests at a real account to demonstrate a security issue.
