//! The legacy plaintext file is removed only after a verified native-store write.
use crate::creds::AppSettings;
use std::path::Path;

pub(crate) trait Store {
    fn read(&self) -> Result<Option<String>, String>;
    fn write(&self, value: &str) -> Result<(), String>;
}

pub(crate) struct NativeStore;
impl Store for NativeStore {
    fn read(&self) -> Result<Option<String>, String> {
        let entry = keyring::Entry::new("dev.aiquota.app", "settings-v1")
            .map_err(|_| "无法初始化系统安全存储".to_string())?;
        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err("无法访问系统安全存储，请解锁钥匙串或检查 Windows 凭据管理器".into()),
        }
    }
    fn write(&self, value: &str) -> Result<(), String> {
        keyring::Entry::new("dev.aiquota.app", "settings-v1")
            .and_then(|entry| entry.set_password(value))
            .map_err(|_| "系统安全存储写入失败；原凭据未被删除".to_string())
    }
}

pub(crate) fn save(store: &impl Store, settings: &AppSettings) -> Result<(), String> {
    let value = serde_json::to_string(settings).map_err(|_| "凭据编码失败")?;
    store.write(&value)?;
    if store.read()?.as_deref() != Some(&value) {
        return Err("系统安全存储校验失败；原凭据未被删除".into());
    }
    Ok(())
}

pub(crate) fn load(path: &Path, store: &impl Store) -> Result<AppSettings, String> {
    let secure = store
        .read()?
        .map(|value| {
            serde_json::from_str::<AppSettings>(&value).map_err(|_| "系统凭据格式无效".to_string())
        })
        .transpose()?;
    let legacy = match std::fs::read_to_string(path) {
        Ok(value) => Some(
            serde_json::from_str::<AppSettings>(&value)
                .map_err(|_| "旧配置格式无效；未修改原文件".to_string())?,
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(_) => return Err("旧配置无法读取；未修改原文件".into()),
    };
    match (secure, legacy) {
        (Some(secure), Some(legacy)) => {
            if secure != legacy {
                return Err("旧配置与系统安全存储存在冲突；两份数据均已保留，请先处理冲突".into());
            }
            std::fs::remove_file(path).map_err(|_| "安全存储已校验，但旧明文文件无法移除")?;
            Ok(secure)
        }
        (None, Some(legacy)) => {
            save(store, &legacy)?;
            std::fs::remove_file(path).map_err(|_| "迁移已校验，但旧明文文件无法移除")?;
            Ok(legacy)
        }
        (Some(secure), None) => Ok(secure),
        (None, None) => Ok(AppSettings::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    struct Memory {
        value: RefCell<Option<String>>,
        fail: bool,
        corrupt: bool,
    }
    impl Store for Memory {
        fn read(&self) -> Result<Option<String>, String> {
            Ok(self.value.borrow().clone())
        }
        fn write(&self, value: &str) -> Result<(), String> {
            if self.fail {
                return Err("unavailable".into());
            }
            *self.value.borrow_mut() = Some(if self.corrupt { "{}" } else { value }.into());
            Ok(())
        }
    }
    #[test]
    fn migration_preserves_legacy_until_verified_and_detects_conflicts() {
        let dir = std::env::temp_dir().join(format!("aiquota-vault-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        let original =
            r#"{"deepseek_key":"test-only-placeholder","glm_key":null,"kimi_accounts":[]}"#;
        for (fail, corrupt) in [(true, false), (false, true), (false, false)] {
            std::fs::write(&path, original).unwrap();
            let store = Memory {
                value: RefCell::new(None),
                fail,
                corrupt,
            };
            let result = load(&path, &store);
            if fail || corrupt {
                assert!(result.is_err());
                assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
            } else {
                assert!(result.is_ok());
                assert!(!path.exists());
                assert!(load(&path, &store).unwrap() == result.unwrap());
                std::fs::write(&path, "{}").unwrap();
                assert!(load(&path, &store).is_err());
                assert!(path.exists());
            }
        }
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
    #[test]
    #[ignore = "writes an isolated test entry to the native credential store"]
    fn native_store_round_trip() {
        let account = format!("self-test-{}", std::process::id());
        let entry = keyring::Entry::new("dev.aiquota.app.test", &account).unwrap();
        entry.set_password("non-secret-test-value").unwrap();
        assert_eq!(entry.get_password().unwrap(), "non-secret-test-value");
        entry.delete_credential().unwrap();
        assert!(matches!(entry.get_password(), Err(keyring::Error::NoEntry)));
    }
}
