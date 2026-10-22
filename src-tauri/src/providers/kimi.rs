use super::{ProviderResult, QuotaWindow};
use crate::creds;

const ID: &str = "kimi";

async fn fetch_one(client: &reqwest::Client, key: &str, account: Option<&str>) -> ProviderResult {
    let resp = match client
        .get("https://api.kimi.com/coding/v1/usages")
        .bearer_auth(key)
        .header("Accept", "application/json")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return ProviderResult::error(ID, format!("网络错误: {e}")),
    };

    let status = resp.status();
    if !status.is_success() {
        let result = ProviderResult::error(ID, match status.as_u16() {
            401 => "认证失效，请重新登录 Kimi Code 或检查 key".to_string(),
            404 => "该账号无订阅额度查询权限 (404)".to_string(),
            _ => super::http_error(status),
        });
        return match (status.as_u16(), account) {
            (401, None) => result.with_action(super::ActionHint::terminal("打开终端登录", "kimi")),
            _ => result,
        };
    }

    let json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(e) => return ProviderResult::error(ID, format!("响应解析失败: {e}")),
    };

    let usages = json.get("usages").cloned().unwrap_or(json.clone());
    let mut windows = vec![];
    for (field, label) in [("limit_5h", "5小时窗口"), ("limit_7d", "7天窗口")] {
        if let Some(w) = usages.get(field) {
            let ratio = w.get("used_ratio").and_then(|r| r.as_f64());
            windows.push(QuotaWindow {
                label: label.into(),
                used_percent: ratio.map(|r| r * 100.0),
                resets_at: w.get("reset_time").cloned(),
                status: "ok".into(),
            });
        }
    }

    // 兼容 camelCase 的 usage 汇总（7 天窗口的点数形式）
    let mut extras = vec![];
    if let Some(u) = json.get("usage") {
        let used = u.get("used").and_then(|v| v.as_str()).unwrap_or("-");
        let limit = u.get("limit").and_then(|v| v.as_str()).unwrap_or("-");
        extras.push(("周额度点数".into(), format!("{used} / {limit}")));
    }

    // 加油包余额（两种命名都兼容）
    let wallet = json
        .get("booster_wallet")
        .or_else(|| json.get("boosterWallet"));
    if let Some(w) = wallet {
        if let Some(b) = w.get("balance") {
            let left = b
                .get("amountLeft")
                .or_else(|| b.get("amount_left"))
                .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())));
            let total = b
                .get("amount")
                .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())));
            if let (Some(left), Some(total)) = (left, total) {
                // UNIT_CURRENCY 定点数，1e8 = 1 元
                extras.push((
                    "加油包".into(),
                    format!("¥{:.2} / ¥{:.2}", left / 1e8, total / 1e8),
                ));
            }
        }
    }

    let mut result = if windows.is_empty() && extras.is_empty() {
        ProviderResult::error(ID, "响应中没有已知的额度字段")
    } else {
        ProviderResult::ok(ID, windows, extras)
    };
    if let Some(label) = account {
        result = result.with_account(label);
    }
    result
}

pub async fn fetch(client: &reqwest::Client) -> Vec<ProviderResult> {
    let mut out = vec![];

    if let Some(key) = creds::kimi_api_key() {
        out.push(fetch_one(client, &key, None).await);
    }

    for acc in creds::kimi_extra_accounts() {
        // Preserve identity on early HTTP/network failures as well as successes.
        out.push(fetch_one(client, &acc.key, Some(&acc.label)).await.with_account(&acc.label));
    }

    if out.is_empty() {
        out.push(ProviderResult::no_credentials(ID));
    }
    out
}
