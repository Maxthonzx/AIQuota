# Contributing

**English** · [简体中文](CONTRIBUTING.zh-CN.md) · [Home](README.md)

Bug reports, documentation improvements, and focused provider fixes are welcome. Please check existing issues first and describe the expected behavior, actual behavior, app version, operating system, and CPU architecture. Use invented examples and redact credentials and personal information. Follow [Security](SECURITY.md) for sensitive reports.

## Make a change

1. Fork the public repository and create a focused branch.
2. Follow the [development guide](docs/DEVELOPMENT.md) to run the app and relevant checks.
3. Preserve existing Windows and macOS behavior. Keep platform-specific process commands in the native backend and keep background Windows children hidden.
4. For provider changes, document the authentication type and quota response, use synthetic fixtures, and add meaningful coverage for parsing or credential boundaries. Do not introduce arbitrary shell execution, remote scripts, plaintext secret storage, or telemetry.
5. Update English and Simplified Chinese user-facing documentation when behavior changes. If you cannot provide a translation, state that in the pull request so it can be completed before release.
6. Explain the concrete behavior change and relevant validation in the pull request. Keep unrelated refactors out of the change.

Third-party copyright and license notices must be retained. New dependencies need a clear purpose and a license review. Contributions are provided under the project's MIT license, subject to any retained third-party licenses.

Do not include real account databases, authentication JSON, private screenshots, local audit exports, machine-specific paths, or personal/work email addresses in commits or attachments. Configure a GitHub noreply commit email if you do not want your personal email in public Git history.
