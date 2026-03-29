mod settings;
mod clipboard;
mod engines;

use clipboard::spawn_clipboard_monitor;
use settings::{get_settings, set_settings};
use engines::TranslationResult;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
async fn translate(text: String, from: String, to: String, engine: String) -> Result<TranslationResult, String> {
    Ok(TranslationResult {
        text: format!("[{}] {}", engine, text),
        engine,
        error: None,
    })
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
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            translate,
            open_settings_window,
            start_dragging,
            minimize_window,
            close_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
