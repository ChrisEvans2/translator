use serde::{Deserialize, Serialize};

pub mod baidu;
pub mod google;
pub mod llmapi;
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

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::Network(msg) => write!(f, "网络错误: {}", msg),
            EngineError::Auth(msg) => write!(f, "认证失败: {}", msg),
            EngineError::RateLimit => write!(f, "请求频率超限，请稍后重试"),
            EngineError::Timeout => write!(f, "请求超时，请检查网络连接"),
            EngineError::Parse(msg) => {
                if msg == "No translation result" {
                    write!(f, "翻译失败：API 未返回结果。请检查 API 凭证是否已正确配置。")
                } else {
                    write!(f, "解析错误: {}", msg)
                }
            }
        }
    }
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

pub fn lang_code_to_name(code: &str) -> &str {
    match code {
        "zh" => "Chinese",
        "en" => "English",
        "ja" => "Japanese",
        "ko" => "Korean",
        "auto" => "auto-detect",
        other => other,
    }
}

#[async_trait::async_trait]
pub trait TranslationEngine: Send + Sync {
    fn name(&self) -> &'static str;
    async fn translate(&self, text: &str, from: &str, to: &str) -> Result<String, EngineError>;
    async fn translate_image(&self, _image_base64: &str, _prompt: &str, _vlm_model: &str) -> Result<String, EngineError> {
        Err(EngineError::Network("Image translation not supported for this engine".to_string()))
    }
}
