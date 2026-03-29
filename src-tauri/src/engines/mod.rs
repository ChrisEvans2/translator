use serde::{Deserialize, Serialize};

pub mod baidu;
pub mod google;
pub mod siliconflow;
pub mod ollama;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    pub text: String,
    pub engine: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineError {
    Network(String),
    Auth(String),
    RateLimit,
    Timeout,
    Parse(String),
}

impl From<reqwest::Error> for EngineError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            EngineError::Timeout
        } else {
            EngineError::Network(err.to_string())
        }
    }
}

#[async_trait::async_trait]
pub trait TranslationEngine: Send + Sync {
    fn name(&self) -> &'static str;
    async fn translate(&self, text: &str, from: &str, to: &str) -> Result<String, EngineError>;
}
