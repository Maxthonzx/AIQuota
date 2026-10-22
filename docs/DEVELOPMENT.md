# Development and releases

**English** · [简体中文](DEVELOPMENT.zh-CN.md) · [Home](../README.md)

## Prerequisites

- Node.js 24 and pnpm 11.
- A Rust toolchain compatible with the package's `rust-version` (currently 1.90 or newer).
- [Tauri's native prerequisites](https://v2.tauri.app/start/prerequisites/) for the operating system: Xcode command-line tools on macOS, or Visual Studio C++ Build Tools and WebView2 on Windows.
- Python 3.11 or newer for release/documentation checks.

The supported release targets are macOS 13.3+ (Apple Silicon and Intel) and Windows 11 x64. The frontend currently has a Simplified Chinese interface. English is the default documentation language.

## Run locally

```sh
pnpm install --frozen-lockfile
pnpm exec tauri dev
```

The Vite development server uses port 1420. For frontend-only work, `pnpm dev` starts the browser UI; native commands require the Tauri runtime or explicit test fixtures. Do not interpret browser fixtures as live provider data.

The Rust backend handles local credential discovery, authenticated quota requests, native secret storage, and allowlisted login helpers. React renders provider connections, quotas, themes, and preferences. Provider adapters live in `src-tauri/src/providers/`; shared frontend connection types and storage allowlisting live in `src/providers.ts`.

## Required checks

```sh
pnpm test
pnpm lint
pnpm build
cargo test --locked --manifest-path src-tauri/Cargo.toml --lib
python scripts/check_release.py
```

Native credential-store checks write isolated, non-secret test entries and delete them afterward:

```sh
cargo test --locked --manifest-path src-tauri/Cargo.toml --lib native_store_round_trip -- --ignored
```

Run those checks only with an unlocked native credential store. CI uses a temporary macOS test keychain. Tests requiring a real, running Antigravity sign-in are intentionally ignored by default. To run those separately on a machine you control:

```sh
cargo test --locked --manifest-path src-tauri/Cargo.toml --lib providers::google -- --include-ignored
```

Never print or upload the resulting credentials, account data, or raw responses. Existing tests cover process/port parsing, hidden Windows child processes, migration failure/conflict handling, login-target rejection, custom JSON mappings, HTTP authentication/redirect handling, themes, reset-time conversion, and stale request rejection.

## Build installers

Run each command on its matching operating system. Native CPU runners are used in release CI.

```sh
# Apple Silicon Mac
rustup target add aarch64-apple-darwin
pnpm exec tauri build --ci --target aarch64-apple-darwin --bundles app,dmg

# Intel Mac
rustup target add x86_64-apple-darwin
pnpm exec tauri build --ci --target x86_64-apple-darwin --bundles app,dmg

# Windows x64
rustup target add x86_64-pc-windows-msvc
pnpm exec tauri build --ci --target x86_64-pc-windows-msvc --bundles nsis,msi
```

Outputs are under `src-tauri/target/<target>/release/bundle/`. macOS packages declare a minimum system version of 13.3, matching the frontend's modern WebKit requirements. Windows uses the system WebView2 runtime and can bootstrap/update it during installation.

## Release workflow

`.github/workflows/build-desktop.yml` is the sole desktop release workflow. It uses native Apple Silicon, native Intel macOS, and Windows x64 build jobs, plus a separate secret scan. Runner labels follow [GitHub's runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).

Before publishing a version:

1. Update `package.json`, `src-tauri/Cargo.toml`, the root package entry in `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json` together.
2. Update English and Chinese README links, changelogs, and `docs/releases/vX.Y.Z.md` / `vX.Y.Z.zh-CN.md`.
3. Run the checks above, scan the Git history for secrets, and inspect dependency audit results. Preserve third-party legal attribution when reviewing scanner findings.
4. Push `main` or dispatch the workflow. CI tests before packaging, validates Mac CPU architectures and the Windows application's PE architecture, then uploads six platform files.
5. The publishing job adds a source snapshot and SHA-256 manifest, verifies that any existing version tag points to this exact commit, and creates a stable release with bilingual notes.

Artifacts use predictable names: two Mac DMGs, two Mac `.app.zip` archives, one Windows setup EXE, one MSI, one source ZIP, and `SHA256SUMS.txt`. Published versions must not be replaced with builds from a different commit; use a new patch version instead.

## Credentials and logs during development

The app identifier and native secret-store service are `dev.aiquota.app`. Avoid changing them casually, because installed users depend on them for configuration continuity. A development build uses the same production identity; use isolated tests or fixtures instead of launching it against real accounts for screenshots.

Keep `.local`, dependency/build caches, private audit results, and original account data out of commits. Use synthetic values in tests and documentation. Public issues and CI logs must never include keys, tokens, CSRF values, account databases, or personal machine paths.
