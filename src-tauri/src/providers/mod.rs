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
    /// 凭证问题时的快捷操作（打开终端/应用/网页）
    pub action: Option<ActionHint>,
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
            action: None,
        }
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
