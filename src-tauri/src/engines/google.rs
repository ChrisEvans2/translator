use async_trait::async_trait;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct GoogleEngine {
    pub mirror_url: String,
    pub official_url: String,
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
struct GoogleResponse {
    data: Option<GoogleData>,
    error: Option<GoogleError>,
}

#[derive(Debug, Deserialize)]
struct GoogleData {
    translations: Option<Vec<GoogleTranslation>>,
}

#[derive(Debug, Deserialize)]
struct GoogleTranslation {
    #[serde(rename = "translatedText")]
    translated_text: String,
}

#[derive(Debug, Deserialize)]
struct GoogleError {
    code: i32,
    message: String,
    status: Option<String>,
}

impl GoogleEngine {
    pub fn new(mirror_url: String, official_url: String, api_key: String) -> Self {
        Self { mirror_url, official_url, api_key }
    }
    
    async fn translate_with_endpoint(
        &self,
        endpoint: &str,
        text: &str,
        from: &str,
        to: &str,
        use_api_key: bool,
    ) -> Result<String, super::EngineError> {
        let is_unofficial_api = endpoint.contains("/translate_a/");
        
        let client = reqwest::Client::new();
        let mut params = if is_unofficial_api {
            vec![
                ("client", "gtx"),
                ("sl", if from.is_empty() { "auto" } else { from }),
                ("tl", to),
                ("dt", "t"),
                ("q", text),
            ]
        } else {
            let mut p = vec![
                ("q", text),
                ("target", to),
                ("format", "text"),
            ];
            
            if !from.is_empty() {
                p.push(("source", from));
            }
            
            if use_api_key && !self.api_key.is_empty() {
                p.push(("key", &self.api_key));
            }
            
            p
        };
        
        let response = client
            .get(endpoint)
            .query(&params)
            .send()
            .await
            .map_err(|e| super::EngineError::Network(e.to_string()))?;
        
        let status = response.status();
        let response_text = response.text().await
            .map_err(|e| super::EngineError::Network(format!("读取响应失败: {}", e)))?;
        
        eprintln!("Google API Response Status: {}", status);
        eprintln!("Google API Response Body: {}", &response_text[..response_text.len().min(500)]);
        
        if is_unofficial_api {
            Self::parse_unofficial_response(&response_text)
        } else {
            let result: GoogleResponse = serde_json::from_str(&response_text)
                .map_err(|e| super::EngineError::Parse(format!("JSON 解析失败: {}. 响应内容: {}", e, response_text)))?;
            
            if let Some(err) = result.error {
                return Err(match err.code {
                    403 => super::EngineError::Auth(format!("{} ({})", err.message, err.code)),
                    429 => super::EngineError::RateLimit,
                    _ => super::EngineError::Network(format!("{} ({})", err.message, err.code)),
                });
            }
                
            result.data
                .and_then(|d| d.translations)
                .and_then(|t| t.into_iter().next())
                .map(|t| t.translated_text)
                .ok_or_else(|| super::EngineError::Parse("No translation result".to_string()))
        }
    }
    
    fn parse_unofficial_response(response_text: &str) -> Result<String, super::EngineError> {
        let parsed: serde_json::Value = serde_json::from_str(response_text)
            .map_err(|e| super::EngineError::Parse(format!("JSON 解析失败: {}", e)))?;
        
        if let Some(arr) = parsed.as_array() {
            if let Some(first) = arr.get(0) {
                if let Some(inner) = first.as_array() {
                    let mut result = String::new();
                    for item in inner {
                        if let Some(translation) = item.as_array() {
                            if let Some(text) = translation.get(0).and_then(|v| v.as_str()) {
                                result.push_str(text);
                            }
                        }
                    }
                    if !result.is_empty() {
                        return Ok(result);
                    }
                }
            }
        }
        
        Err(super::EngineError::Parse("无法从响应中提取翻译结果".to_string()))
    }
}

#[async_trait::async_trait]
impl super::TranslationEngine for GoogleEngine {
    fn name(&self) -> &'static str {
        "google"
    }

    async fn translate(&self, text: &str, from: &str, to: &str) -> Result<String, super::EngineError> {
        let mirror_available = !self.mirror_url.is_empty();
        let official_available = !self.official_url.is_empty() || !self.api_key.is_empty();
        
        let mirror_result = if mirror_available {
            eprintln!("尝试使用镜像源 URL: {}", self.mirror_url);
            self.translate_with_endpoint(&self.mirror_url, text, from, to, false).await
        } else {
            Err(super::EngineError::Network("未配置镜像源 URL".to_string()))
        };
        
        match mirror_result {
            Ok(translated) => Ok(translated),
            Err(e) if mirror_available && official_available => {
                eprintln!("镜像源失败: {:?}, 尝试使用官方 API", e);
                let endpoint = if !self.official_url.is_empty() {
                    self.official_url.clone()
                } else {
                    "https://translation.googleapis.com/language/translate/v2".to_string()
                };
                self.translate_with_endpoint(&endpoint, text, from, to, true).await
            }
            Err(e) if !mirror_available && official_available => {
                let endpoint = if !self.official_url.is_empty() {
                    self.official_url.clone()
                } else {
                    "https://translation.googleapis.com/language/translate/v2".to_string()
                };
                self.translate_with_endpoint(&endpoint, text, from, to, true).await
            }
            Err(_) => Err(super::EngineError::Auth("未配置镜像源 URL、官方 URL 或 API Key".to_string()))
        }
    }
}
