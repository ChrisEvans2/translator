use async_trait::async_trait;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct GoogleEngine {
    pub url: String,
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
struct GoogleResponse {
    data: Option<GoogleData>,
}

#[derive(Debug, Deserialize)]
struct GoogleData {
    translations: Option<Vec<GoogleTranslation>>,
}

#[derive(Debug, Deserialize)]
struct GoogleTranslation {
    translatedText: String,
}

impl GoogleEngine {
    pub fn new(url: String, api_key: String) -> Self {
        Self { url, api_key }
    }
}

#[async_trait::async_trait]
impl super::TranslationEngine for GoogleEngine {
    fn name(&self) -> &'static str {
        "google"
    }

    async fn translate(&self, text: &str, from: &str, to: &str) -> Result<String, super::EngineError> {
        let endpoint = if !self.url.is_empty() {
            format!("{}/translate/v2", self.url)
        } else {
            "https://translation.googleapis.com/language/translate/v2".to_string()
        };

        let client = reqwest::Client::new();
        let mut request = client.post(&endpoint);
        
        if !self.api_key.is_empty() {
            request = request.query(&[("key", &self.api_key)]);
        }
        
        let body = serde_json::json!({
            "q": text,
            "source": from,
            "target": to,
            "format": "text"
        });
        
        let response = request
            .json(&body)
            .send()
            .await
            .map_err(|e| super::EngineError::Network(e.to_string()))?;
            
        let result: GoogleResponse = response
            .json()
            .await
            .map_err(|e| super::EngineError::Parse(e.to_string()))?;
            
        result.data
            .and_then(|d| d.translations)
            .and_then(|t| t.into_iter().next())
            .map(|t| t.translatedText)
            .ok_or_else(|| super::EngineError::Parse("No translation result".to_string()))
    }
}
