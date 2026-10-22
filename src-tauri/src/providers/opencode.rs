use super::{ActionHint, ProviderResult, QuotaWindow};
use crate::creds;

const ID: &str = "opencode";

pub async fn fetch(client: &reqwest::Client) -> Vec<ProviderResult> {
    let Some(key) = creds::opencode_key() else {
        return vec![ProviderResult::no_credentials(ID)
            .with_action(ActionHint::terminal("打开终端登录", "opencode auth login"))];
    };

    let resp = match client
        .get("https://opencode.ai/zen/go/v1/usage")
        .bearer_auth(&key)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return vec![ProviderResult::error(ID, format!("网络错误: {e}"))],
    };

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        // key 彻底过期：任何客户端都救不回来，必须去 Console 重建
        if status == 401 && body.contains("api_key_expired") {
            return vec![ProviderResult::error(
                ID,
                "API Key 已过期，需要到 OpenCode Console 创建新 key（桌面端和 CLI 共用同一 key）",
            )
            .with_action(ActionHint::url(
                "打开 Console 创建新 Key",
                "https://opencode.ai/console",
            ))];
        }
        let result = ProviderResult::error(ID, super::http_error(reqwest::StatusCode::from_u16(status).unwrap_or_default()));
        return vec![if status == 401 {
            result.with_action(ActionHint::terminal("打开终端登录", "opencode auth login"))
        } else {
            result
        }];
    }

    let json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(e) => return vec![ProviderResult::error(ID, format!("响应解析失败: {e}"))],
    };

    let usage = match json.get("usage") {
        Some(u) => u,
        None => return vec![ProviderResult::error(ID, "响应中缺少 usage 字段")],
    };

    let mut windows = vec![];
    for (field, label) in [
        ("rolling", "5小时窗口"),
        ("weekly", "每周窗口"),
        ("monthly", "每月窗口"),
    ] {
        if let Some(w) = usage.get(field) {
            windows.push(QuotaWindow {
                label: label.into(),
                used_percent: w.get("percent").and_then(|p| p.as_f64()),
                resets_at: w.get("resetsAt").cloned(),
                status: w
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("ok")
                    .to_string(),
            });
        }
    }

    if windows.is_empty() {
        return vec![ProviderResult::error(ID, "响应中没有已知的额度窗口")];
    }

    vec![ProviderResult::ok(ID, windows, vec![])]
}
