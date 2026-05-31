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

struct MouseDragState {
    start_x: i32,
    start_y: i32,
    started_at: Instant,
}

#[derive(Clone, Copy)]
enum CopyShortcut {
    CtrlC,
    CtrlShiftC,
}

const MIN_SELECTION_DRAG_DISTANCE_PX: i32 = 8;
const MAX_SELECTION_DRAG_DURATION: Duration = Duration::from_secs(8);
const MAX_UIA_PARENT_DEPTH: usize = 24;
const UIA_RETRY_DELAYS_MS: [u64; 5] = [0, 100, 180, 280, 420];
const SAME_TEXT_DEBOUNCE: Duration = Duration::from_millis(1500);

static STATE: Mutex<Option<SelectionState>> = Mutex::new(None);
static MOUSE_DRAG_STATE: Mutex<Option<MouseDragState>> = Mutex::new(None);
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

fn get_foreground_window_info() -> Option<(String, String)> {
    use winapi::shared::minwindef::DWORD;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::winbase::QueryFullProcessImageNameW;
    use winapi::um::winnt::PROCESS_QUERY_LIMITED_INFORMATION;
    use winapi::um::winuser::{GetClassNameW, GetForegroundWindow, GetWindowThreadProcessId};

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }

        let mut class_buf = [0u16; 256];
        let class_len = GetClassNameW(hwnd, class_buf.as_mut_ptr(), class_buf.len() as i32);
        let class_name = if class_len > 0 {
            String::from_utf16_lossy(&class_buf[..class_len as usize]).to_lowercase()
        } else {
            String::new()
        };

        let mut process_id: DWORD = 0;
        GetWindowThreadProcessId(hwnd, &mut process_id);
        if process_id == 0 {
            return Some((class_name, String::new()));
        }

        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
        if process.is_null() {
            return Some((class_name, String::new()));
        }

        let mut path_buf = [0u16; 1024];
        let mut path_len = path_buf.len() as DWORD;
        let exe_name =
            if QueryFullProcessImageNameW(process, 0, path_buf.as_mut_ptr(), &mut path_len) != 0 {
                let path = String::from_utf16_lossy(&path_buf[..path_len as usize]).to_lowercase();
                path.rsplit('\\').next().unwrap_or(&path).to_string()
            } else {
                String::new()
            };

        CloseHandle(process);
        Some((class_name, exe_name))
    }
}

fn is_terminal_foreground_window() -> bool {
    let Some((class_name, exe_name)) = get_foreground_window_info() else {
        return false;
    };

    matches!(
        class_name.as_str(),
        "consolewindowclass" | "cascadia_hosting_window_class"
    ) || matches!(
        exe_name.as_str(),
        "cmd.exe"
            | "powershell.exe"
            | "pwsh.exe"
            | "windowsterminal.exe"
            | "opencode.exe"
            | "conhost.exe"
            | "openconsole.exe"
    )
}

fn simulate_copy(shortcut: CopyShortcut) {
    use std::mem::size_of;
    use winapi::um::winuser::{
        SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, VK_CONTROL, VK_SHIFT,
    };

    unsafe {
        let input_count = match shortcut {
            CopyShortcut::CtrlC => 4,
            CopyShortcut::CtrlShiftC => 6,
        };
        let mut inputs: [INPUT; 6] = std::mem::zeroed();

        // Ctrl down
        inputs[0].type_ = INPUT_KEYBOARD;
        *inputs[0].u.ki_mut() = std::mem::zeroed();
        inputs[0].u.ki_mut().wVk = VK_CONTROL as u16;

        let c_down_index = match shortcut {
            CopyShortcut::CtrlC => 1,
            CopyShortcut::CtrlShiftC => {
                inputs[1].type_ = INPUT_KEYBOARD;
                *inputs[1].u.ki_mut() = std::mem::zeroed();
                inputs[1].u.ki_mut().wVk = VK_SHIFT as u16;
                2
            }
        };
        let c_up_index = c_down_index + 1;
        let shift_up_index = c_up_index + 1;
        let ctrl_up_index = input_count - 1;

        // C down
        inputs[c_down_index].type_ = INPUT_KEYBOARD;
        *inputs[c_down_index].u.ki_mut() = std::mem::zeroed();
        inputs[c_down_index].u.ki_mut().wVk = 0x43; // 'C' key

        // C up
        inputs[c_up_index].type_ = INPUT_KEYBOARD;
        *inputs[c_up_index].u.ki_mut() = std::mem::zeroed();
        inputs[c_up_index].u.ki_mut().wVk = 0x43;
        inputs[c_up_index].u.ki_mut().dwFlags = KEYEVENTF_KEYUP;

        if matches!(shortcut, CopyShortcut::CtrlShiftC) {
            inputs[shift_up_index].type_ = INPUT_KEYBOARD;
            *inputs[shift_up_index].u.ki_mut() = std::mem::zeroed();
            inputs[shift_up_index].u.ki_mut().wVk = VK_SHIFT as u16;
            inputs[shift_up_index].u.ki_mut().dwFlags = KEYEVENTF_KEYUP;
        }

        // Ctrl up
        inputs[ctrl_up_index].type_ = INPUT_KEYBOARD;
        *inputs[ctrl_up_index].u.ki_mut() = std::mem::zeroed();
        inputs[ctrl_up_index].u.ki_mut().wVk = VK_CONTROL as u16;
        inputs[ctrl_up_index].u.ki_mut().dwFlags = KEYEVENTF_KEYUP;

        SendInput(
            input_count as u32,
            inputs.as_mut_ptr(),
            size_of::<INPUT>() as i32,
        );
    }

    std::thread::sleep(Duration::from_millis(200));
}

fn read_clipboard(app: &AppHandle) -> Option<String> {
    app.clipboard().read_text().ok().filter(|t| !t.is_empty())
}

fn normalize_selected_text(text: String) -> Option<String> {
    let text = text.trim().to_string();
    if text.len() < 2 || text.len() > 5000 {
        return None;
    }

    Some(text)
}

fn get_selected_text_from_uia() -> Option<String> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_MULTITHREADED,
    };
    use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation, IUIAutomationElement};

    struct ComGuard(bool);

    impl Drop for ComGuard {
        fn drop(&mut self) {
            if self.0 {
                unsafe { CoUninitialize() };
            }
        }
    }

    fn get_text_from_element(element: &IUIAutomationElement) -> Option<String> {
        use windows::core::Interface;
        use windows::Win32::UI::Accessibility::{IUIAutomationTextPattern, UIA_TextPatternId};

        let pattern = unsafe { element.GetCurrentPattern(UIA_TextPatternId) }.ok()?;
        let text_pattern = pattern.cast::<IUIAutomationTextPattern>().ok()?;
        let selections = unsafe { text_pattern.GetSelection() }.ok()?;
        let count = unsafe { selections.Length() }.ok()?;

        for index in 0..count {
            let range = unsafe { selections.GetElement(index) }.ok()?;
            let text = unsafe { range.GetText(5000) }.ok()?.to_string();
            if let Some(text) = normalize_selected_text(text) {
                return Some(text);
            }
        }

        None
    }

    fn get_text_from_element_or_parents(
        automation: &IUIAutomation,
        element: IUIAutomationElement,
    ) -> Option<String> {
        let walker = unsafe { automation.RawViewWalker() }.ok()?;
        let mut current = Some(element);

        for _ in 0..MAX_UIA_PARENT_DEPTH {
            let element = current?;
            if let Some(text) = get_text_from_element(&element) {
                return Some(text);
            }

            current = unsafe { walker.GetParentElement(&element) }.ok();
        }

        None
    }

    let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
    let _guard = ComGuard(initialized);

    let automation: IUIAutomation =
        unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) }.ok()?;

    let (x, y) = get_mouse_position();
    let point = POINT {
        x: x as i32,
        y: y as i32,
    };

    if let Ok(element) = unsafe { automation.ElementFromPoint(point) } {
        if let Some(text) = get_text_from_element_or_parents(&automation, element) {
            return Some(text);
        }
    }

    let focused = unsafe { automation.GetFocusedElement() }.ok()?;
    get_text_from_element_or_parents(&automation, focused)
}

fn get_selected_text_from_clipboard(app: &AppHandle, restore_after_copy: bool) -> Option<String> {
    let before_copy = read_clipboard(app);

    if restore_after_copy {
        save_clipboard(app);
    }

    let shortcut = if is_terminal_foreground_window() {
        CopyShortcut::CtrlShiftC
    } else {
        CopyShortcut::CtrlC
    };

    simulate_copy(shortcut);

    let text = read_clipboard(app);

    if restore_after_copy {
        restore_clipboard(app);
    }

    let text = normalize_selected_text(text?)?;
    if before_copy.as_deref() == Some(text.as_str()) {
        return None;
    }

    Some(text)
}

fn read_mouse_hook_position(l_param: isize) -> Option<(i32, i32)> {
    use winapi::um::winuser::MSLLHOOKSTRUCT;

    if l_param == 0 {
        return None;
    }

    unsafe {
        let hook = &*(l_param as *const MSLLHOOKSTRUCT);
        Some((hook.pt.x, hook.pt.y))
    }
}

fn record_mouse_down(l_param: isize) {
    if let Some((x, y)) = read_mouse_hook_position(l_param) {
        let mut state = MOUSE_DRAG_STATE.lock().unwrap();
        *state = Some(MouseDragState {
            start_x: x,
            start_y: y,
            started_at: Instant::now(),
        });
    }
}

fn should_trigger_from_mouse_up(l_param: isize) -> bool {
    let Some((end_x, end_y)) = read_mouse_hook_position(l_param) else {
        return false;
    };

    let mut state = MOUSE_DRAG_STATE.lock().unwrap();
    let Some(start) = state.take() else {
        return false;
    };

    if start.started_at.elapsed() > MAX_SELECTION_DRAG_DURATION {
        return false;
    }

    let dx = (end_x - start.start_x).abs();
    let dy = (end_y - start.start_y).abs();

    dx.max(dy) >= MIN_SELECTION_DRAG_DISTANCE_PX
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

    if text == state.last_text && elapsed < SAME_TEXT_DEBOUNCE {
        return false;
    }

    if text.len() < 2 || text.len() > 5000 {
        return false;
    }

    state.last_trigger = now;
    state.last_text = text.to_string();
    true
}

fn show_selection_popup(app: AppHandle, text: String) {
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

pub fn trigger_selection_from_hotkey(app: AppHandle) {
    let settings = crate::settings::get_settings().unwrap_or_default();

    if !settings.selection_enabled {
        return;
    }

    let Some(text) = get_selected_text_from_clipboard(&app, settings.selection_restore_clipboard)
    else {
        return;
    };

    show_selection_popup(app, text);
}

fn trigger_selection_from_mouse(app: AppHandle) {
    for delay_ms in UIA_RETRY_DELAYS_MS {
        if delay_ms > 0 {
            std::thread::sleep(Duration::from_millis(delay_ms));
        }

        if let Some(text) = get_selected_text_from_uia() {
            show_selection_popup(app, text);
            return;
        }
    }
}

pub fn apply_settings(app: AppHandle, settings: &crate::settings::Settings) -> Result<(), String> {
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
                trigger_selection_from_hotkey(app_clone.clone());
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
            CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, WH_MOUSE_LL,
            WM_LBUTTONDOWN, WM_LBUTTONUP,
        };

        unsafe extern "system" fn hook_proc(
            n_code: i32,
            w_param: WPARAM,
            l_param: LPARAM,
        ) -> LRESULT {
            if n_code >= 0 && w_param as u32 == WM_LBUTTONDOWN {
                record_mouse_down(l_param);
            }

            if n_code >= 0
                && w_param as u32 == WM_LBUTTONUP
                && should_trigger_from_mouse_up(l_param)
            {
                let settings = crate::settings::get_settings().unwrap_or_default();

                if settings.selection_enabled && settings.selection_auto_mode {
                    if let Some(app) = APP_HANDLE.get() {
                        let app = app.clone();

                        std::thread::spawn(move || {
                            std::thread::sleep(Duration::from_millis(150));
                            trigger_selection_from_mouse(app);
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
