use serde::Serialize;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Clone, Serialize)]
struct ClipboardChangedPayload {
    text: String,
}

pub fn spawn_clipboard_monitor(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let poll_interval = Duration::from_millis(100);
        let debounce = Duration::from_millis(300);

        let mut last_emitted = String::new();
        let mut pending_text: Option<String> = None;
        let mut pending_since: Option<Instant> = None;

        loop {
            let app_handle = app.clone();
            let current_text = tauri::async_runtime::spawn_blocking(move || {
                app_handle.clipboard().read_text().ok()
            })
            .await
            .ok()
            .flatten();

            if let Some(text) = current_text {
                if text.is_empty() {
                    pending_text = None;
                    pending_since = None;
                } else if pending_text.as_deref() != Some(text.as_str()) {
                    pending_text = Some(text);
                    pending_since = Some(Instant::now());
                }
            } else {
                // 剪切板内容为空或为图像等非文本内容，清除待处理状态
                pending_text = None;
                pending_since = None;
            }

            if let Some(candidate) = &pending_text {
                if let Some(since) = pending_since {
                    if since.elapsed() >= debounce && *candidate != last_emitted {
                        let payload = ClipboardChangedPayload {
                            text: candidate.clone(),
                        };

                        if app.emit("clipboard-changed", payload).is_ok() {
                            last_emitted = candidate.clone();
                        }

                        pending_text = None;
                        pending_since = None;
                    }
                }
            }

            tokio::time::sleep(poll_interval).await;
        }
    });
}
