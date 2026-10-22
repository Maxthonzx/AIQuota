# 开发与发布

[English](DEVELOPMENT.md) · **简体中文** · [首页](../README.zh-CN.md)

## 前置依赖

- Node.js 24、pnpm 11。
- 满足包内 `rust-version` 的 Rust 工具链，目前为 1.90 及以上。
- 对应系统的 [Tauri 原生依赖](https://v2.tauri.app/start/prerequisites/)：macOS 使用 Xcode 命令行工具；Windows 使用 Visual Studio C++ Build Tools 与 WebView2。
- Python 3.11 及以上，用于发布与文档检查。

发布目标为 macOS 13.3+（Apple Silicon、Intel）和 Windows 11 x64。前端界面目前为简体中文，文档默认语言为英文。

## 本地运行

```sh
pnpm install --frozen-lockfile
pnpm exec tauri dev
```

Vite 开发服务器使用 1420 端口。仅修改前端时可运行 `pnpm dev`；原生命令需要 Tauri 运行时或明确的测试桩。不要把浏览器演示数据当作真实供应商数据。

Rust 后端负责本机凭据发现、带认证的额度请求、原生安全存储和白名单登录入口。React 负责供应商连接、额度展示、主题与偏好。供应商适配器位于 `src-tauri/src/providers/`，前端共享连接类型与存储字段白名单位于 `src/providers.ts`。

## 必做检查

```sh
pnpm test
pnpm lint
pnpm build
cargo test --locked --manifest-path src-tauri/Cargo.toml --lib
python scripts/check_release.py
```

原生安全存储测试使用隔离的虚构测试项，结束后删除：

```sh
cargo test --locked --manifest-path src-tauri/Cargo.toml --lib native_store_round_trip -- --ignored
```

原生测试需解锁系统凭据存储，CI 使用临时 macOS 钥匙串。需要真实 Antigravity 登录的测试默认忽略，可在自己控制的机器上单独运行：

```sh
cargo test --locked --manifest-path src-tauri/Cargo.toml --lib providers::google -- --include-ignored
```

不要打印或上传过程中获取的凭据、账号数据或原始响应。现有测试覆盖进程/端口解析、Windows 隐藏子进程、迁移失败和冲突、登录目标拒绝、自定义 JSON 映射、HTTP 认证与重定向、主题、重置时间换算以及旧请求结果拒绝。

## 构建安装包

在对应操作系统上运行以下命令，发布 CI 使用原生 CPU runner：

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

产物位于 `src-tauri/target/<target>/release/bundle/`。macOS 包声明最低 13.3，以匹配前端需要的现代 WebKit；Windows 使用系统 WebView2，安装时可获取或更新运行时。

## 发布流程

`.github/workflows/build-desktop.yml` 是唯一桌面发布工作流，分别使用原生 Apple Silicon、原生 Intel macOS 和 Windows x64 构建任务，并独立扫描密钥。Runner 标签遵循 [GitHub 官方说明](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)。

发布前：

1. 同步更新 `package.json`、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock` 的应用条目与 `src-tauri/tauri.conf.json`。
2. 同步英文/中文 README 下载链接、更新日志和 `docs/releases/vX.Y.Z.md` / `vX.Y.Z.zh-CN.md`。
3. 运行上述检查，扫描 Git 历史中的密钥并查看依赖审计结果；处理扫描结果时保留第三方法定版权信息。
4. 推送 `main` 或手动触发工作流。CI 先测试再打包，检查 Mac CPU 架构和 Windows 应用 PE 架构，然后上传六个系统包。
5. 发布任务增加源码快照和 SHA-256 清单，确认同名版本标签只指向当前提交，再生成带双语说明的正式 Release。

产物包括两个 Mac DMG、两个 Mac `.app.zip`、一个 Windows EXE、一个 MSI、一个源码 ZIP，以及 `SHA256SUMS.txt`。不得使用另一提交覆盖已发布版本；有变更应升级补丁版本。

## 开发时的凭据与日志

应用标识和原生凭据服务为 `dev.aiquota.app`，不要随意改动，以免现有用户配置中断。开发构建使用相同身份；截图应使用隔离测试或虚构数据，不要对真实账号启动开发版本。

`.local`、依赖/构建缓存、私人审计结果和原始账号数据不得提交。测试与文档使用虚构值；公开 Issue 和 CI 日志不得包含 Key、token、CSRF、账号数据库或个人机器路径。
