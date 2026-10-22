use super::{ProviderResult, QuotaWindow};
use crate::creds;

const ID: &str = "cursor";

fn session_cookie(user_id: &str, token: &str) -> String {
    format!("WorkosCursorSessionToken={user_id}%3A%3A{token}")
}

pub async fn fetch(client: &reqwest::Client) -> Vec<ProviderResult> {
    let Some((user_id, token)) = creds::cursor_auth() else {
        return vec![ProviderResult::no_credentials(ID)];
    };
    let cookie = session_cookie(&user_id, &token);

    // 1) 套餐月度额度
    let summary = client
        .get("https://cursor.com/api/usage-summary")
        .header("Cookie", &cookie)
        .send()
        .await;

    let summary = match summary {
        Ok(r) => r,
        Err(e) => return vec![ProviderResult::error(ID, format!("网络错误: {e}"))],
    };
    if summary.status().as_u16() == 401 || summary.status().as_u16() == 403 {
        return vec![ProviderResult::error(ID, "Cursor 登录已过期，打开一次 Cursor IDE 即可自动刷新")
            .with_action(super::ActionHint::app("打开 Cursor", "Cursor"))];
    }
    if !summary.status().is_success() {
        return vec![ProviderResult::error(ID, super::http_error(summary.status()))];
    }
    let summary_json: serde_json::Value = match summary.json().await {
        Ok(j) => j,
        Err(e) => return vec![ProviderResult::error(ID, format!("响应解析失败: {e}"))],
    };

    let mut windows = vec![];
    let mut extras = vec![];

    if let Some(m) = summary_json.get("membershipType").and_then(|v| v.as_str()) {
        extras.push(("套餐".into(), m.to_string()));
    }
    if let Some(plan) = summary_json
        .get("individualUsage")
        .and_then(|u| u.get("plan"))
    {
        let pct = plan.get("totalPercentUsed").and_then(|p| p.as_f64());
        windows.push(QuotaWindow {
            label: "月度包含额度".into(),
            used_percent: pct,
            resets_at: summary_json.get("billingCycleEnd").cloned(),
            status: "ok".into(),
        });
        let used = plan.get("used").and_then(|v| v.as_f64());
        let limit = plan.get("limit").and_then(|v| v.as_f64());
        if let (Some(u), Some(l)) = (used, limit) {
            extras.push(("已用 / 总量".into(), format!("{u:.0} / {l:.0}")));
        }
    }
    if let Some(on_demand) = summary_json
        .get("individualUsage")
        .and_then(|u| u.get("onDemand"))
        .and_then(|o| o.get("used"))
        .and_then(|v| v.as_f64())
    {
        if on_demand > 0.0 {
            extras.push(("按需付费已用".into(), format!("${:.2}", on_demand / 100.0)));
        }
    }

    // 2) Grok 周额度池（X 套餐共享池，内部代号 Sand）
    let sand = client
        .post("https://cursor.com/api/dashboard/get-sand-usage-status")
        .header("Cookie", &cookie)
        .header("Origin", "https://cursor.com")
        .json(&serde_json::json!({}))
        .send()
        .await;

    if let Ok(r) = sand {
        if r.status().is_success() {
            if let Ok(j) = r.json::<serde_json::Value>().await {
                let has_pool = j
                    .get("hasNonZeroIncludedLimit")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                if has_pool {
                    let raw = j.get("usagePercent").and_then(|p| p.as_f64());
                    // 兼容 0–1 和 0–100 两种形态
                    let pct = raw.map(|p| if p <= 1.0 { p * 100.0 } else { p });
                    windows.push(QuotaWindow {
                        label: "Grok 周额度池".into(),
                        used_percent: pct,
                        resets_at: j.get("nextResetTimestampUtc").cloned(),
                        status: if j
                            .get("hasAvailableUsage")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true)
                        {
                            "ok".into()
                        } else {
                            "limited".into()
                        },
                    });
                }
            }
        }
    }

    if windows.is_empty() {
        return vec![ProviderResult::error(ID, "响应中没有额度数据")];
    }

    vec![ProviderResult::ok(ID, windows, extras)]
}
