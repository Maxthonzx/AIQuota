use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn home() -> Option<PathBuf> {
    dirs::home_dir()
}

// ---- Kimi Code ----

/// First `api_key` found under any `[providers.*]` section of ~/.kimi-code/config.toml
pub fn kimi_api_key() -> Option<String> {
    match provider_key("kimi") {
        Ok(Some(key)) => return Some(key),
        Err(_) => return None,
        Ok(None) => (),
    }
    let path = home()?.join(".kimi-code/config.toml");
    let text = std::fs::read_to_string(path).ok()?;
    let doc: toml::Value = text.parse().ok()?;
    let providers = doc.get("providers")?.as_table()?;
    for (_, v) in providers {
        if let Some(key) = v.get("api_key").and_then(|k| k.as_str()) {
            if !key.is_empty() {
                return Some(key.to_string());
            }
        }
    }
    None
}

// ---- Codex / ChatGPT ----

#[derive(Deserialize)]
struct CodexAuth {
    tokens: Option<CodexTokens>,
}

#[derive(Deserialize)]
struct CodexTokens {
    access_token: Option<String>,
    account_id: Option<String>,
}

/// (access_token, account_id) from ~/.codex/auth.json
pub fn codex_auth() -> Option<(String, String)> {
    let path = home()?.join(".codex/auth.json");
    let text = std::fs::read_to_string(path).ok()?;
    let auth: CodexAuth = serde_json::from_str(&text).ok()?;
    let t = auth.tokens?;
    Some((t.access_token?, t.account_id?))
}

// ---- OpenCode Go ----

// OpenCode v2 使用数据库；只读取用户已经选中的 Go 账号，不自行切换账号。
fn opencode_db_key(path: &std::path::Path) -> Option<String> {
    let conn = rusqlite::Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    ).ok()?;
    let value: String = conn.query_row(
        "SELECT value FROM credential WHERE integration_id = 'opencode-go' AND active = 1 LIMIT 1",
        [],
        |row| row.get(0),
    ).ok()?;
    let json: serde_json::Value = serde_json::from_str(&value).ok()?;
    json.get("key").and_then(|v| v.as_str())
        .filter(|key| !key.trim().is_empty()).map(str::to_owned)
}

/// Read OpenCode v2's active Go credential, with legacy auth.json compatibility.
pub fn opencode_key() -> Option<String> {
    match provider_key("opencode") {
        Ok(Some(key)) => return Some(key),
        Err(_) => return None,
        Ok(None) => (),
    }
    let home = home()?;
    let data = std::env::var_os("XDG_DATA_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| home.join(".local/share"));
    if let Some(key) = opencode_db_key(&data.join("opencode/opencode.db")) {
        return Some(key);
    }
    let candidates = [
        data.join("opencode/auth.json"),
        home.join(".local/share/opencode/auth.json"),
        dirs::data_local_dir()
            .unwrap_or_else(|| home.clone())
            .join("opencode/auth.json"),
    ];
    for path in candidates {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        if let Some(key) = json
            .get("opencode-go")
            .and_then(|g| g.get("key"))
            .and_then(|k| k.as_str())
        {
            if !key.is_empty() {
                return Some(key.to_string());
            }
        }
    }
    None
}

// ---- Cursor ----

fn cursor_db_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        Some(home()?.join("Library/Application Support/Cursor/User/globalStorage/state.vscdb"))
    }
    #[cfg(target_os = "windows")]
    {
        Some(PathBuf::from(std::env::var("APPDATA").ok()?).join("Cursor/User/globalStorage/state.vscdb"))
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        Some(home()?.join(".config/Cursor/User/globalStorage/state.vscdb"))
    }
}

/// (user_id, access_token) from Cursor IDE's state.vscdb.
/// Cookie format used by cursor.com: WorkosCursorSessionToken={userId}%3A%3A{token}
pub fn cursor_auth() -> Option<(String, String)> {
    let path = cursor_db_path()?;
    let conn = rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).ok()?;
    let token: String = conn
        .query_row(
            "SELECT value FROM ItemTable WHERE key = 'cursorAuth/accessToken'",
            [],
            |r| r.get(0),
        )
        .ok()?;
    // JWT payload -> sub, e.g. "auth0|user_01K..."
    let payload = token.split('.').nth(1)?;
    use base64::Engine;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(payload).ok()?;
    let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    let sub = json.get("sub")?.as_str()?;
    let user_id = sub.split('|').nth(1)?.to_string();
    Some((user_id, token))
}

// ---- DeepSeek & extra Kimi accounts (stored in app config dir) ----

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct ExtraAccount {
    pub label: String,
    pub key: String,
}

#[derive(Serialize, Deserialize, Default, PartialEq)]
pub(crate) struct AppSettings {
    deepseek_key: Option<String>,
    glm_key: Option<String>,
    #[serde(default)]
    kimi_accounts: Vec<ExtraAccount>,
}

fn settings_path() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("AIQuota/settings.json"))
}

static SETTINGS_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
fn read_settings() -> Result<AppSettings, String> {
    crate::vault::load(&settings_path().ok_or("无法确定配置目录")?, &crate::vault::NativeStore)
}
fn write_settings(s: &AppSettings) -> Result<(), String> {
    crate::vault::save(&crate::vault::NativeStore, s)
}
pub fn secure_storage_status() -> Result<(), String> {
    let _guard = SETTINGS_LOCK.lock().map_err(|_| "凭据存储锁不可用")?;
    read_settings().map(|_| ())
}

pub fn deepseek_key() -> Option<String> {
    let _guard = SETTINGS_LOCK.lock().ok()?;
    read_settings().ok()?.deepseek_key.filter(|k| !k.is_empty())
}

pub fn set_deepseek_key(key: Option<String>) -> Result<(), String> {
    let _guard = SETTINGS_LOCK.lock().map_err(|_| "凭据存储锁不可用")?;
    let mut s = read_settings()?;
    s.deepseek_key = key.filter(|k| !k.trim().is_empty()).map(|k| k.trim().to_string());
    write_settings(&s)
}

// ---- GLM (user-provided key) ----

pub fn glm_key() -> Option<String> {
    let _guard = SETTINGS_LOCK.lock().ok()?;
    read_settings().ok()?.glm_key.filter(|k| !k.is_empty())
}

pub fn set_glm_key(key: Option<String>) -> Result<(), String> {
    let _guard = SETTINGS_LOCK.lock().map_err(|_| "凭据存储锁不可用")?;
    let mut s = read_settings()?;
    s.glm_key = key.filter(|k| !k.trim().is_empty()).map(|k| k.trim().to_string());
    write_settings(&s)
}

// ---- Extra Kimi accounts ----

pub fn kimi_extra_accounts() -> Vec<ExtraAccount> {
    let _guard = SETTINGS_LOCK.lock().ok();
    read_settings().unwrap_or_default().kimi_accounts
}

/// Returns the labels of configured extra Kimi accounts (keys never leave the backend).
pub fn kimi_extra_labels() -> Vec<String> {
    let _guard = SETTINGS_LOCK.lock().ok();
    read_settings().unwrap_or_default().kimi_accounts.into_iter().map(|a| a.label).collect()
}

pub fn add_kimi_account(label: String, key: String) -> Result<(), String> {
    let label = label.trim().to_string();
    let key = key.trim().to_string();
    if label.is_empty() || key.is_empty() {
        return Err("名称和 key 都不能为空".into());
    }
    let _guard = SETTINGS_LOCK.lock().map_err(|_| "凭据存储锁不可用")?;
    let mut s = read_settings()?;
    if s.kimi_accounts.iter().any(|a| a.label == label) {
        return Err("同名账号已存在".into());
    }
    s.kimi_accounts.push(ExtraAccount { label, key });
    write_settings(&s)
}

pub fn remove_kimi_account(label: &str) -> Result<(), String> {
    let _guard = SETTINGS_LOCK.lock().map_err(|_| "凭据存储锁不可用")?;
    let mut s = read_settings()?;
    s.kimi_accounts.retain(|a| a.label != label);
    write_settings(&s)
}

// Each new manual connection has its own native-store entry, avoiding the
// Windows per-entry size limit when many custom services are configured.
fn key_entry(id: &str) -> Result<keyring::Entry, String> {
    if !matches!(id, "kimi" | "opencode") &&
        !(id.starts_with("custom-") && id.len() <= 80 && id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')) {
        return Err("不支持的凭据标识".into());
    }
    keyring::Entry::new("dev.aiquota.app", &format!("provider-{id}"))
        .map_err(|_| "无法初始化系统安全存储".into())
}

pub fn provider_key(id: &str) -> Result<Option<String>, String> {
    if matches!(id, "deepseek" | "glm") {
        let _guard = SETTINGS_LOCK.lock().map_err(|_| "凭据存储锁不可用")?;
        let settings = read_settings()?;
        return Ok(if id == "deepseek" { settings.deepseek_key } else { settings.glm_key }.filter(|key| !key.is_empty()));
    }
    match key_entry(id)?.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("无法读取系统安全存储，请解锁后重试".into()),
    }
}

pub fn set_provider_key(id: &str, key: Option<String>) -> Result<(), String> {
    let key = key.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty());
    if key.as_ref().is_some_and(|s| s.len() > 2048 || s.contains(['\r', '\n'])) {
        return Err("Key 长度不能超过 2048 字节，且不能包含换行".into());
    }
    if id == "deepseek" { return set_deepseek_key(key); }
    if id == "glm" { return set_glm_key(key); }
    let entry = key_entry(id)?;
    if let Some(key) = key {
        entry.set_password(&key).map_err(|_| "系统安全存储写入失败")?;
        if entry.get_password().map_err(|_| "系统安全存储校验失败")? != key {
            return Err("系统安全存储校验失败".into());
        }
    } else {
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => (),
            Err(_) => return Err("无法移除系统凭据，请解锁后重试".into()),
        }
    }
    Ok(())
}

#[cfg(test)]
mod provider_key_tests {
    use super::*;

    #[test]
    fn rejects_unsupported_secret_identifiers_and_invalid_key_values() {
        for id in ["codex", "custom-../settings", "custom-test\n", "settings-v1"] {
            assert!(key_entry(id).is_err());
        }
        assert!(set_provider_key("custom-validation", Some("a\nb".into())).is_err());
        assert!(set_provider_key("custom-validation", Some("x".repeat(2049))).is_err());
    }

    #[test]
    #[ignore = "writes an isolated custom-provider entry to the native credential store"]
    fn native_store_round_trip() {
        let id = format!("custom-self-test-{}", std::process::id());
        set_provider_key(&id, Some("test-only-placeholder".into())).unwrap();
        let value = provider_key(&id);
        let removed = set_provider_key(&id, None);
        assert_eq!(value.unwrap().as_deref(), Some("test-only-placeholder"));
        removed.unwrap();
        assert_eq!(provider_key(&id).unwrap(), None);
        set_provider_key(&id, None).unwrap();
    }
}

#[cfg(test)]
mod opencode_tests {
    use super::*;

    #[test]
    fn reads_only_active_go_credential_without_changing_database() {
        let path = std::env::temp_dir().join(format!("aiquota-opencode-{}.db", std::process::id()));
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE credential (integration_id TEXT, value TEXT, active INTEGER);").unwrap();
        for (provider, key, active) in [("opencode-go", "inactive", 0), ("opencode", "other", 1), ("opencode-go", "selected", 1)] {
            conn.execute("INSERT INTO credential VALUES (?1, ?2, ?3)", rusqlite::params![provider, serde_json::json!({"key": key}).to_string(), active]).unwrap();
        }
        assert_eq!(opencode_db_key(&path).as_deref(), Some("selected"));
        conn.execute("DELETE FROM credential WHERE integration_id = 'opencode-go' AND active = 1", []).unwrap();
        assert_eq!(opencode_db_key(&path), None);
        conn.execute("INSERT INTO credential VALUES ('opencode-go', 'invalid', 1)", []).unwrap();
        assert_eq!(opencode_db_key(&path), None);
        drop(conn);
        std::fs::remove_file(path).unwrap();
    }
}
