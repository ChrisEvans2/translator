use async_trait::async_trait;
use serde::Deserialize;
use md5;

use super::{EngineError, TranslationEngine};

#[derive(Debug, Clone)]
pub struct BaiduEngine {
    pub app_id: String,
    pub secret_key: String,
}

#[derive(Debug, Deserialize)]
struct BaiduResponse {
    trans_result: Option<Vec<BaiduTransResult>>,
    error_code: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct BaiduTransResult {
    dst: String,
}

impl BaiduEngine {
    pub fn new(app_id: String, secret_key: String) -> Self {
        Self { app_id, secret_key }
    }

    fn generate_sign(&self, text: &str, salt: &str) -> String {
        use md5::{Md5, Digest};
        let input = format!("{}{}{}{}", self.app_id, text, salt, self.secret_key);
        let mut hasher = Md5::new();
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[async_trait::async_trait]
impl TranslationEngine for BaiduEngine {
    fn name(&self) -> &'static str {
        "baidu"
    }

    async fn translate(&self, text: &str, from: &str, to: &str) -> Result<String, EngineError> {
        let salt = format!("{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis());
        
        let sign = self.generate_sign(text, &salt);
        
        let client = reqwest::Client::new();
        let params = [
            ("q", text),
            ("from", from),
            ("to", to),
            ("appid", &self.app_id),
            ("salt", &salt),
            ("sign", &sign),
        ];
        
        let response = client
            .post("https://fanyi.baidu.com/api/trans/vip/translate")
            .form(&params)
            .send()
            .await
            .map_err(|e| EngineError::Network(e.to_string()))?;
            
        let result: BaiduResponse = response
            .json()
            .await
            .map_err(|e| EngineError::Parse(e.to_string()))?;
            
        if let Some(error_code) = result.error_code {
            match error_code {
                54003 => return Err(EngineError::RateLimit),
                54001 | 54005 | 58000 => return Err(EngineError::Auth(format!("API error: {}", error_code))),
                _ => return Err(EngineError::Network(format!("API error: {}", error_code))),
            }
        }
        
        result.trans_result
            .and_then(|r| r.into_iter().next())
            .map(|t| t.dst)
            .ok_or_else(|| EngineError::Parse("No translation result".to_string()))
    }
}
