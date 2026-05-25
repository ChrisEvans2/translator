use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    images: Option<Vec<String>>,
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

    async fn translate(&self, text: &str, from: &str, to: &str) -> Result<String, super::EngineError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| super::EngineError::Network(e.to_string()))?;
        let to_name = super::lang_code_to_name(to);
        let from_name = super::lang_code_to_name(from);
        
        let system_prompt = if from == "auto" {
            format!("You are a translator. Translate the following text to {}. Preserve all LaTeX formulas exactly as they appear, including their $ or $$ delimiters. Output only the translated text without any explanation.", to_name)
        } else {
            format!("You are a translator. Translate the following text from {} to {}. Preserve all LaTeX formulas exactly as they appear, including their $ or $$ delimiters. Output only the translated text without any explanation.", from_name, to_name)
        };

        let request = OllamaRequest {
            model: self.model.clone(),
            messages: vec![
                OllamaMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                    images: None,
                },
                OllamaMessage {
                    role: "user".to_string(),
                    content: text.to_string(),
                    images: None,
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

    async fn translate_image(&self, image_base64: &str, prompt: &str, vlm_model: &str) -> Result<String, super::EngineError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| super::EngineError::Network(e.to_string()))?;

        let model = if vlm_model.is_empty() { &self.model } else { vlm_model };

        let request = OllamaRequest {
            model: model.to_string(),
            messages: vec![
                OllamaMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                    images: Some(vec![image_base64.to_string()]),
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
