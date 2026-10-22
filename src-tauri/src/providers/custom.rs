//! User-configured GET/JSON quota adapters. Secrets never enter renderer storage.
use super::{ProviderResult, QuotaWindow};
use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Deserialize)]
pub struct CustomConfig {
    pub id: String,
    pub endpoint: String,
    pub auth: String,
    pub header: String,
    pub metric: String,
    pub value_path: String,
    pub total_path: String,
    pub reset_path: String,
    pub unit: String,
}

fn validate(config: &CustomConfig) -> Result<reqwest::Url, &'static str> {
    if !config.id.starts_with("custom-")
        || config.id.len() > 80
        || !config
            .id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-')
    {
        return Err("自定义供应商标识无效");
    }
    if config.endpoint.len() > 2048 {
        return Err("查询地址过长");
    }
    let url = reqwest::Url::parse(&config.endpoint).map_err(|_| "请输入完整的额度查询 URL")?;
    let loopback = url.host_str().is_some_and(|host| {
        host == "localhost"
            || host
                .trim_matches(['[', ']'])
                .parse::<std::net::IpAddr>()
                .is_ok_and(|ip| ip.is_loopback())
    });
    if url.scheme() != "https" && !(url.scheme() == "http" && loopback) {
        return Err("查询地址必须使用 HTTPS；本机 localhost 可使用 HTTP");
    }
    if url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err("查询地址不能包含用户名、密码或片段，请在认证栏填写 Key");
    }
    if !matches!(config.auth.as_str(), "none" | "bearer" | "header") {
        return Err("认证方式无效");
    }
    if config.auth == "header" {
        let name = reqwest::header::HeaderName::from_bytes(config.header.as_bytes())
            .map_err(|_| "认证 Header 名称无效")?;
        if !matches!(
            name.as_str(),
            "x-api-key" | "api-key" | "x-goog-api-key" | "authorization"
        ) {
            return Err("认证 Header 支持 x-api-key、api-key、x-goog-api-key 或 Authorization");
        }
    }
    if !matches!(config.metric.as_str(), "balance" | "percent" | "usage") {
        return Err("额度显示方式无效");
    }
    for path in [&config.value_path, &config.total_path, &config.reset_path] {
        if path.len() > 200
            || (!path.is_empty()
                && path.split('.').any(|s| {
                    s.is_empty()
                        || !s
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
                }))
        {
            return Err("字段路径请使用 data.balance 或 data.items.0.used 这样的点号格式");
        }
    }
    if config.value_path.is_empty() || (config.metric == "usage" && config.total_path.is_empty()) {
        return Err("请填写额度字段路径");
    }
    if config.unit.chars().count() > 16 {
        return Err("单位最多 16 个字符");
    }
    Ok(url)
}

fn field<'a>(json: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() {
        return None;
    }
    path.split('.').try_fold(json, |v, part| {
        if v.is_array() {
            v.get(part.parse::<usize>().ok()?)
        } else {
            v.get(part)
        }
    })
}
fn number(json: &Value, path: &str) -> Result<f64, String> {
    field(json, path)
        .and_then(|v| v.as_f64().or_else(|| v.as_str()?.parse().ok()))
        .filter(|n| n.is_finite())
        .ok_or_else(|| format!("字段 {path} 不存在或不是有效数字，请检查字段映射"))
}
fn parse(config: &CustomConfig, json: &Value) -> Result<ProviderResult, String> {
    let value = number(json, &config.value_path)?;
    let reset = if config.reset_path.is_empty() {
        None
    } else {
        let v = field(json, &config.reset_path).ok_or("未找到重置时间字段，请检查字段映射")?;
        if !v.is_null() && !v.is_string() && !v.is_number() {
            return Err("重置时间必须是 ISO 时间或时间戳".into());
        }
        Some(v.clone())
    };
    let mut extras = vec![];
    let percent = match config.metric.as_str() {
        "balance" => {
            extras.push((
                "可用余额".into(),
                format!(
                    "{value:.2}{}",
                    if config.unit.is_empty() {
                        String::new()
                    } else {
                        format!(" {}", config.unit)
                    }
                ),
            ));
            None
        }
        "percent" => {
            if !(0.0..=100.0).contains(&value) {
                return Err("已用百分比必须在 0–100 之间；比例值需由接口转换为百分比".into());
            }
            Some(value)
        }
        "usage" => {
            let total = number(json, &config.total_path)?;
            if total <= 0.0 || value < 0.0 {
                return Err("总额度必须大于 0，已用量不能为负数".into());
            }
            let percent = value / total * 100.0;
            if !percent.is_finite() {
                return Err("额度数值超出有效范围".into());
            }
            extras.push((
                "已用 / 总量".into(),
                format!("{value} / {total} {}", config.unit),
            ));
            Some(percent)
        }
        _ => return Err("额度显示方式无效".into()),
    };
    let windows = if percent.is_some() || reset.is_some() {
        vec![QuotaWindow {
            label: "当前额度".into(),
            used_percent: percent,
            resets_at: reset,
            status: if percent.is_some_and(|n| n >= 100.0) {
                "limited"
            } else {
                "ok"
            }
            .into(),
        }]
    } else {
        vec![]
    };
    Ok(ProviderResult::ok(&config.id, windows, extras))
}

async fn query(config: &CustomConfig, key: Option<String>) -> Result<ProviderResult, String> {
    let url = validate(config)?;
    // A redirected endpoint must never forward the user's credential elsewhere.
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|_| "无法创建查询客户端")?;
    let mut request = client.get(url).header("Accept", "application/json");
    if config.auth != "none" {
        let key = key
            .filter(|s| !s.is_empty())
            .ok_or("请先配置此供应商的 API Key")?;
        request = if config.auth == "bearer" {
            request.bearer_auth(key)
        } else {
            request.header(&config.header, key)
        };
    }
    // Deliberately omit reqwest's URL-bearing errors and untrusted response bodies.
    let mut response = request
        .send()
        .await
        .map_err(|_| "无法连接额度接口，请检查地址、网络或证书")?;
    if response.status().is_redirection() {
        return Err("额度接口返回重定向，请直接填写最终 HTTPS 地址".into());
    }
    if !response.status().is_success() {
        return Err(match response.status().as_u16() {
            401 | 403 => "认证失败，请检查 Key、认证方式和接口权限".into(),
            code => format!("额度接口请求失败 (HTTP {code})"),
        });
    }
    const MAX_BODY: usize = 1_048_576;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "额度响应读取失败")? {
        if bytes.len() + chunk.len() > MAX_BODY {
            return Err("额度响应超过 1 MB，请使用精简的查询接口".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    let json: Value = serde_json::from_slice(&bytes).map_err(|_| "额度接口未返回有效 JSON")?;
    parse(config, &json)
}

pub async fn fetch(config: &CustomConfig) -> ProviderResult {
    let result = match validate(config) {
        Err(error) => Err(error.into()),
        Ok(_) => match if config.auth == "none" {
            Ok(None)
        } else {
            crate::creds::provider_key(&config.id)
        } {
            Ok(key) => query(config, key).await,
            Err(error) => Err(error),
        },
    };
    result.unwrap_or_else(|error| ProviderResult::error(&config.id, error))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> CustomConfig {
        serde_json::from_value(serde_json::json!({"id":"custom-test","endpoint":"https://example.com/quota","auth":"none","header":"x-api-key","metric":"usage","value_path":"data.items.0.used","total_path":"data.total","reset_path":"data.reset","unit":"tokens"})).unwrap()
    }
    #[test]
    fn maps_nested_arrays_numeric_strings_and_resets_without_silent_zeroes() {
        let config = config();
        let json = serde_json::json!({"data":{"items":[{"used":"25"}],"total":100,"reset":"2026-11-02T08:53:16Z"}});
        let result = parse(&config, &json).unwrap();
        assert_eq!(result.windows[0].used_percent, Some(25.0));
        assert_eq!(
            result.windows[0].resets_at,
            Some(json["data"]["reset"].clone())
        );
        assert!(parse(&config, &serde_json::json!({})).is_err());
        let mut invalid = json;
        invalid["data"]["total"] = 0.into();
        assert!(parse(&config, &invalid).is_err());
    }
    #[test]
    fn supports_balance_and_percent_but_rejects_invalid_mapping_and_urls() {
        let mut config = config();
        config.reset_path.clear();
        config.value_path = "balance".into();
        config.metric = "balance".into();
        assert_eq!(
            parse(&config, &serde_json::json!({"balance":"12.34"}))
                .unwrap()
                .extras[0]
                .1,
            "12.34 tokens"
        );
        config.metric = "percent".into();
        assert_eq!(
            parse(&config, &serde_json::json!({"balance":100}))
                .unwrap()
                .windows[0]
                .status,
            "limited"
        );
        assert!(parse(&config, &serde_json::json!({"balance":101})).is_err());
        for url in [
            "http://example.com",
            "file:///tmp/quota",
            "https://user:password@example.com",
            "https://example.com/#key",
        ] {
            config.endpoint = url.into();
            assert!(validate(&config).is_err(), "{url}");
        }
        for url in [
            "https://example.com/quota",
            "http://127.0.0.1:8080",
            "http://[::1]:8080",
        ] {
            config.endpoint = url.into();
            assert!(validate(&config).is_ok(), "{url}");
        }
        config.value_path = "data..balance".into();
        assert!(validate(&config).is_err());
        config.value_path = "data.balance".into();
        config.auth = "header".into();
        config.header = "Host".into();
        assert!(validate(&config).is_err());
    }
    #[tokio::test]
    async fn queries_json_with_auth_and_refuses_redirects() {
        use std::io::{Read, Write};
        for redirect in [false, true] {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let mut config = config();
            config.endpoint = format!("http://{}/quota", listener.local_addr().unwrap());
            config.auth = "bearer".into();
            config.metric = "balance".into();
            config.value_path = "balance".into();
            config.reset_path.clear();
            let server = std::thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0u8; 4096];
                let len = stream.read(&mut request).unwrap();
                let request = String::from_utf8_lossy(&request[..len]).to_lowercase();
                assert!(request.contains("authorization: bearer test-placeholder"));
                if redirect {
                    stream.write_all(b"HTTP/1.1 302 Found\r\nLocation: https://example.com/secret\r\nContent-Length: 0\r\n\r\n").unwrap();
                } else {
                    stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 14\r\n\r\n{\"balance\":42}").unwrap();
                }
            });
            let result = query(&config, Some("test-placeholder".into())).await;
            server.join().unwrap();
            if redirect {
                assert!(result.err().unwrap().contains("重定向"));
            } else {
                assert_eq!(result.unwrap().extras[0].1, "42.00 tokens");
            }
        }
    }
}
