//! Process discovery shared by credential checks and quota queries.

/// Windows: 任何控制台程序（netstat/powershell/...）都必须带 CREATE_NO_WINDOW 启动，
/// 否则会创建 conhost 控制台窗口，一闪而过并抢走前台焦点。
#[cfg(target_os = "windows")]
pub(super) fn hidden(prog: &str) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    let mut cmd = std::process::Command::new(prog);
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    cmd
}

pub(super) fn find_language_server() -> Option<(u32, String)> {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    let output = std::process::Command::new("ps")
        .args(["-ww", "-A", "-o", "pid,command"])
        .output()
        .ok()?;
    #[cfg(target_os = "windows")]
    let output = {
        // CIM is available on Windows versions where WMIC has been removed.
        // Emit an explicit PID + tab + command line, never a formatted table.
        hidden("powershell.exe")
            .args([
                "-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
                "$ErrorActionPreference = 'Stop'; [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false); Get-CimInstance Win32_Process -Filter \"Name LIKE '%language_server%'\" | ForEach-Object { if ($_.CommandLine) { [Console]::WriteLine(([string]$_.ProcessId + [char]9 + $_.CommandLine)) } }",
            ])
            .output()
            .ok()?
    };
    if !output.status.success() {
        return None;
    }
    parse_processes(&String::from_utf8_lossy(&output.stdout))
}

fn parse_processes(text: &str) -> Option<(u32, String)> {
    text.lines().find_map(|line| {
        let line = line.trim();
        let end = line.find(char::is_whitespace)?;
        let pid = line[..end].parse::<u32>().ok().filter(|pid| *pid != 0)?;
        let command = line[end..].trim_start();
        let lower = command.to_ascii_lowercase();
        if !lower.contains("language_server") || !lower.contains("antigravity") {
            return None;
        }
        let mut args = command.split_whitespace();
        while let Some(arg) = args.next() {
            let arg = arg.trim_matches('"');
            let token = if arg == "--csrf_token" {
                args.next()?.trim_matches('"')
            } else if let Some(token) = arg.strip_prefix("--csrf_token=") {
                token.trim_matches('"')
            } else {
                continue;
            };
            if !token.is_empty() && !token.starts_with("--") {
                return Some((pid, token.to_string()));
            }
        }
        None
    })
}

#[cfg(any(target_os = "windows", test))]
pub(super) fn parse_listening_ports(text: &str, pid: u32) -> Vec<u16> {
    text.lines()
        .filter_map(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            if fields.len() != 5
                || fields[0] != "TCP"
                || fields[3] != "LISTENING"
                || fields[4].parse::<u32>().ok()? != pid
            {
                return None;
            }
            fields[1].rsplit(':').next()?.parse().ok()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "windows")]
    #[test]
    fn hidden_child_has_no_console() {
        const CHILD_ENV: &str = "AIQUOTA_CONSOLE_TEST_CHILD";
        if std::env::var_os(CHILD_ENV).is_some() {
            #[link(name = "kernel32")]
            unsafe extern "system" {
                fn GetConsoleWindow() -> *mut std::ffi::c_void;
            }
            assert!(unsafe { GetConsoleWindow() }.is_null());
            return;
        }
        let exe = std::env::current_exe().unwrap();
        let output = hidden(exe.to_str().unwrap())
            .args(["hidden_child_has_no_console", "--nocapture"])
            .env(CHILD_ENV, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    }

    #[test]
    fn parses_windows_paths_and_csrf_formats() {
        for arg in [
            "--csrf_token secret",
            "--csrf_token=secret",
            "--csrf_token \"secret\"",
            "--csrf_token=\"secret\"",
        ] {
            let text =
                format!("31652\t\"C:\\Program Files\\Antigravity\\language_server.exe\" {arg}\r\n");
            assert_eq!(parse_processes(&text), Some((31652, "secret".into())));
        }
    }

    #[test]
    fn skips_headers_unrelated_and_incomplete_processes() {
        let text = "PID Command\n12 other_language_server --csrf_token wrong\n13 antigravity language_server --csrf_token\n14 antigravity language_server --csrf_token --other\n15 antigravity language_server --csrf_token=\n16 antigravity language_server --csrf_token good";
        assert_eq!(parse_processes(text), Some((16, "good".into())));
        assert_eq!(parse_processes(""), None);
    }

    #[test]
    fn preserves_unix_process_format() {
        assert_eq!(
            parse_processes("  123 /opt/antigravity/language_server --csrf_token=secret"),
            Some((123, "secret".into()))
        );
    }

    #[test]
    fn matches_exact_pid_and_listening_state() {
        let text = "TCP 127.0.0.1:5555 0.0.0.0:0 LISTENING 9123\nTCP [::1]:5556 [::]:0 LISTENING 123\nTCP 127.0.0.1:5557 0.0.0.0:0 ESTABLISHED 123\nUDP 0.0.0.0:5558 *:* 123";
        assert_eq!(parse_listening_ports(text, 123), vec![5556]);
    }

    #[cfg(target_os = "windows")]
    #[test]
    #[ignore = "requires a running Antigravity language server"]
    fn detects_live_antigravity_and_its_ports() {
        let (pid, token) =
            find_language_server().expect("Antigravity must be detected without WMIC");
        assert!(!token.is_empty());
        let output = hidden("netstat").args(["-ano"]).output().unwrap();
        assert!(!parse_listening_ports(&String::from_utf8_lossy(&output.stdout), pid).is_empty());
    }
}
