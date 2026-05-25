use serde::Deserialize;
use std::time::Duration;

use super::{EngineError, TranslationEngine};

#[derive(Debug, Clone)]
pub struct BaiduEngine {
    pub app_id: String,
    pub secret_key: String,
}

#[derive(Debug, Deserialize)]
struct BaiduResponse {
    trans_result: Option<Vec<BaiduTransResult>>,
    error_code: Option<String>,
    error_msg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BaiduTransResult {
    src: String,
    dst: String,
}

impl BaiduEngine {
    pub fn new(app_id: String, secret_key: String) -> Self {
        Self { app_id, secret_key }
    }

    fn generate_sign(&self, text: &str, salt: &str) -> String {
        let app_id = self.app_id.trim();
        let secret_key = self.secret_key.trim();
        use md5::{Md5, Digest};
        let input = format!("{}{}{}{}", app_id, text, salt, secret_key);
        eprintln!("[Baidu DEBUG] app_id: '{}'", app_id);
        eprintln!("[Baidu DEBUG] secret_key: '{}'", secret_key);
        eprintln!("[Baidu DEBUG] text (q): '{}'", text);
        eprintln!("[Baidu DEBUG] salt: '{}'", salt);
        eprintln!("[Baidu DEBUG] sign input: '{}'", input);
        let mut hasher = Md5::new();
        hasher.update(input.as_bytes());
        let sign = hex::encode(hasher.finalize());
        eprintln!("[Baidu DEBUG] sign: '{}'", sign);
        sign
    }
}

#[async_trait::async_trait]
impl TranslationEngine for BaiduEngine {
    fn name(&self) -> &'static str {
        "baidu"
    }

    async fn translate(&self, text: &str, from: &str, to: &str) -> Result<String, EngineError> {
        let salt = rand::random::<u16>().to_string();
        let sign = self.generate_sign(text, &salt);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| EngineError::Network(e.to_string()))?;
        let response = client
            .get("https://api.fanyi.baidu.com/api/trans/vip/translate")
            .query(&[
                ("q", text),
                ("from", from),
                ("to", to),
                ("appid", &self.app_id),
                ("salt", &salt),
                ("sign", &sign),
            ])
            .send()
            .await
            .map_err(|e| {
                let detail = e.to_string();
                if e.is_timeout() {
                    EngineError::Network(format!("请求超时: {}", detail))
                } else if e.is_connect() {
                    EngineError::Network(format!("连接失败: {} (请检查网络/代理设置)", detail))
                } else if e.is_request() {
                    EngineError::Network(format!("请求错误: {}", detail))
                } else {
                    EngineError::Network(format!("请求失败: {}", detail))
                }
            })?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| EngineError::Parse(format!("Failed to read response body: {}", e)))?;

        if body.is_empty() {
            return Err(EngineError::Network(format!("百度翻译 API 返回空响应 (HTTP {})", status)));
        }

        let result: BaiduResponse = serde_json::from_str(&body)
            .map_err(|e| EngineError::Parse(format!("error decoding response body: {}\nRaw: {}", e, body.chars().take(200).collect::<String>())))?;
            
        if let Some(error_code) = result.error_code {
            match error_code.as_str() {
                "54003" => return Err(EngineError::RateLimit),
                "54001" | "54005" | "58000" => return Err(EngineError::Auth(format!("签名错误: {}", error_code))),
                _ => return Err(EngineError::Network(format!("API error: {}{}", error_code, result.error_msg.map(|m| format!(" - {}", m)).unwrap_or_default()))),
            }
        }
        
        result.trans_result
            .and_then(|r| r.into_iter().next())
            .map(|t| t.dst)
            .ok_or_else(|| EngineError::Parse("No translation result".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baidu_sign_official_example() {
        let engine = BaiduEngine::new("2015063000000001".to_string(), "12345678".to_string());
        let sign = engine.generate_sign("apple", "1435660288");
        assert_eq!(sign, "f89f9594663708c1605f3d736d01d2d4");
    }
}
