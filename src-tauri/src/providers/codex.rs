use super::{ProviderResult, QuotaWindow};
use crate::creds;

const ID: &str = "codex";

fn window_label(secs: Option<i64>, fallback: &str) -> String {
    match secs {
        Some(18000) => "5小时窗口".into(),
        Some(604800) => "每周窗口".into(),
        Some(s) if s % 604800 == 0 => format!("{}天窗口", s / 86400),
        Some(s) if s % 3600 == 0 => format!("{}小时窗口", s / 3600),
        _ => fallback.into(),
    }
}

fn parse_window(w: &serde_json::Value, fallback: &str) -> Option<QuotaWindow> {
    let secs = w.get("limit_window_seconds").and_then(|s| s.as_i64());
    Some(QuotaWindow {
        label: window_label(secs, fallback),
        used_percent: w.get("used_percent").and_then(|p| p.as_f64()),
        resets_at: w.get("reset_at").cloned(),
        status: "ok".into(),
    })
}

pub async fn fetch(client: &reqwest::Client) -> Vec<ProviderResult> {
    let Some((token, account_id)) = creds::codex_auth() else {
        return vec![ProviderResult::no_credentials(ID)];
    };

    let resp = match client
        .get("https://chatgpt.com/backend-api/wham/usage")
        .bearer_auth(&token)
        .header("ChatGPT-Account-Id", &account_id)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return vec![ProviderResult::error(ID, format!("网络错误: {e}"))],
    };

    let status = resp.status();
    if !status.is_success() {
        let result = ProviderResult::error(ID, match status.as_u16() {
            401 => "登录已过期，打开终端跑一次 codex 即可自动刷新".to_string(),
            _ => super::http_error(status),
        });
        return vec![if status.as_u16() == 401 {
            result.with_action(super::ActionHint::terminal("打开终端刷新", "codex"))
        } else {
            result
        }];
    }

    let json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(e) => return vec![ProviderResult::error(ID, format!("响应解析失败: {e}"))],
    };

    let mut windows = vec![];
    let rl = json.get("rate_limit");
    if let Some(rl) = rl {
        if let Some(w) = rl.get("primary_window").and_then(|w| parse_window(w, "主窗口")) {
            windows.push(w);
        }
        if let Some(w) = rl.get("secondary_window").and_then(|w| parse_window(w, "次窗口")) {
            windows.push(w);
        }
    }

    let mut extras = vec![];
    if let Some(plan) = json.get("plan_type").and_then(|p| p.as_str()) {
        extras.push(("套餐".into(), plan.to_string()));
    }
    if let Some(rl) = rl {
        let limited = rl.get("limit_reached").and_then(|v| v.as_bool()).unwrap_or(false)
            || !rl.get("allowed").and_then(|v| v.as_bool()).unwrap_or(true);
        extras.push((
            "限流状态".into(),
            if limited { "已被限流".into() } else { "正常".into() },
        ));
        for w in windows.iter_mut() {
            if w.used_percent.map(|p| p >= 100.0).unwrap_or(false) {
                w.status = "limited".into();
            }
        }
    }

    if windows.is_empty() {
        return vec![ProviderResult::error(ID, "响应中没有 rate_limit 窗口")];
    }

    vec![ProviderResult::ok(ID, windows, extras)]
}
