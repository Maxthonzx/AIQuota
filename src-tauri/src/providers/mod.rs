pub mod codex;
pub mod cursor;
pub mod custom;
pub mod deepseek;
pub mod glm;
pub mod google;
pub mod kimi;
pub mod opencode;

use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct QuotaWindow {
    pub label: String,
    pub used_percent: Option<f64>,
    /// ISO 8601 string or epoch seconds (number), normalized on the frontend
    pub resets_at: Option<serde_json::Value>,
    /// "ok" | "limited" | other provider-specific values
    pub status: String,
}

#[derive(Serialize, Clone)]
pub struct ActionHint {
    pub label: String,
    /// "terminal" | "app" | "url"
    pub kind: String,
    /// 终端命令 / 应用名 / URL
    pub target: String,
}

impl ActionHint {
    pub fn terminal(label: &str, cmd: &str) -> Self {
        Self { label: label.into(), kind: "terminal".into(), target: cmd.into() }
    }
    pub fn app(label: &str, name: &str) -> Self {
        Self { label: label.into(), kind: "app".into(), target: name.into() }
    }
    pub fn url(label: &str, url: &str) -> Self {
        Self { label: label.into(), kind: "url".into(), target: url.into() }
    }
}

#[derive(Serialize)]
pub struct ProviderResult {
    pub id: String,
    /// account label for multi-account providers (e.g. a second Kimi key)
    pub account: Option<String>,
    /// "ok" | "error" | "no_credentials"
    pub status: String,
    pub error: Option<String>,
    pub windows: Vec<QuotaWindow>,
    /// extra key-value rows, e.g. balances
    pub extras: Vec<(String, String)>,
    /// 套餐到期时间（ISO 8601 字符串或秒/毫秒时间戳），接口自动识别；前端显示时手动填写优先
    pub plan_expires_at: Option<serde_json::Value>,
    /// 凭证问题时的快捷操作（打开终端/应用/网页）
    pub action: Option<ActionHint>,
}

/// 各供应商接口响应里可能出现的套餐到期字段名（camelCase 与 snake_case 都列）。
pub const PLAN_EXPIRY_KEYS: &[&str] = &[
    "expireTime",
    "expire_time",
    "planExpireTime",
    "plan_expire_time",
    "expiryTime",
    "expiry_time",
    "planEndTime",
    "plan_end_time",
    "expireAt",
    "expire_at",
    "validUntil",
    "valid_until",
    "subscriptionEndTime",
    "membershipEndTime",
];

/// 在响应对象的第一层按候选字段名查找套餐到期时间（字符串或数字才有效）。
pub fn find_expiry_field(value: &serde_json::Value, keys: &[&str]) -> Option<serde_json::Value> {
    let obj = value.as_object()?;
    for (key, v) in obj {
        if keys.contains(&key.as_str()) && (v.is_string() || v.is_number()) {
            return Some(v.clone());
        }
    }
    None
}

impl ProviderResult {
    pub fn ok(id: &str, windows: Vec<QuotaWindow>, extras: Vec<(String, String)>) -> Self {
        Self {
            id: id.into(),
            account: None,
            status: "ok".into(),
            error: None,
            windows,
            extras,
            plan_expires_at: None,
            action: None,
        }
    }

    pub fn no_credentials(id: &str) -> Self {
        Self {
            id: id.into(),
            account: None,
            status: "no_credentials".into(),
            error: None,
            windows: vec![],
            extras: vec![],
            plan_expires_at: None,
            action: None,
        }
    }

    pub fn error(id: &str, msg: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            account: None,
            status: "error".into(),
            error: Some(msg.into()),
            windows: vec![],
            extras: vec![],
            plan_expires_at: None,
            action: None,
        }
    }

    pub fn with_plan_expiry(mut self, value: Option<serde_json::Value>) -> Self {
        self.plan_expires_at = value;
        self
    }

    pub fn with_account(mut self, label: &str) -> Self {
        self.account = Some(label.into());
        self
    }

    pub fn with_action(mut self, action: ActionHint) -> Self {
        self.action = Some(action);
        self
    }
}

/// Map an HTTP failure status to a user-facing message.
pub fn http_error(status: reqwest::StatusCode) -> String {
    match status.as_u16() {
        401 => "认证已过期，请重新登录对应的 CLI 后重试".into(),
        403 => "权限不足或账号无此服务许可 (403)".into(),
        429 => "额度已耗尽或被限流 (429)".into(),
        s => format!("请求失败 (HTTP {s})"),
    }
}

#[cfg(test)]
mod expiry_tests {
    use super::*;

    #[test]
    fn finds_first_candidate_string_or_number_and_ignores_others() {
        let json = serde_json::json!({"plan_expire_time":"2026-05-12T14:30:00Z"});
        assert_eq!(
            find_expiry_field(&json, PLAN_EXPIRY_KEYS),
            Some(serde_json::json!("2026-05-12T14:30:00Z"))
        );
        // 候选列表里排在前面的字段不存在时，也能命中后面的
        assert_eq!(
            find_expiry_field(&json, &["expireTime", "plan_expire_time"]),
            Some(serde_json::json!("2026-05-12T14:30:00Z"))
        );
        // null、布尔、嵌套对象与未知字段都无效
        let noisy = serde_json::json!({"expireTime":null,"valid_until":true,"deep":{"expireTime":"x"}});
        assert_eq!(find_expiry_field(&noisy, PLAN_EXPIRY_KEYS), None);
        assert_eq!(find_expiry_field(&serde_json::json!([]), PLAN_EXPIRY_KEYS), None);
    }
}
