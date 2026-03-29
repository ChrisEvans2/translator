use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub source_lang: String,
    pub target_lang: String,
    pub engine: String,
    pub clipboard_enabled: bool,
    pub theme_color: String,
    pub bg_color: String,
    pub text_color: String,
    pub transparency: u8,
    pub locale: String,
    // Engine configs
    pub baidu_app_id: String,
    pub baidu_secret_key: String,
    pub google_url: String,
    pub google_api_key: String,
    pub siliconflow_api_key: String,
    pub siliconflow_model: String,
    pub ollama_url: String,
    pub ollama_model: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            source_lang: "auto".to_string(),
            target_lang: "zh".to_string(),
            engine: "baidu".to_string(),
            clipboard_enabled: true,
            theme_color: "#426666".to_string(),
            bg_color: "#3f3f3f".to_string(),
            text_color: "#ffffff".to_string(),
            transparency: 50,
            locale: "zh-CN".to_string(),
            // Engine defaults
            baidu_app_id: String::new(),
            baidu_secret_key: String::new(),
            google_url: String::new(),
            google_api_key: String::new(),
            siliconflow_api_key: String::new(),
            siliconflow_model: "deepseek-ai/DeepSeek-V3".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            ollama_model: "llama2".to_string(),
        }
    }
}

fn get_settings_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("translate_app");
    fs::create_dir_all(&config_dir).ok();
    config_dir.join("settings.json")
}

#[tauri::command]
pub fn get_settings() -> Result<Settings, String> {
    let path = get_settings_path();
    if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    } else {
        Ok(Settings::default())
    }
}

#[tauri::command]
pub fn set_settings(settings: Settings) -> Result<(), String> {
    let path = get_settings_path();
    let content = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}
