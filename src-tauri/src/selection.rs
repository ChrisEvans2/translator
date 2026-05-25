use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

#[derive(Clone, Serialize)]
pub struct SelectionPayload {
    pub text: String,
    pub x: f64,
    pub y: f64,
}

struct SelectionState {
    last_trigger: Instant,
    last_text: String,
    original_clipboard: Option<String>,
}

static STATE: Mutex<Option<SelectionState>> = Mutex::new(None);
static LAST_PAYLOAD: Mutex<Option<SelectionPayload>> = Mutex::new(None);
static MOUSE_MONITOR_STARTED: AtomicBool = AtomicBool::new(false);
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

fn set_last_payload(payload: SelectionPayload) {
    let mut last_payload = LAST_PAYLOAD.lock().unwrap();
    *last_payload = Some(payload);
}

pub fn get_last_payload() -> Option<SelectionPayload> {
    LAST_PAYLOAD.lock().unwrap().clone()
}

pub fn ensure_popup_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window("selection-popup") {
        return Ok(window);
    }

    WebviewWindowBuilder::new(
        app,
        "selection-popup",
        WebviewUrl::App("selection-popup.html".into()),
    )
    .title("")
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .visible(false)
    .focused(false)
    .inner_size(360.0, 120.0)
    .build()
    .map_err(|e| e.to_string())
}

fn get_mouse_position() -> (f64, f64) {
    use enigo::Mouse;
    let enigo = enigo::Enigo::new(&enigo::Settings::default()).unwrap();
    if let Ok((x, y)) = enigo.location() {
        (x as f64, y as f64)
    } else {
        (0.0, 0.0)
    }
}

fn save_clipboard(app: &AppHandle) {
    if let Ok(text) = app.clipboard().read_text() {
        let mut state = STATE.lock().unwrap();
        if let Some(s) = state.as_mut() {
            s.original_clipboard = Some(text);
        }
    }
}

fn restore_clipboard(app: &AppHandle) {
    let mut state = STATE.lock().unwrap();
    if let Some(s) = state.as_mut() {
        if let Some(ref text) = s.original_clipboard {
            let _ = app.clipboard().write_text(text.clone());
            s.original_clipboard = None;
        }
    }
}

fn simulate_copy() {
    use std::mem::size_of;
    use winapi::um::winuser::{SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, VK_CONTROL};

    unsafe {
        let mut inputs: [INPUT; 4] = std::mem::zeroed();

        // Ctrl down
        inputs[0].type_ = INPUT_KEYBOARD;
        *inputs[0].u.ki_mut() = std::mem::zeroed();
        inputs[0].u.ki_mut().wVk = VK_CONTROL as u16;

        // C down
        inputs[1].type_ = INPUT_KEYBOARD;
        *inputs[1].u.ki_mut() = std::mem::zeroed();
        inputs[1].u.ki_mut().wVk = 0x43; // 'C' key

        // C up
        inputs[2].type_ = INPUT_KEYBOARD;
        *inputs[2].u.ki_mut() = std::mem::zeroed();
        inputs[2].u.ki_mut().wVk = 0x43;
        inputs[2].u.ki_mut().dwFlags = KEYEVENTF_KEYUP;

        // Ctrl up
        inputs[3].type_ = INPUT_KEYBOARD;
        *inputs[3].u.ki_mut() = std::mem::zeroed();
        inputs[3].u.ki_mut().wVk = VK_CONTROL as u16;
        inputs[3].u.ki_mut().dwFlags = KEYEVENTF_KEYUP;

        SendInput(4, inputs.as_mut_ptr(), size_of::<INPUT>() as i32);
    }

    std::thread::sleep(Duration::from_millis(200));
}

fn read_clipboard(app: &AppHandle) -> Option<String> {
    app.clipboard().read_text().ok().filter(|t| !t.is_empty())
}

fn should_trigger(text: &str) -> bool {
    let mut state = STATE.lock().unwrap();
    let state = state.get_or_insert(SelectionState {
        last_trigger: Instant::now(),
        last_text: String::new(),
        original_clipboard: None,
    });

    let now = Instant::now();
    let elapsed = now.duration_since(state.last_trigger);

    if elapsed < Duration::from_millis(1000) {
        return false;
    }

    if text == state.last_text {
        return false;
    }

    if text.len() < 2 || text.len() > 5000 {
        return false;
    }

    state.last_trigger = now;
    state.last_text = text.to_string();
    true
}

pub fn trigger_selection(app: AppHandle) {
    let settings = crate::settings::get_settings().unwrap_or_default();

    if !settings.selection_enabled {
        return;
    }

    if settings.selection_restore_clipboard {
        save_clipboard(&app);
    }

    simulate_copy();

    let text = read_clipboard(&app);

    if settings.selection_restore_clipboard {
        restore_clipboard(&app);
    }

    let Some(text) = text else { return };

    if !should_trigger(&text) {
        return;
    }

    let (x, y) = get_mouse_position();

    let payload = SelectionPayload {
        text: text.clone(),
        x,
        y,
    };

    set_last_payload(payload.clone());

    let Ok(window) = ensure_popup_window(&app) else {
        return;
    };

    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
        x: x as i32 + 10,
        y: y as i32 + 10,
    }));
    let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
        width: 360.0,
        height: 120.0,
    }));
    let _ = window.show();
    let _ = window.emit("selection-text-ready", &payload);
}

pub fn apply_settings(
    app: AppHandle,
    settings: &crate::settings::Settings,
) -> Result<(), String> {
    if !settings.selection_enabled {
        let _ = unregister_global_shortcut(app.clone());
        if let Some(window) = app.get_webview_window("selection-popup") {
            let _ = window.hide();
        }
        return Ok(());
    }

    ensure_popup_window(&app)?;

    if settings.selection_auto_mode {
        let _ = unregister_global_shortcut(app.clone());
        spawn_mouse_monitor(app);
        return Ok(());
    }

    let _ = unregister_global_shortcut(app.clone());
    if !settings.selection_hotkey.trim().is_empty() {
        register_global_shortcut(app, &settings.selection_hotkey)?;
    }

    Ok(())
}

pub fn register_global_shortcut(app: AppHandle, hotkey: &str) -> Result<(), String> {
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

    let parts: Vec<&str> = hotkey.split('+').collect();
    if parts.len() < 2 {
        return Err("Invalid hotkey format".to_string());
    }

    let mut modifiers = Modifiers::empty();
    let mut code = None;

    for part in &parts {
        match part.trim().to_lowercase().as_str() {
            "alt" => modifiers |= Modifiers::ALT,
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "shift" => modifiers |= Modifiers::SHIFT,
            "super" | "win" | "meta" => modifiers |= Modifiers::SUPER,
            "q" => code = Some(Code::KeyQ),
            "w" => code = Some(Code::KeyW),
            "e" => code = Some(Code::KeyE),
            "r" => code = Some(Code::KeyR),
            "t" => code = Some(Code::KeyT),
            "y" => code = Some(Code::KeyY),
            "a" => code = Some(Code::KeyA),
            "s" => code = Some(Code::KeyS),
            "d" => code = Some(Code::KeyD),
            "f" => code = Some(Code::KeyF),
            "g" => code = Some(Code::KeyG),
            "z" => code = Some(Code::KeyZ),
            "x" => code = Some(Code::KeyX),
            "c" => code = Some(Code::KeyC),
            "v" => code = Some(Code::KeyV),
            "1" => code = Some(Code::Digit1),
            "2" => code = Some(Code::Digit2),
            "3" => code = Some(Code::Digit3),
            "4" => code = Some(Code::Digit4),
            "5" => code = Some(Code::Digit5),
            "6" => code = Some(Code::Digit6),
            "7" => code = Some(Code::Digit7),
            "8" => code = Some(Code::Digit8),
            "9" => code = Some(Code::Digit9),
            "0" => code = Some(Code::Digit0),
            "space" => code = Some(Code::Space),
            "tab" => code = Some(Code::Tab),
            "f1" => code = Some(Code::F1),
            "f2" => code = Some(Code::F2),
            "f3" => code = Some(Code::F3),
            "f4" => code = Some(Code::F4),
            "f5" => code = Some(Code::F5),
            "f6" => code = Some(Code::F6),
            "f7" => code = Some(Code::F7),
            "f8" => code = Some(Code::F8),
            "f9" => code = Some(Code::F9),
            "f10" => code = Some(Code::F10),
            "f11" => code = Some(Code::F11),
            "f12" => code = Some(Code::F12),
            _ => {}
        }
    }

    let code = code.ok_or("No key found in hotkey")?;
    let shortcut = Shortcut::new(Some(modifiers), code);

    let app_clone = app.clone();
    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                trigger_selection(app_clone.clone());
            }
        })
        .map_err(|e: tauri_plugin_global_shortcut::Error| e.to_string())?;

    Ok(())
}

pub fn unregister_global_shortcut(app: AppHandle) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e: tauri_plugin_global_shortcut::Error| e.to_string())?;
    Ok(())
}

pub fn spawn_mouse_monitor(app: AppHandle) {
    if MOUSE_MONITOR_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }

    let _ = APP_HANDLE.set(app);

    std::thread::spawn(move || {
        use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
        use winapi::um::winuser::{
            CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
        };
        use winapi::um::winuser::{WH_MOUSE_LL, WM_LBUTTONUP};

        unsafe extern "system" fn hook_proc(
            n_code: i32,
            w_param: WPARAM,
            l_param: LPARAM,
        ) -> LRESULT {
            if n_code >= 0 && w_param as u32 == WM_LBUTTONUP {
                let settings = crate::settings::get_settings().unwrap_or_default();

                if settings.selection_enabled && settings.selection_auto_mode {
                    if let Some(app) = APP_HANDLE.get() {
                        let app = app.clone();

                        std::thread::spawn(move || {
                            std::thread::sleep(Duration::from_millis(150));
                            trigger_selection(app);
                        });
                    }
                }
            }

            CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
        }

        unsafe {
            let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(hook_proc), std::ptr::null_mut(), 0);

            if hook.is_null() {
                MOUSE_MONITOR_STARTED.store(false, Ordering::SeqCst);
                return;
            }

            let mut msg = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {}
            UnhookWindowsHookEx(hook);
            MOUSE_MONITOR_STARTED.store(false, Ordering::SeqCst);
        }
    });
}
