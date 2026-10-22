/// Never execute renderer-controlled commands, application names or URLs.
/// Return backend-owned constants after validating the complete pair.
pub(crate) fn resolve(kind: &str, target: &str) -> Result<(&'static str, &'static str), String> {
    match (kind, target) {
        ("terminal", "codex") => Ok(("terminal", "codex")),
        ("terminal", "kimi") => Ok(("terminal", "kimi")),
        ("terminal", "opencode auth login") => Ok(("terminal", "opencode auth login")),
        ("app", "Cursor") => Ok(("app", "Cursor")),
        ("app", "Antigravity") => Ok(("app", "Antigravity")),
        ("url", "https://opencode.ai/console") => Ok(("url", "https://opencode.ai/console")),
        _ => Err("不允许的登录操作".into()),
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn rejects_shell_injection_and_arbitrary_urls_and_programs() {
        for (kind, target) in [
            ("terminal", "codex & calc.exe"),
            ("terminal", "codex\nwhoami"),
            ("url", "javascript:alert(1)"),
            ("url", "https://opencode.ai/console?cmd=bad"),
            ("app", "../../cmd"),
            ("app", "codex"),
            ("unknown", "codex"),
        ] {
            assert!(super::resolve(kind, target).is_err());
        }
        for (kind, target) in [
            ("terminal", "codex"),
            ("terminal", "kimi"),
            ("terminal", "opencode auth login"),
            ("app", "Cursor"),
            ("app", "Antigravity"),
            ("url", "https://opencode.ai/console"),
        ] {
            assert_eq!(super::resolve(kind, target).unwrap(), (kind, target));
        }
    }
}
