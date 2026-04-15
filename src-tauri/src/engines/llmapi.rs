use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct LLMApiEngine {
    pub api_key: String,
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
    content: String,
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
    pub fn new(api_key: String, model: String) -> Self {
        Self { 
            api_key, 
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
        let client = reqwest::Client::new();
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
                    content: system_prompt,
                },
                LLMApiMessage {
                    role: "user".to_string(),
                    content: text.to_string(),
                },
            ],
        };
        
        let response = client
            .post("https://api.siliconflow.cn/v1/chat/completions")
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
