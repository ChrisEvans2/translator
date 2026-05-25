use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct LLMApiEngine {
    pub api_key: String,
    pub url: String,
    pub model: String,
}

#[derive(Debug, Serialize)]
struct LLMApiRequest {
    model: String,
    messages: Vec<LLMApiMessage>,
}

#[derive(Debug, Serialize)]
struct LLMApiMessage {
    role: String,
    content: serde_json::Value,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum LLMApiContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: LLMApiImageUrl },
}

#[derive(Debug, Serialize)]
struct LLMApiImageUrl {
    url: String,
}

#[derive(Debug, Deserialize)]
struct LLMApiResponse {
    choices: Option<Vec<LLMApiChoice>>,
}

#[derive(Debug, Deserialize)]
struct LLMApiChoice {
    message: Option<LLMApiMessageResponse>,
}

#[derive(Debug, Deserialize)]
struct LLMApiMessageResponse {
    content: String,
}

impl LLMApiEngine {
    pub fn new(api_key: String, url: String, model: String) -> Self {
        Self { 
            api_key,
            url: if url.is_empty() { "https://api.siliconflow.cn/v1/chat/completions".to_string() } else { url },
            model: if model.is_empty() { "deepseek-ai/DeepSeek-V3".to_string() } else { model } 
        }
    }
}

#[async_trait::async_trait]
impl super::TranslationEngine for LLMApiEngine {
    fn name(&self) -> &'static str {
        "llmapi"
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

        let request = LLMApiRequest {
            model: self.model.clone(),
            messages: vec![
                LLMApiMessage {
                    role: "system".to_string(),
                    content: serde_json::json!(system_prompt),
                },
                LLMApiMessage {
                    role: "user".to_string(),
                    content: serde_json::json!(text),
                },
            ],
        };
        
        let response = client
            .post(&self.url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| super::EngineError::Network(e.to_string()))?;
            
        let result: LLMApiResponse = response
            .json()
            .await
            .map_err(|e| super::EngineError::Parse(e.to_string()))?;
            
        result.choices
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.message)
            .map(|m| m.content)
            .ok_or_else(|| super::EngineError::Parse("No translation result".to_string()))
    }

    async fn translate_image(&self, image_base64: &str, prompt: &str, vlm_model: &str) -> Result<String, super::EngineError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| super::EngineError::Network(e.to_string()))?;

        let model = if vlm_model.is_empty() { &self.model } else { vlm_model };

        let content = vec![
            LLMApiContentPart::Text { text: prompt.to_string() },
            LLMApiContentPart::ImageUrl {
                image_url: LLMApiImageUrl {
                    url: format!("data:image/png;base64,{}", image_base64),
                },
            },
        ];

        let request = LLMApiRequest {
            model: model.to_string(),
            messages: vec![
                LLMApiMessage {
                    role: "user".to_string(),
                    content: serde_json::to_value(content).map_err(|e| super::EngineError::Parse(e.to_string()))?,
                },
            ],
        };

        let response = client
            .post(&self.url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| super::EngineError::Network(e.to_string()))?;

        let result: LLMApiResponse = response
            .json()
            .await
            .map_err(|e| super::EngineError::Parse(e.to_string()))?;

        result.choices
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.message)
            .map(|m| m.content)
            .ok_or_else(|| super::EngineError::Parse("No translation result".to_string()))
    }
}
