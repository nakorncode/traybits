use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Mutex};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, PhysicalPosition, State, WindowEvent,
};

const TRAY_ID: &str = "main";
const SETTINGS_FILE: &str = "settings.json";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ToastPayload {
    id: u64,
    source: String,
    title: String,
    body: String,
    tone: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NotificationListenerStatus {
    feasible: bool,
    api: String,
    permission_required: bool,
    packaging_risk: String,
    prototype_step: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    run_on_startup: bool,
    run_high_priority: bool,
    close_behavior: CloseBehavior,
    enable_tray_icon: bool,
    caps_lock_language_switch: CapsLockLanguageSwitchSettings,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum CloseBehavior {
    MinimizeToTray,
    Exit,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CapsLockLanguageSwitchSettings {
    enabled: bool,
    preserve_caps_lock_with: CapsLockFallbackHotkey,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum CapsLockFallbackHotkey {
    CtrlCaps,
    ShiftCaps,
    AltCaps,
}

struct AppState {
    settings: Mutex<AppSettings>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            run_on_startup: false,
            run_high_priority: false,
            close_behavior: CloseBehavior::MinimizeToTray,
            enable_tray_icon: true,
            caps_lock_language_switch: CapsLockLanguageSwitchSettings {
                enabled: false,
                preserve_caps_lock_with: CapsLockFallbackHotkey::CtrlCaps,
            },
        }
    }
}

#[tauri::command]
fn get_app_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    state
        .settings
        .lock()
        .map(|settings| settings.clone())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn update_app_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let settings = normalize_settings(settings);
    apply_runtime_settings(&app, &settings)?;
    save_settings(&app, &settings)?;
    keyboard::apply_settings(&settings.caps_lock_language_switch);

    *state.settings.lock().map_err(|error| error.to_string())? = settings.clone();
    Ok(settings)
}

#[tauri::command]
fn notification_listener_status() -> NotificationListenerStatus {
    NotificationListenerStatus {
        feasible: true,
        api: "Windows.UI.Notifications.Management.UserNotificationListener".into(),
        permission_required: true,
        packaging_risk: "Needs a packaged Windows identity, user notification listener capability, and explicit user permission before real notification capture can be trusted.".into(),
        prototype_step: "Build a Windows-only Rust spike with the windows crate, request access on the UI thread, subscribe to NotificationChanged, then sync GetNotificationsAsync(NotificationKinds.Toast).".into(),
    }
}

#[tauri::command]
fn push_demo_toast(app: AppHandle, tone: String) -> Result<(), String> {
    let payload = ToastPayload {
        id: monotonic_millis(),
        source: match tone.as_str() {
            "language-switch" => "Caps Lock Language Switch".into(),
            "rest" => "Eye Rest Reminder".into(),
            "windows" => "Windows Notification".into(),
            _ => "TrayBits".into(),
        },
        title: match tone.as_str() {
            "language-switch" => "Input language switched".into(),
            "rest" => "Look 20 feet away".into(),
            "windows" => "New notification captured".into(),
            _ => "TrayBits toast".into(),
        },
        body: match tone.as_str() {
            "language-switch" => "Caps Lock was intercepted by Rust and converted into a language-switch action.".into(),
            "rest" => "Rest your eyes for 20 seconds. This is rendered by the Solid overlay window.".into(),
            "windows" => "This demo event follows the same bridge shape planned for UserNotificationListener.".into(),
            _ => "Rust emitted this event to the always-on-top toast webview.".into(),
        },
        tone,
    };

    show_toast_window(&app)?;
    app.emit_to("toast", "traybits://toast", payload)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn hide_toast_overlay(app: AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("toast") else {
        return Ok(());
    };

    window.hide().map_err(|error| error.to_string())
}

fn normalize_settings(mut settings: AppSettings) -> AppSettings {
    if !settings.enable_tray_icon && settings.close_behavior == CloseBehavior::MinimizeToTray {
        settings.close_behavior = CloseBehavior::Exit;
    }
    settings
}

fn load_settings(app: &AppHandle) -> AppSettings {
    let Ok(path) = settings_path(app) else {
        return AppSettings::default();
    };

    let Ok(raw) = fs::read_to_string(path) else {
        return AppSettings::default();
    };

    serde_json::from_str::<AppSettings>(&raw)
        .map(normalize_settings)
        .unwrap_or_default()
}

fn save_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let raw = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    fs::write(path, raw).map_err(|error| error.to_string())
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join(SETTINGS_FILE))
        .map_err(|error| error.to_string())
}

fn apply_runtime_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    startup::set_run_on_startup(settings.run_on_startup)?;
    priority::set_high_priority(settings.run_high_priority)?;
    sync_tray_icon(app, settings.enable_tray_icon)?;
    Ok(())
}

fn sync_tray_icon(app: &AppHandle, enabled: bool) -> Result<(), String> {
    if enabled {
        if app.tray_by_id(TRAY_ID).is_none() {
            create_tray_icon(app)?;
        }
    } else {
        let _ = app.remove_tray_by_id(TRAY_ID);
    }

    Ok(())
}

fn create_tray_icon(app: &AppHandle) -> Result<(), String> {
    let show_main = MenuItemBuilder::with_id("show_main", "Open TrayBits")
        .build(app)
        .map_err(|error| error.to_string())?;
    let demo_toast = MenuItemBuilder::with_id("demo_toast", "Show demo toast")
        .build(app)
        .map_err(|error| error.to_string())?;
    let quit = MenuItemBuilder::with_id("quit", "Quit")
        .build(app)
        .map_err(|error| error.to_string())?;
    let menu = MenuBuilder::new(app)
        .items(&[&show_main, &demo_toast, &quit])
        .build()
        .map_err(|error| error.to_string())?;

    TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("TrayBits")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show_main" => show_main_window(app),
            "demo_toast" => {
                let _ = push_demo_toast(app.clone(), "windows".into());
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn show_toast_window(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("toast") else {
        return Err("toast window is not configured".into());
    };

    if let Ok(Some(monitor)) = window.current_monitor() {
        let size = monitor.size();
        let scale = monitor.scale_factor();
        let logical_width = 440.0 * scale;
        let logical_height = 520.0 * scale;
        let margin = 24.0 * scale;
        let x = size.width as f64 - logical_width - margin;
        let y = size.height as f64 - logical_height - margin;
        let _ = window.set_position(PhysicalPosition::new(x.max(0.0) as i32, y.max(0.0) as i32));
    }

    let _ = window.set_always_on_top(true);
    window.show().map_err(|error| error.to_string())
}

fn monotonic_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

fn handle_window_event(window: &tauri::Window, event: &WindowEvent) {
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };

    if window.label() != "main" {
        return;
    }

    let app = window.app_handle();
    let state = app.state::<AppState>();
    let settings = state.settings.lock().map(|settings| settings.clone());

    if matches!(
        settings,
        Ok(AppSettings {
            close_behavior: CloseBehavior::MinimizeToTray,
            enable_tray_icon: true,
            ..
        })
    ) {
        api.prevent_close();
        let _ = window.hide();
    }
}

#[cfg(target_os = "windows")]
mod startup {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const VALUE_NAME: &str = "TrayBits";

    pub fn set_run_on_startup(enabled: bool) -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey(RUN_KEY)
            .map_err(|error| error.to_string())?;

        if enabled {
            let exe = std::env::current_exe().map_err(|error| error.to_string())?;
            key.set_value(VALUE_NAME, &format!("\"{}\"", exe.display()))
                .map_err(|error| error.to_string())
        } else {
            match key.delete_value(VALUE_NAME) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error.to_string()),
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod startup {
    pub fn set_run_on_startup(_enabled: bool) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(target_os = "windows")]
mod priority {
    use windows::Win32::System::Threading::{
        GetCurrentProcess, SetPriorityClass, HIGH_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
    };

    pub fn set_high_priority(enabled: bool) -> Result<(), String> {
        let class = if enabled {
            HIGH_PRIORITY_CLASS
        } else {
            NORMAL_PRIORITY_CLASS
        };

        unsafe { SetPriorityClass(GetCurrentProcess(), class) }.map_err(|error| error.to_string())
    }
}

#[cfg(not(target_os = "windows"))]
mod priority {
    pub fn set_high_priority(_enabled: bool) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(target_os = "windows")]
mod keyboard {
    use super::{CapsLockFallbackHotkey, CapsLockLanguageSwitchSettings};
    use std::{
        sync::{Mutex, OnceLock},
        thread,
    };
    use windows::Win32::{
        Foundation::{LPARAM, LRESULT, WPARAM},
        UI::{
            Input::KeyboardAndMouse::{
                GetAsyncKeyState, VK_CAPITAL, VK_CONTROL, VK_MENU, VK_SHIFT,
            },
            WindowsAndMessaging::{
                CallNextHookEx, DispatchMessageW, GetMessageW, PostMessageW, SetWindowsHookExW,
                TranslateMessage, HC_ACTION, HHOOK, HWND_BROADCAST, KBDLLHOOKSTRUCT, MSG,
                WH_KEYBOARD_LL, WM_INPUTLANGCHANGEREQUEST, WM_KEYDOWN, WM_SYSKEYDOWN,
            },
        },
    };

    const HKL_NEXT: isize = 1;

    #[derive(Clone, Copy)]
    struct HookSettings {
        enabled: bool,
        preserve_caps_lock_with: CapsLockFallbackHotkey,
    }

    static SETTINGS: OnceLock<Mutex<HookSettings>> = OnceLock::new();
    static HOOK_STARTED: OnceLock<()> = OnceLock::new();

    pub fn start() {
        let _ = SETTINGS.set(Mutex::new(HookSettings {
            enabled: false,
            preserve_caps_lock_with: CapsLockFallbackHotkey::CtrlCaps,
        }));

        let _ = HOOK_STARTED.get_or_init(|| {
            thread::spawn(|| unsafe {
                let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0);
                if let Ok(hook) = hook {
                    run_message_loop(hook);
                }
            });
        });
    }

    pub fn apply_settings(settings: &CapsLockLanguageSwitchSettings) {
        start();
        if let Some(lock) = SETTINGS.get() {
            if let Ok(mut hook_settings) = lock.lock() {
                *hook_settings = HookSettings {
                    enabled: settings.enabled,
                    preserve_caps_lock_with: settings.preserve_caps_lock_with,
                };
            }
        }
    }

    unsafe fn run_message_loop(_hook: HHOOK) {
        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }

    unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code != HC_ACTION as i32 {
            return CallNextHookEx(None, code, wparam, lparam);
        }

        let event = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        if event.vkCode != VK_CAPITAL.0 as u32 {
            return CallNextHookEx(None, code, wparam, lparam);
        }

        let Some(settings) = current_settings() else {
            return CallNextHookEx(None, code, wparam, lparam);
        };

        if !settings.enabled {
            return CallNextHookEx(None, code, wparam, lparam);
        }

        if preserve_combo_pressed(settings.preserve_caps_lock_with) {
            return CallNextHookEx(None, code, wparam, lparam);
        }

        if wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize {
            switch_input_language();
        }

        LRESULT(1)
    }

    fn current_settings() -> Option<HookSettings> {
        SETTINGS
            .get()
            .and_then(|lock| lock.lock().ok().map(|settings| *settings))
    }

    fn preserve_combo_pressed(hotkey: CapsLockFallbackHotkey) -> bool {
        match hotkey {
            CapsLockFallbackHotkey::CtrlCaps => key_down(VK_CONTROL.0 as i32),
            CapsLockFallbackHotkey::ShiftCaps => key_down(VK_SHIFT.0 as i32),
            CapsLockFallbackHotkey::AltCaps => key_down(VK_MENU.0 as i32),
        }
    }

    fn key_down(vkey: i32) -> bool {
        unsafe { GetAsyncKeyState(vkey) < 0 }
    }

    fn switch_input_language() {
        let _ = unsafe {
            PostMessageW(
                Some(HWND_BROADCAST),
                WM_INPUTLANGCHANGEREQUEST,
                WPARAM(0),
                LPARAM(HKL_NEXT),
            )
        };
    }
}

#[cfg(not(target_os = "windows"))]
mod keyboard {
    use super::CapsLockLanguageSwitchSettings;

    pub fn start() {}

    pub fn apply_settings(_settings: &CapsLockLanguageSwitchSettings) {}
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let settings = load_settings(app.handle());
            app.manage(AppState {
                settings: Mutex::new(settings.clone()),
            });
            apply_runtime_settings(app.handle(), &settings)?;
            keyboard::apply_settings(&settings.caps_lock_language_switch);
            Ok(())
        })
        .on_window_event(handle_window_event)
        .invoke_handler(tauri::generate_handler![
            get_app_settings,
            hide_toast_overlay,
            notification_listener_status,
            push_demo_toast,
            update_app_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
