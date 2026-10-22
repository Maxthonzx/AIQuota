use super::{ProviderResult, QuotaWindow};
use crate::creds;

const ID: &str = "glm";

const ENDPOINTS: [&str; 2] = [
    "https://open.bigmodel.cn/api/monitor/usage/quota/limit", // 国内版
    "https://api.z.ai/api/monitor/usage/quota/limit",         // 国际版
];

async fn fetch_from(client: &reqwest::Client, url: &str, key: &str) -> Result<serde_json::Value, String> {
    // 智谱的 key 直接作为 Authorization 值，不带 Bearer 前缀
    let resp = client
        .get(url)
        .header("Authorization", key)
        .send()
        .await
        .map_err(|e| format!("网络错误: {e}"))?;
    let status = resp.status().as_u16();
    let json: serde_json::Value = resp.json().await.map_err(|e| format!("响应解析失败: {e}"))?;
    if status == 429 {
        return Err("额度已耗尽或被限流 (429)，稍后自动恢复显示".into());
    }
    if status == 401 || status == 403 {
        return Err("401".into());
    }
    let code = json.get("code").and_then(|c| c.as_i64()).unwrap_or(0);
    let success = json.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
    if code == 1001 {
        return Err("401".into());
    }
    if !success && code != 200 {
        let msg = json.get("msg").and_then(|m| m.as_str()).unwrap_or("未知错误");
        return Err(format!("查询失败: {msg}"));
    }
    Ok(json)
}

fn window_label(unit: Option<i64>, reset_ms: Option<i64>) -> String {
    match unit {
        Some(3) => "5小时窗口".into(),
        Some(6) => "每周窗口".into(),
        _ => {
            // unit 缺失时用重置时间启发式：<=6 小时视为 5h 窗口
            match reset_ms {
                Some(ms) if ms - now_ms() <= 6 * 3600 * 1000 => "5小时窗口".into(),
                Some(_) => "每周窗口".into(),
                None => "额度窗口".into(),
            }
        }
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub async fn fetch(client: &reqwest::Client) -> Vec<ProviderResult> {
    let Some(key) = creds::glm_key() else {
        return vec![ProviderResult::no_credentials(ID)];
    };

    // 先试国内版，401 再试国际版（两端账号体系不同，401 说明 key 属于另一端）
    let mut last_err = String::new();
    let mut json = None;
    for url in ENDPOINTS {
        match fetch_from(client, url, &key).await {
            Ok(j) => {
                json = Some(j);
                break;
            }
            Err(e) => {
                let is_auth = e == "401";
                last_err = if is_auth {
                    "API Key 无效或与账号区域不匹配".to_string()
                } else {
                    e
                };
                if !is_auth {
                    break;
                }
            }
        }
    }
    let Some(json) = json else {
        return vec![ProviderResult::error(ID, last_err)];
    };

    let Some(data) = json.get("data") else {
        return vec![ProviderResult::error(ID, "响应中没有 data 字段")];
    };

    let mut extras = vec![];
    if let Some(level) = data.get("level").and_then(|l| l.as_str()) {
        extras.push(("套餐等级".into(), level.to_string()));
    }

    let mut windows = vec![];
    if let Some(limits) = data.get("limits").and_then(|l| l.as_array()) {
        for item in limits {
            let kind = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
            if kind != "TOKENS_LIMIT" && kind != "CREDIT_LIMIT" {
                continue; // TIME_LIMIT 等是 MCP 工具额度，跳过
            }
            let reset = item.get("nextResetTime").and_then(|t| t.as_i64());
            // 已用百分比：优先新形态 currentValue/usage，退回 percentage
            let pct = match (
                item.get("currentValue").and_then(|v| v.as_f64()),
                item.get("usage").and_then(|v| v.as_f64()),
            ) {
                (Some(cur), Some(total)) if total > 0.0 => Some(cur / total * 100.0),
                _ => item.get("percentage").and_then(|p| p.as_f64()),
            };
            windows.push(QuotaWindow {
                label: window_label(item.get("unit").and_then(|u| u.as_i64()), reset),
                used_percent: pct,
                resets_at: reset.map(|ms| serde_json::Value::from(ms)),
                status: "ok".into(),
            });
        }
    }

    if windows.is_empty() {
        return vec![ProviderResult::error(ID, "响应中没有套餐额度窗口（key 可能不是套餐 Key）")];
    }

    vec![ProviderResult::ok(ID, windows, extras)]
}
