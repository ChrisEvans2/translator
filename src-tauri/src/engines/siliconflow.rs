use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct SiliconFlowEngine {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Serialize)]
struct SiliconFlowRequest {
    model: String,
    messages: Vec<SiliconFlowMessage>,
}

#[derive(Debug, Serialize)]
struct SiliconFlowMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct SiliconFlowResponse {
    choices: Option<Vec<SiliconFlowChoice>>,
}

#[derive(Debug, Deserialize)]
struct SiliconFlowChoice {
    message: Option<SiliconFlowMessageResponse>,
}

#[derive(Debug, Deserialize)]
struct SiliconFlowMessageResponse {
    content: String,
}

impl SiliconFlowEngine {
    pub fn new(api_key: String, model: String) -> Self {
        Self { 
            api_key, 
            model: if model.is_empty() { "deepseek-ai/DeepSeek-V3".to_string() } else { model } 
        }
    }
}

#[async_trait::async_trait]
impl super::TranslationEngine for SiliconFlowEngine {
    fn name(&self) -> &'static str {
        "siliconflow"
    }

    async fn translate(&self, text: &str, _from: &str, to: &str) -> Result<String, super::EngineError> {
        let client = reqwest::Client::new();
        
        let request = SiliconFlowRequest {
            model: self.model.clone(),
            messages: vec![
                SiliconFlowMessage {
                    role: "system".to_string(),
                    content: format!("Translate to {}. Keep LaTeX formulas unchanged.", to),
                },
                SiliconFlowMessage {
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
            
        let result: SiliconFlowResponse = response
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
