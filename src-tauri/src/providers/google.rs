use super::{ProviderResult, QuotaWindow};

#[path = "google_process.rs"]
mod process;
use process::find_language_server;

const ID: &str = "google";

/// 凭证检测用：Antigravity 语言服务器是否在运行
pub fn ls_available() -> bool {
    find_language_server().is_some()
}

/// Listening TCP ports of a process.
fn listening_ports(pid: u32) -> Vec<u16> {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let Ok(out) = std::process::Command::new("lsof")
            .args(["-a", "-p", &pid.to_string(), "-iTCP", "-sTCP:LISTEN"])
            .output()
        else {
            return vec![];
        };
        let text = String::from_utf8_lossy(&out.stdout);
        text.lines()
            .filter_map(|l| {
                let m = l.rfind("localhost:").or_else(|| l.rfind("*:"))?;
                let rest = &l[m..];
                let start = rest.rfind(':')? + 1;
                let end = rest[start..].find(' ').map(|i| start + i).unwrap_or(rest.len());
                rest[start..end].trim_end_matches(')').parse().ok()
            })
            .collect()
    }
    #[cfg(target_os = "windows")]
    {
        let Ok(out) = process::hidden("netstat").args(["-ano"]).output() else {
            return vec![];
        };
        process::parse_listening_ports(&String::from_utf8_lossy(&out.stdout), pid)
    }
}

async fn query_user_status(port: u16, csrf: &str) -> Option<serde_json::Value> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true) // LS uses a self-signed localhost cert
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?;
    let resp = client
        .post(format!(
            "https://127.0.0.1:{port}/exa.language_server_pb.LanguageServerService/GetUserStatus"
        ))
        .header("Content-Type", "application/json")
        .header("Connect-Protocol-Version", "1")
        .header("x-codeium-csrf-token", csrf)
        .body("{}")
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: serde_json::Value = resp.json().await.ok()?;
    if json.get("userStatus").is_some() {
        Some(json)
    } else {
        None
    }
}

/// Pool grouping mirroring Antigravity's own "Models & Usage" panel.
fn pool_of(model: &str) -> Option<&'static str> {
    let m = model.to_lowercase();
    if m.contains("claude") || m.contains("gpt") {
        return Some("Claude / GPT 系列");
    }
    if m.contains("gemini-4") || m.contains("gemini-4-") {
        return Some("Gemini 4 系列");
    }
    if m.contains("gemini-3") {
        return Some("Gemini 3 系列");
    }
    if m.contains("gemini-2.5") || m.contains("gemini-2-5") {
        return Some("Gemini 2.5 系列");
    }
    None
}

pub async fn fetch(_client: &reqwest::Client) -> Vec<ProviderResult> {
    let Some((pid, csrf)) = find_language_server() else {        return vec![ProviderResult::error(ID, "未检测到运行中的 Antigravity，请启动 IDE 后刷新")
            .with_action(super::ActionHint::app("打开 Antigravity", "Antigravity"))];
    };

    let mut status_json = None;
    for port in listening_ports(pid) {
        if let Some(j) = query_user_status(port, &csrf).await {
            status_json = Some(j);
            break;
        }
    }
    let Some(json) = status_json else {
        return vec![ProviderResult::error(ID, "无法连接 Antigravity 本地服务")];
    };

    let user_status = json.get("userStatus").cloned().unwrap_or_default();
    let configs = user_status
        .get("cascadeModelConfigData")
        .and_then(|d| d.get("clientModelConfigs"))
        .and_then(|c| c.as_array())
        .cloned()
        .unwrap_or_default();

    // 每模型 quotaInfo：无 remainingFraction 字段 = 该池已耗尽（0%）
    let mut pools: std::collections::HashMap<&str, (f64, Option<serde_json::Value>)> =
        std::collections::HashMap::new();
    for cfg in &configs {
        let model = cfg
            .get("modelId")
            .and_then(|m| m.as_str())
            .unwrap_or("");
        let Some(pool) = pool_of(model) else { continue };
        let quota = cfg.get("quotaInfo");
        let remaining = quota
            .and_then(|q| q.get("remainingFraction"))
            .and_then(|f| f.as_f64())
            .unwrap_or(0.0);
        let reset = quota.and_then(|q| q.get("resetTime")).cloned();
        pools
            .entry(pool)
            .and_modify(|(frac, rt)| {
                if remaining < *frac {
                    *frac = remaining;
                    *rt = reset.clone();
                }
            })
            .or_insert((remaining, reset));
    }

    if pools.is_empty() {
        return vec![ProviderResult::error(ID, "Antigravity 返回中没有模型额度信息")];
    }

    let order = [
        "Gemini 4 系列",
        "Gemini 3 系列",
        "Gemini 2.5 系列",
        "Claude / GPT 系列",
    ];
    let mut windows = vec![];
    for label in order {
        if let Some((frac, reset)) = pools.get(label) {
            windows.push(QuotaWindow {
                label: label.to_string(),
                used_percent: Some((1.0 - frac) * 100.0),
                resets_at: reset.clone(),
                status: if *frac <= 0.0 { "limited".into() } else { "ok".into() },
            });
        }
    }

    let mut extras = vec![];
    if let Some(plan) = user_status
        .get("planStatus")
        .and_then(|p| p.get("planInfo"))
        .and_then(|i| i.get("planName"))
        .and_then(|n| n.as_str())
    {
        extras.push(("套餐".into(), plan.to_string()));
    }

    vec![ProviderResult::ok(ID, windows, extras)]
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    #[tokio::test]
    #[ignore = "requires a running, signed-in Antigravity IDE"]
    async fn fetches_live_antigravity_quota_without_wmic() {
        assert!(super::ls_available());
        let results = super::fetch(&reqwest::Client::new()).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, "ok", "{:?}", results[0].error);
        assert!(!results[0].windows.is_empty());
    }
}
