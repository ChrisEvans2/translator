mod settings;
mod clipboard;
mod engines;

use clipboard::spawn_clipboard_monitor;
use settings::{get_settings, set_settings};
use engines::{TranslationResult, TranslationEngine, baidu::BaiduEngine, google::GoogleEngine, llmapi::LLMApiEngine, ollama::OllamaEngine};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
async fn translate(text: String, from: String, to: String, engine: String) -> Result<TranslationResult, String> {
    let settings = get_settings()?;
    
    // 验证凭证是否已配置
    let credential_error = match engine.as_str() {
        "baidu" if settings.baidu_app_id.is_empty() || settings.baidu_secret_key.is_empty() => {
            Some("百度翻译未配置。请在设置中填入 APP ID 和密钥。")
        }
        "llmapi" if settings.llmapi_api_key.is_empty() => {
            Some("大模型API Key 未配置。请在设置中填入 API Key。")
        }
        "google" if settings.google_mirror_url.is_empty() && settings.google_official_url.is_empty() && settings.google_api_key.is_empty() => {
            Some("Google 翻译配置未完成。请在设置中至少填入镜像源 URL、官方 URL 或 API Key 之一。")
        }
        "ollama" if settings.ollama_url.is_empty() => {
            Some("Ollama URL 未配置。请在设置中填入服务地址。")
        }
        _ => None,
    };
    
    if let Some(error_msg) = credential_error {
        return Ok(TranslationResult {
            text: String::new(),
            engine: engine.clone(),
            error: Some(error_msg.to_string()),
        });
    }
    
    let translator: Box<dyn TranslationEngine> = match engine.as_str() {
        "baidu" => Box::new(BaiduEngine::new(settings.baidu_app_id, settings.baidu_secret_key)),
        "google" => Box::new(GoogleEngine::new(settings.google_mirror_url, settings.google_official_url, settings.google_api_key)),
        "llmapi" => Box::new(LLMApiEngine::new(settings.llmapi_api_key, settings.llmapi_model)),
        "ollama" => Box::new(OllamaEngine::new(settings.ollama_url, settings.ollama_model)),
        _ => return Err(format!("Unknown engine: {}", engine)),
    };
    
    let from_lang = &from;
    
    match translator.translate(&text, from_lang, &to).await {
        Ok(translated) => Ok(TranslationResult {
            text: translated,
            engine: engine.clone(),
            error: None,
        }),
        Err(e) => Ok(TranslationResult {
            text: String::new(),
            engine: engine.clone(),
            error: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
async fn translate_image(image_base64: String, to: String, engine: String) -> Result<TranslationResult, String> {
    let settings = get_settings()?;
    
    if !settings.image_translation_enabled {
        return Ok(TranslationResult {
            text: String::new(),
            engine: engine.clone(),
            error: Some("图片翻译未开启。请在设置中开启。".to_string()),
        });
    }

    let to_name = engines::lang_code_to_name(&to);
    let prompt = format!("Translate the text in this image to {}. Output only the translated text without any explanation.", to_name);

    let translator: Box<dyn TranslationEngine> = match engine.as_str() {
        "llmapi" => Box::new(LLMApiEngine::new(settings.llmapi_api_key, settings.llmapi_model)),
        "ollama" => Box::new(OllamaEngine::new(settings.ollama_url, settings.ollama_model)),
        _ => return Ok(TranslationResult {
            text: String::new(),
            engine: engine.clone(),
            error: Some("图片翻译仅支持大模型API和Ollama引擎。".to_string()),
        }),
    };

    let vlm_model = match engine.as_str() {
        "llmapi" => &settings.llmapi_vlm_model,
        "ollama" => &settings.ollama_vlm_model,
        _ => "",
    };

    match translator.translate_image(&image_base64, &prompt, vlm_model).await {
        Ok(translated) => Ok(TranslationResult {
            text: translated,
            engine: engine.clone(),
            error: None,
        }),
        Err(e) => Ok(TranslationResult {
            text: String::new(),
            engine: engine.clone(),
            error: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
fn start_dragging(window: tauri::WebviewWindow) -> Result<(), String> {
    window
        .start_dragging()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn minimize_window(window: tauri::WebviewWindow) -> Result<(), String> {
    window
        .minimize()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn close_window(window: tauri::WebviewWindow) -> Result<(), String> {
    window
        .close()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_window_always_on_top(window: tauri::WebviewWindow, always_on_top: bool) -> Result<(), String> {
    window
        .set_always_on_top(always_on_top)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.unminimize().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window("settings") {
        existing.show().map_err(|e| e.to_string())?;
        existing.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let width = (760.0_f64 * 2.0 / 3.0).round();
    let height = (560.0_f64 * 2.0 / 3.0).round();
    let min_width = (520.0_f64 * 2.0 / 3.0).round();
    let min_height = (420.0_f64 * 2.0 / 3.0).round();

    let window = WebviewWindowBuilder::new(
        &app,
        "settings",
        WebviewUrl::App("settings.html".into()),
    )
    .title("设置")
    .decorations(false)
    .transparent(false)
    .resizable(true)
    .center()
    .inner_size(width, height)
    .min_inner_size(min_width, min_height)
    .build()
    .map_err(|e| e.to_string())?;

    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            spawn_clipboard_monitor(app.handle().clone());
            
            if let Some(window) = app.get_webview_window("main") {
                let settings = get_settings().unwrap_or_default();
                let _ = window.set_always_on_top(settings.always_on_top);
            }
            
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            translate,
            translate_image,
            open_settings_window,
            start_dragging,
            minimize_window,
            close_window,
            set_window_always_on_top,
            show_main_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
