use super::ProviderResult;
use crate::creds;

const ID: &str = "deepseek";

pub async fn fetch(client: &reqwest::Client) -> Vec<ProviderResult> {
    let Some(key) = creds::deepseek_key() else {
        return vec![ProviderResult::no_credentials(ID)];
    };

    let resp = match client
        .get("https://api.deepseek.com/user/balance")
        .bearer_auth(&key)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return vec![ProviderResult::error(ID, format!("网络错误: {e}"))],
    };

    if !resp.status().is_success() {
        let msg = match resp.status().as_u16() {
            401 => "API Key 无效或已过期，请重新填写".to_string(),
            _ => super::http_error(resp.status()),
        };
        return vec![ProviderResult::error(ID, msg)];
    }

    let json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(e) => return vec![ProviderResult::error(ID, format!("响应解析失败: {e}"))],
    };

    let mut extras = vec![];
    if let Some(infos) = json.get("balance_infos").and_then(|b| b.as_array()) {
        for info in infos {
            let currency = info
                .get("currency")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string();
            let total = info
                .get("total_balance")
                .and_then(|t| t.as_str())
                .unwrap_or("-");
            let granted = info
                .get("granted_balance")
                .and_then(|t| t.as_str())
                .unwrap_or("0");
            let topped = info
                .get("topped_up_balance")
                .and_then(|t| t.as_str())
                .unwrap_or("0");
            extras.push((format!("总额 ({currency})"), total.to_string()));
            extras.push((format!("充值余额 ({currency})"), topped.to_string()));
            extras.push((format!("赠送余额 ({currency})"), granted.to_string()));
        }
    }
    extras.push(("重置周期".into(), "预付费余额制，无重置".into()));

    vec![ProviderResult::ok(ID, vec![], extras)]
}
