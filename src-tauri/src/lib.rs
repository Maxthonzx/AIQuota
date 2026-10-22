mod creds;
mod vault;
mod login;
mod providers;

use providers::ProviderResult;
use serde::Serialize;
#[cfg(target_os = "macos")]
use std::path::PathBuf;

struct AppState {
    client: reqwest::Client,
}

#[derive(Serialize)]
struct CredentialStatus {
    storage_error: Option<String>,
    kimi: bool,
    codex: bool,
    opencode: bool,
    google: bool,
    deepseek: bool,
    cursor: bool,
    glm: bool,
}

#[tauri::command]
async fn fetch_all(state: tauri::State<'_, AppState>) -> Result<Vec<ProviderResult>, String> {
    let client = state.client.clone();
    let (deepseek, opencode, kimi, codex, google, cursor, glm) = tokio::join!(
        providers::deepseek::fetch(&client),
        providers::opencode::fetch(&client),
        providers::kimi::fetch(&client),
        providers::codex::fetch(&client),
        providers::google::fetch(&client),
        providers::cursor::fetch(&client),
        providers::glm::fetch(&client),
    );
    Ok([deepseek, opencode, kimi, codex, google, cursor, glm]
        .into_iter()
        .flatten()
        .collect())
}

#[tauri::command]
async fn fetch_provider(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Vec<ProviderResult>, String> {
    let client = state.client.clone();
    Ok(match id.as_str() {
        "deepseek" => providers::deepseek::fetch(&client).await,
        "opencode" => providers::opencode::fetch(&client).await,
        "kimi" => providers::kimi::fetch(&client).await,
        "codex" => providers::codex::fetch(&client).await,
        "google" => providers::google::fetch(&client).await,
        "cursor" => providers::cursor::fetch(&client).await,
        "glm" => providers::glm::fetch(&client).await,
        _ => return Err(format!("unknown provider: {id}")),
    })
}

#[tauri::command]
fn credential_status() -> CredentialStatus {
    CredentialStatus {
        storage_error: creds::secure_storage_status().err(),
        kimi: creds::kimi_api_key().is_some() || !creds::kimi_extra_labels().is_empty(),
        codex: creds::codex_auth().is_some(),
        opencode: creds::opencode_key().is_some(),
        google: providers::google::ls_available(),
        deepseek: creds::deepseek_key().is_some(),
        cursor: creds::cursor_auth().is_some(),
        glm: creds::glm_key().is_some(),
    }
}

#[tauri::command]
fn set_deepseek_key(key: Option<String>) -> Result<(), String> {
    creds::set_deepseek_key(key)
}

#[tauri::command]
fn set_glm_key(key: Option<String>) -> Result<(), String> {
    creds::set_glm_key(key)
}

#[tauri::command]
fn has_provider_key(id: String) -> Result<bool, String> {
    creds::provider_key(&id).map(|key| key.is_some())
}

#[tauri::command]
fn set_provider_key(id: String, key: Option<String>) -> Result<(), String> {
    creds::set_provider_key(&id, key)
}

#[tauri::command]
async fn fetch_custom(config: providers::custom::CustomConfig) -> ProviderResult {
    providers::custom::fetch(&config).await
}

/// 打开快捷操作：终端命令 / 应用 / 网页
#[tauri::command]
fn open_login(kind: String, target: String) -> Result<(), String> {
    let (kind, target) = login::resolve(&kind, &target)?;
    #[cfg(target_os = "macos")]
    {
        match kind {
            "url" => {
                std::process::Command::new("open")
                    .arg(&target)
                    .spawn()
                    .map_err(|e| e.to_string())?;
            }
            "app" => {
                std::process::Command::new("open")
                    .args(["-a", &target])
                    .spawn()
                    .map_err(|e| e.to_string())?;
            }
            "terminal" => {
                // 写一个 .command 脚本让 Terminal 打开，不需要「控制 Terminal」自动化权限
                let dir = dirs::cache_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .join("AIQuota");
                std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
                let script_path = dir.join("aiquota-login.command");
                std::fs::write(
                    &script_path,
                    format!("#!/bin/sh\n{target}\necho \"\"\necho \"完成后可直接关闭本窗口\"\n"),
                )
                .map_err(|e| e.to_string())?;
                std::process::Command::new("chmod")
                    .args(["+x", &script_path.to_string_lossy()])
                    .output()
                    .map_err(|e| e.to_string())?;
                std::process::Command::new("open")
                    .arg(&script_path)
                    .spawn()
                    .map_err(|e| e.to_string())?;
            }
            _ => return Err(format!("unknown action kind: {kind}")),
        }
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        match kind {
            "app" => {
                let local = std::env::var("LOCALAPPDATA").map_err(|e| e.to_string())?;
                let path = std::path::PathBuf::from(local).join("Programs").join(target.to_lowercase()).join(format!("{target}.exe"));
                std::process::Command::new(path).creation_flags(0x08000000).spawn().map_err(|_| "无法打开应用，请确认已安装".to_string())?;
            }
            "url" => {
                std::process::Command::new("cmd")
                    .args(["/c", "start", "", target])
                    .creation_flags(0x08000000).spawn().map_err(|e| e.to_string())?;
            }
            "terminal" => {
                std::process::Command::new("cmd")
                    .args(["/c", "start", "", "cmd", "/k", &target])
                    .creation_flags(0x08000000) // 只隐藏外层 cmd，start 仍会新开可见终端
                    .spawn()
                    .map_err(|e| e.to_string())?;
            }
            _ => return Err(format!("unknown action kind: {kind}")),
        }
        return Ok(());
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        let _ = (kind, target);
        Err("当前平台不支持".into())
    }
}

#[tauri::command]
fn secure_storage_status() -> Result<(), String> {
    creds::secure_storage_status()
}

#[tauri::command]
fn list_kimi_accounts() -> Vec<String> {
    creds::kimi_extra_labels()
}

#[tauri::command]
fn add_kimi_account(label: String, key: String) -> Result<(), String> {
    creds::add_kimi_account(label, key)
}

#[tauri::command]
fn remove_kimi_account(label: String) -> Result<(), String> {
    creds::remove_kimi_account(&label)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("failed to build http client");

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(AppState { client })
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            fetch_all,
            fetch_provider,
            credential_status,
            set_deepseek_key,
            set_glm_key,
            has_provider_key,
            set_provider_key,
            fetch_custom,
            open_login,
            list_kimi_accounts,
            secure_storage_status,
            add_kimi_account,
            remove_kimi_account,
        ])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
