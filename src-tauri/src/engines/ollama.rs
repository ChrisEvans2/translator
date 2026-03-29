use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct OllamaEngine {
    pub url: String,
    pub model: String,
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: Option<OllamaMessageResponse>,
}

#[derive(Debug, Deserialize)]
struct OllamaMessageResponse {
    content: String,
}

impl OllamaEngine {
    pub fn new(url: String, model: String) -> Self {
        Self {
            url: if url.is_empty() { "http://localhost:11434".to_string() } else { url },
            model: if model.is_empty() { "llama2".to_string() } else { model },
        }
    }
}

#[async_trait::async_trait]
impl super::TranslationEngine for OllamaEngine {
    fn name(&self) -> &'static str {
        "ollama"
    }

    async fn translate(&self, text: &str, _from: &str, to: &str) -> Result<String, super::EngineError> {
        let client = reqwest::Client::new();
        
        let request = OllamaRequest {
            model: self.model.clone(),
            messages: vec![
                OllamaMessage {
                    role: "system".to_string(),
                    content: format!("Translate to {}. Keep LaTeX formulas unchanged.", to),
                },
                OllamaMessage {
                    role: "user".to_string(),
                    content: text.to_string(),
                },
            ],
            stream: false,
        };
        
        let response = client
            .post(format!("{}/api/chat", self.url))
            .json(&request)
            .send()
            .await
            .map_err(|e| super::EngineError::Network(e.to_string()))?;
            
        let result: OllamaResponse = response
            .json()
            .await
            .map_err(|e| super::EngineError::Parse(e.to_string()))?;
            
        result.message
            .map(|m| m.content)
            .ok_or_else(|| super::EngineError::Parse("No translation result".to_string()))
    }
}
