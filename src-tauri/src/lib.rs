use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::PathBuf, sync::Mutex};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Monitor, PhysicalPosition, State, WindowEvent,
};

const TRAY_ID: &str = "main";
const SETTINGS_FILE: &str = "settings.json";
const NOTIFICATION_SOUND_FILE: &str = "traybits-notification.wav";
const NOTIFICATION_SOUND_ASSET_PATH: &str = "assets/sounds/traybits-notification.wav";

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

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppNotification {
    id: String,
    title: String,
    body: String,
    source: String,
    source_app_user_model_id: Option<String>,
    origin: NotificationOrigin,
    created_at: String,
    tone: String,
    silent: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NotificationDeliveryStatus {
    ok: bool,
    message: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreviewNotificationResult {
    notification: AppNotification,
    native_notification: NotificationDeliveryStatus,
    overlay: NotificationDeliveryStatus,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
enum NotificationOrigin {
    Windows,
    Demo,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NotificationCaptureStatus {
    enabled: bool,
    access: String,
    message: String,
    mode: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OverlayMonitorOption {
    id: String,
    label: String,
    is_primary: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    run_on_startup: bool,
    run_high_priority: bool,
    close_behavior: CloseBehavior,
    enable_tray_icon: bool,
    #[serde(default = "default_notification_sound_enabled")]
    notification_sound_enabled: bool,
    #[serde(default = "default_notification_overlay_placement")]
    notification_overlay_placement: OverlayPlacement,
    #[serde(default = "default_notification_overlay_monitor")]
    notification_overlay_monitor: String,
    #[serde(default = "default_notification_overlay_debug_visible")]
    notification_overlay_debug_visible: bool,
    caps_lock_language_switch: CapsLockLanguageSwitchSettings,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum CloseBehavior {
    MinimizeToTray,
    Exit,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum OverlayPlacement {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    Center,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
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
    notifications: Mutex<Vec<AppNotification>>,
    notification_capture_status: Mutex<NotificationCaptureStatus>,
    captured_windows_notification_keys: Mutex<HashSet<String>>,
}

fn default_notification_overlay_placement() -> OverlayPlacement {
    OverlayPlacement::TopRight
}

fn default_notification_overlay_monitor() -> String {
    "primary".into()
}

fn default_notification_sound_enabled() -> bool {
    true
}

fn default_notification_overlay_debug_visible() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            run_on_startup: false,
            run_high_priority: false,
            close_behavior: CloseBehavior::MinimizeToTray,
            enable_tray_icon: true,
            notification_sound_enabled: default_notification_sound_enabled(),
            notification_overlay_placement: default_notification_overlay_placement(),
            notification_overlay_monitor: default_notification_overlay_monitor(),
            notification_overlay_debug_visible: default_notification_overlay_debug_visible(),
            caps_lock_language_switch: CapsLockLanguageSwitchSettings {
                enabled: false,
                preserve_caps_lock_with: CapsLockFallbackHotkey::CtrlCaps,
            },
        }
    }
}

impl Default for NotificationCaptureStatus {
    fn default() -> Self {
        Self {
            enabled: false,
            access: "notStarted".into(),
            message: "Windows notification capture has not started yet.".into(),
            mode: "none".into(),
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
    app.emit("traybits://settings-updated", settings.clone())
        .map_err(|error| error.to_string())?;
    sync_overlay_debug_visibility(&app, &settings);
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
fn get_notifications(state: State<'_, AppState>) -> Result<Vec<AppNotification>, String> {
    state
        .notifications
        .lock()
        .map(|notifications| notifications.clone())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_notification_capture_status(
    state: State<'_, AppState>,
) -> Result<NotificationCaptureStatus, String> {
    state
        .notification_capture_status
        .lock()
        .map(|status| status.clone())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_notification_overlay_monitors(app: AppHandle) -> Result<Vec<OverlayMonitorOption>, String> {
    let monitors = app
        .available_monitors()
        .map_err(|error| error.to_string())?;
    let primary_name = app
        .primary_monitor()
        .map_err(|error| error.to_string())?
        .and_then(|monitor| monitor.name().cloned());

    let mut options = vec![OverlayMonitorOption {
        id: "primary".into(),
        label: "Primary screen".into(),
        is_primary: true,
    }];

    options.extend(monitors.iter().enumerate().map(|(index, monitor)| {
        let size = monitor.size();
        let name = monitor
            .name()
            .cloned()
            .unwrap_or_else(|| format!("Screen {}", index + 1));
        let is_primary = primary_name
            .as_ref()
            .is_some_and(|primary| monitor.name().is_some_and(|name| name == primary));
        OverlayMonitorOption {
            id: format!("monitor:{index}"),
            label: format!(
                "Screen {} - {} ({} x {})",
                index + 1,
                name,
                size.width,
                size.height
            ),
            is_primary,
        }
    }));

    Ok(options)
}

#[tauri::command]
fn push_demo_notification(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<PreviewNotificationResult, String> {
    let settings = state
        .settings
        .lock()
        .map(|settings| settings.clone())
        .map_err(|error| error.to_string())?;
    let notification = AppNotification {
        id: format!("demo-{}", monotonic_millis()),
        title: "Demo notification".into(),
        body: "TrayBits sent this native Windows notification and mirrored it into the persistent overlay.".into(),
        source: "TrayBits".into(),
        source_app_user_model_id: None,
        origin: NotificationOrigin::Demo,
        created_at: now_timestamp(),
        tone: "windows".into(),
        silent: false,
    };

    let native_notification =
        match show_native_notification(&app, &notification, settings.notification_sound_enabled) {
            Ok(()) => NotificationDeliveryStatus {
                ok: true,
                message: "Native Windows notification sent through WinRT.".into(),
            },
            Err(error) => NotificationDeliveryStatus {
                ok: false,
                message: error,
            },
        };
    mark_notification_seen(state.inner(), &notification);
    let overlay = add_notification(
        &app,
        state.inner(),
        notification.clone(),
        true,
        settings.notification_sound_enabled,
    )?;
    Ok(PreviewNotificationResult {
        notification,
        native_notification,
        overlay,
    })
}

#[tauri::command]
fn dismiss_notification(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    dismiss_notification_by_id(&app, state.inner(), &id)
}

#[tauri::command]
fn clear_notifications(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    state
        .notifications
        .lock()
        .map_err(|error| error.to_string())?
        .clear();
    app.emit("traybits://notifications-cleared", ())
        .map_err(|error| error.to_string())?;
    hide_toast_overlay(app)
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

fn add_notification(
    app: &AppHandle,
    state: &AppState,
    notification: AppNotification,
    show_overlay: bool,
    play_sound: bool,
) -> Result<NotificationDeliveryStatus, String> {
    {
        let mut notifications = state
            .notifications
            .lock()
            .map_err(|error| error.to_string())?;
        notifications.insert(0, notification.clone());
        notifications.truncate(100);
    }

    let overlay = if show_overlay {
        match show_toast_window(app) {
            Ok(()) => NotificationDeliveryStatus {
                ok: true,
                message: "Transparent overlay window was requested.".into(),
            },
            Err(error) => NotificationDeliveryStatus {
                ok: false,
                message: error,
            },
        }
    } else {
        NotificationDeliveryStatus {
            ok: true,
            message: "Overlay intentionally skipped.".into(),
        }
    };
    if play_sound {
        notification_sound::play(app);
    }
    app.emit("traybits://notification-added", notification.clone())
        .map_err(|error| error.to_string())?;
    if show_overlay {
        app.emit_to("toast", "traybits://notification-added", notification)
            .map_err(|error| error.to_string())?;
    }
    Ok(overlay)
}

fn show_native_notification(
    app: &AppHandle,
    notification: &AppNotification,
    sound_enabled: bool,
) -> Result<(), String> {
    native_windows_notification::show(app, notification, sound_enabled)
}

fn mark_notification_seen(state: &AppState, notification: &AppNotification) {
    if let Ok(mut captured_keys) = state.captured_windows_notification_keys.lock() {
        captured_keys.insert(notification_dedupe_key(notification));
    }
}

fn notification_dedupe_key(notification: &AppNotification) -> String {
    format!(
        "{}|{}|{}",
        notification.source, notification.title, notification.body
    )
}

fn dismiss_notification_by_id(app: &AppHandle, state: &AppState, id: &str) -> Result<(), String> {
    let mut notifications = state
        .notifications
        .lock()
        .map_err(|error| error.to_string())?;
    let previous_len = notifications.len();
    notifications.retain(|notification| notification.id != id);

    if notifications.len() != previous_len {
        app.emit("traybits://notification-dismissed", id.to_string())
            .map_err(|error| error.to_string())?;
    }

    if notifications.is_empty() {
        let _ = hide_toast_overlay(app.clone());
    }

    Ok(())
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

fn sync_overlay_debug_visibility(app: &AppHandle, settings: &AppSettings) {
    if settings.notification_overlay_debug_visible {
        let _ = show_toast_window(app);
    } else {
        let _ = hide_toast_overlay(app.clone());
    }
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

    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
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
        });

    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }

    tray.build(app)
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

    if let Some(monitor) = selected_overlay_monitor(app, &window) {
        let work_area = monitor.work_area();
        let scale = monitor.scale_factor();
        let logical_width = 440.0 * scale;
        let logical_height = 520.0 * scale;
        let margin = 24.0 * scale;
        let placement = app
            .try_state::<AppState>()
            .and_then(|state| {
                state
                    .settings
                    .lock()
                    .ok()
                    .map(|settings| settings.notification_overlay_placement)
            })
            .unwrap_or(OverlayPlacement::TopRight);
        let (x, y) = overlay_position(
            placement,
            work_area.position.x as f64,
            work_area.position.y as f64,
            work_area.size.width as f64,
            work_area.size.height as f64,
            logical_width,
            logical_height,
            margin,
        );
        let _ = window.set_position(PhysicalPosition::new(x as i32, y as i32));
    }

    let _ = window.set_always_on_top(true);
    window.show().map_err(|error| error.to_string())
}

fn selected_overlay_monitor(app: &AppHandle, window: &tauri::WebviewWindow) -> Option<Monitor> {
    let selected = app
        .try_state::<AppState>()
        .and_then(|state| {
            state
                .settings
                .lock()
                .ok()
                .map(|settings| settings.notification_overlay_monitor.clone())
        })
        .unwrap_or_else(default_notification_overlay_monitor);

    if selected == "primary" {
        return app.primary_monitor().ok().flatten();
    }

    let index = selected
        .strip_prefix("monitor:")
        .and_then(|value| value.parse::<usize>().ok());

    if let Some(index) = index {
        if let Ok(monitors) = app.available_monitors() {
            if let Some(monitor) = monitors.into_iter().nth(index) {
                return Some(monitor);
            }
        }
    }

    window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten())
        .or_else(|| {
            app.available_monitors()
                .ok()
                .and_then(|mut monitors| monitors.pop())
        })
}

fn overlay_position(
    placement: OverlayPlacement,
    screen_x: f64,
    screen_y: f64,
    screen_width: f64,
    screen_height: f64,
    overlay_width: f64,
    overlay_height: f64,
    margin: f64,
) -> (f64, f64) {
    let left = screen_x + margin;
    let center_x = screen_x + (screen_width - overlay_width) / 2.0;
    let right = screen_x + screen_width - overlay_width - margin;
    let top = screen_y + margin;
    let center_y = screen_y + (screen_height - overlay_height) / 2.0;
    let bottom = screen_y + screen_height - overlay_height - margin;

    match placement {
        OverlayPlacement::TopLeft => (left, top),
        OverlayPlacement::TopCenter => (center_x, top),
        OverlayPlacement::TopRight => (right, top),
        OverlayPlacement::MiddleLeft => (left, center_y),
        OverlayPlacement::Center => (center_x, center_y),
        OverlayPlacement::MiddleRight => (right, center_y),
        OverlayPlacement::BottomLeft => (left, bottom),
        OverlayPlacement::BottomCenter => (center_x, bottom),
        OverlayPlacement::BottomRight => (right, bottom),
    }
}

fn monotonic_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

fn now_timestamp() -> String {
    monotonic_millis().to_string()
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
mod notification_sound {
    use super::{AppHandle, NOTIFICATION_SOUND_ASSET_PATH, NOTIFICATION_SOUND_FILE};
    use std::{os::windows::ffi::OsStrExt, path::PathBuf};
    use tauri::Manager;
    use windows::core::PCWSTR;
    use windows::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_NODEFAULT};

    pub fn play(app: &AppHandle) {
        let Some(path) = sound_path(app) else {
            return;
        };
        let mut wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let _ = PlaySoundW(
                PCWSTR(wide.as_mut_ptr()),
                None,
                SND_FILENAME | SND_ASYNC | SND_NODEFAULT,
            );
        }
    }

    fn sound_path(app: &AppHandle) -> Option<PathBuf> {
        let mut candidates = Vec::new();

        if let Ok(resource_dir) = app.path().resource_dir() {
            candidates.push(resource_dir.join(NOTIFICATION_SOUND_FILE));
            candidates.push(resource_dir.join(NOTIFICATION_SOUND_ASSET_PATH));
        }

        if let Ok(current_dir) = std::env::current_dir() {
            candidates.push(current_dir.join(NOTIFICATION_SOUND_ASSET_PATH));
            candidates.push(current_dir.join("..").join(NOTIFICATION_SOUND_ASSET_PATH));
        }

        candidates.into_iter().find(|path| path.is_file())
    }
}

#[cfg(not(target_os = "windows"))]
mod notification_sound {
    use super::AppHandle;

    pub fn play(_app: &AppHandle) {}
}

#[cfg(target_os = "windows")]
mod native_windows_notification {
    use super::{AppHandle, AppNotification};
    use tauri_winrt_notification::{Duration, Toast};
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    pub fn show(
        app: &AppHandle,
        notification: &AppNotification,
        sound_enabled: bool,
    ) -> Result<(), String> {
        let app_id = app.config().identifier.clone();
        register_app_user_model_id(&app_id)?;

        let mut toast = Toast::new(&app_id)
            .title(&notification.title)
            .text1(&notification.body)
            .duration(Duration::Short);

        if !sound_enabled {
            toast = toast.sound(None);
        }

        toast
            .show()
            .map_err(|error| format!("Could not send WinRT notification: {error}"))
    }

    fn register_app_user_model_id(app_id: &str) -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey(format!(r"SOFTWARE\Classes\AppUserModelId\{app_id}"))
            .map_err(|error| format!("Could not register AppUserModelId: {error}"))?;

        key.set_value("DisplayName", &"TrayBits")
            .map_err(|error| format!("Could not write notification display name: {error}"))?;
        key.set_value("IconBackgroundColor", &"0")
            .map_err(|error| format!("Could not write notification icon color: {error}"))?;

        if let Ok(exe) = std::env::current_exe() {
            let icon_uri = exe.to_string_lossy().to_string();
            let _ = key.set_value("IconUri", &icon_uri);
        }

        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
mod native_windows_notification {
    use super::{AppHandle, AppNotification};

    pub fn show(
        _app: &AppHandle,
        _notification: &AppNotification,
        _sound_enabled: bool,
    ) -> Result<(), String> {
        Err("Native Windows notifications are only available on Windows.".into())
    }
}

fn set_notification_capture_status(
    state: &AppState,
    enabled: bool,
    access: impl Into<String>,
    message: impl Into<String>,
    mode: impl Into<String>,
) {
    if let Ok(mut status) = state.notification_capture_status.lock() {
        *status = NotificationCaptureStatus {
            enabled,
            access: access.into(),
            message: message.into(),
            mode: mode.into(),
        };
    }
}

#[cfg(target_os = "windows")]
mod notification_capture {
    use super::{
        add_notification, notification_dedupe_key, now_timestamp, set_notification_capture_status,
        AppHandle, AppNotification, AppState, NotificationOrigin,
    };
    use std::{thread, time::Duration};
    use tauri::Manager;
    use windows::UI::Notifications::Management::{
        UserNotificationListener, UserNotificationListenerAccessStatus,
    };
    use windows::UI::Notifications::{
        KnownNotificationBindings, NotificationKinds, UserNotification,
    };

    pub fn start(app: AppHandle) {
        thread::spawn(move || run_capture_loop(app));
    }

    fn run_capture_loop(app: AppHandle) {
        let state = app.state::<AppState>();
        let listener = match UserNotificationListener::Current() {
            Ok(listener) => listener,
            Err(error) => {
                set_notification_capture_status(
                    state.inner(),
                    false,
                    "unavailable",
                    format!("Windows notification capture is unavailable: {error}"),
                    "none",
                );
                return;
            }
        };

        let mut access = match listener.GetAccessStatus() {
            Ok(access) => access,
            Err(error) => {
                set_notification_capture_status(
                    state.inner(),
                    false,
                    "error",
                    format!("Could not read Windows notification access status: {error}"),
                    "none",
                );
                return;
            }
        };

        if access == UserNotificationListenerAccessStatus::Unspecified {
            match listener
                .RequestAccessAsync()
                .and_then(|operation| operation.get())
            {
                Ok(requested_access) => access = requested_access,
                Err(error) => {
                    set_notification_capture_status(
                        state.inner(),
                        false,
                        "error",
                        format!("Windows notification access request failed: {error}"),
                        "none",
                    );
                    return;
                }
            }
        }

        if access != UserNotificationListenerAccessStatus::Allowed {
            set_notification_capture_status(
                state.inner(),
                false,
                access_status_name(access),
                "Windows notification capture is not allowed.",
                "none",
            );
            return;
        }

        set_notification_capture_status(
            state.inner(),
            true,
            "allowed",
            "Windows notification capture is enabled.",
            "polling",
        );

        let mut initial_sync = true;
        loop {
            capture_current_notifications(&app, state.inner(), &listener, initial_sync);
            initial_sync = false;
            thread::sleep(Duration::from_millis(750));
        }
    }

    fn capture_current_notifications(
        app: &AppHandle,
        state: &AppState,
        listener: &UserNotificationListener,
        initial_sync: bool,
    ) {
        let notifications = match listener
            .GetNotificationsAsync(NotificationKinds::Toast)
            .and_then(|operation| operation.get())
        {
            Ok(notifications) => notifications,
            Err(error) => {
                set_notification_capture_status(
                    state,
                    false,
                    "error",
                    format!("Windows notification polling failed: {error}"),
                    "polling",
                );
                return;
            }
        };

        let Ok(size) = notifications.Size() else {
            return;
        };

        for index in 0..size {
            let Ok(notification) = notifications.GetAt(index) else {
                continue;
            };
            let Ok(notification_id) = notification.Id() else {
                continue;
            };

            let Some(mut app_notification) = extract_notification(notification_id, &notification)
            else {
                continue;
            };
            app_notification.silent = initial_sync;
            let dedupe_key = notification_dedupe_key(&app_notification);
            let already_seen = {
                let Ok(mut captured_keys) = state.captured_windows_notification_keys.lock() else {
                    continue;
                };
                !captured_keys.insert(dedupe_key)
            };

            if already_seen {
                continue;
            }

            let sound_enabled = state
                .settings
                .lock()
                .map(|settings| settings.notification_sound_enabled)
                .unwrap_or(true);
            let _ = add_notification(
                app,
                state,
                app_notification,
                !initial_sync,
                !initial_sync && sound_enabled,
            );
        }
    }

    fn extract_notification(id: u32, notification: &UserNotification) -> Option<AppNotification> {
        let texts = extract_texts(notification);
        let (source, source_app_user_model_id) = extract_source(notification);
        let title = texts
            .first()
            .cloned()
            .filter(|text| !text.trim().is_empty())
            .unwrap_or_else(|| source.clone());
        let body = if texts.len() > 1 {
            texts[1..].join("\n")
        } else {
            format!("Windows notification ID {id}")
        };

        Some(AppNotification {
            id: format!("windows-{id}"),
            title,
            body,
            source,
            source_app_user_model_id,
            origin: NotificationOrigin::Windows,
            created_at: now_timestamp(),
            tone: "windows".into(),
            silent: false,
        })
    }

    fn extract_texts(notification: &UserNotification) -> Vec<String> {
        let Ok(notification_payload) = notification.Notification() else {
            return Vec::new();
        };
        let Ok(visual) = notification_payload.Visual() else {
            return Vec::new();
        };
        let Ok(template) = KnownNotificationBindings::ToastGeneric() else {
            return Vec::new();
        };
        let Ok(binding) = visual.GetBinding(&template) else {
            return Vec::new();
        };
        let Ok(text_elements) = binding.GetTextElements() else {
            return Vec::new();
        };
        let Ok(size) = text_elements.Size() else {
            return Vec::new();
        };

        let mut texts = Vec::new();
        for index in 0..size {
            if let Ok(text_element) = text_elements.GetAt(index) {
                if let Ok(text) = text_element.Text() {
                    let text = text.to_string_lossy();
                    if !text.trim().is_empty() {
                        texts.push(text);
                    }
                }
            }
        }
        texts
    }

    fn extract_source(notification: &UserNotification) -> (String, Option<String>) {
        let Ok(app_info) = notification.AppInfo() else {
            return ("Windows notification".into(), None);
        };
        let display_name = app_info
            .DisplayInfo()
            .and_then(|display_info| display_info.DisplayName())
            .map(|name| name.to_string_lossy())
            .unwrap_or_else(|_| "Windows notification".into());
        let app_user_model_id = app_info
            .AppUserModelId()
            .map(|id| id.to_string_lossy())
            .ok();

        (display_name, app_user_model_id)
    }

    fn access_status_name(access: UserNotificationListenerAccessStatus) -> &'static str {
        if access == UserNotificationListenerAccessStatus::Allowed {
            "allowed"
        } else if access == UserNotificationListenerAccessStatus::Denied {
            "denied"
        } else {
            "unspecified"
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod notification_capture {
    use super::{set_notification_capture_status, AppHandle, AppState};

    pub fn start(app: AppHandle) {
        let state = app.state::<AppState>();
        set_notification_capture_status(
            state.inner(),
            false,
            "unsupported",
            "Windows notification capture is only available on Windows.",
            "none",
        );
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
        System::Threading::GetCurrentProcessId,
        UI::{
            Input::KeyboardAndMouse::{
                keybd_event, ActivateKeyboardLayout, GetAsyncKeyState, GetKeyState,
                GetKeyboardLayout, GetKeyboardLayoutList, ACTIVATE_KEYBOARD_LAYOUT_FLAGS,
                KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KLF_ACTIVATE, KLF_SETFORPROCESS,
                VK_CAPITAL, VK_CONTROL, VK_MENU, VK_SHIFT,
            },
            WindowsAndMessaging::{
                CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetMessageW,
                GetWindowThreadProcessId, PostMessageW, SetWindowsHookExW, TranslateMessage,
                HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_INPUTLANGCHANGEREQUEST,
                WM_KEYDOWN, WM_SYSKEYDOWN,
            },
        },
    };

    const HKL_NEXT: isize = 1;
    const INPUTLANGCHANGE_FORWARD: usize = 2;

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
            if is_keydown_message(wparam) {
                toggle_caps_lock();
            }
            return LRESULT(1);
        }

        if is_keydown_message(wparam) {
            force_caps_lock_off();
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

    fn is_keydown_message(wparam: WPARAM) -> bool {
        wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize
    }

    fn is_caps_lock_on() -> bool {
        unsafe { GetKeyState(VK_CAPITAL.0 as i32) & 1 != 0 }
    }

    fn force_caps_lock_off() {
        if is_caps_lock_on() {
            toggle_caps_lock();
        }
    }

    fn toggle_caps_lock() {
        unsafe {
            keybd_event(VK_CAPITAL.0 as u8, 0x45, KEYEVENTF_EXTENDEDKEY, 0);
            keybd_event(
                VK_CAPITAL.0 as u8,
                0x45,
                KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP,
                0,
            );
        }
    }

    fn switch_input_language() {
        let foreground_window = unsafe { GetForegroundWindow() };
        if foreground_window.is_invalid() {
            return;
        }

        let mut process_id = 0;
        let foreground_thread_id =
            unsafe { GetWindowThreadProcessId(foreground_window, Some(&mut process_id)) };

        if process_id == unsafe { GetCurrentProcessId() } {
            activate_next_keyboard_layout(foreground_thread_id);
            return;
        }

        let _ = unsafe {
            PostMessageW(
                Some(foreground_window),
                WM_INPUTLANGCHANGEREQUEST,
                WPARAM(INPUTLANGCHANGE_FORWARD),
                LPARAM(HKL_NEXT),
            )
        };
    }

    fn activate_next_keyboard_layout(thread_id: u32) {
        let count = unsafe { GetKeyboardLayoutList(None) };
        if count <= 1 {
            return;
        }

        let mut layouts = vec![Default::default(); count as usize];
        let loaded = unsafe { GetKeyboardLayoutList(Some(&mut layouts)) };
        if loaded <= 1 {
            return;
        }

        let current = unsafe { GetKeyboardLayout(thread_id) };
        let next = layouts
            .iter()
            .position(|layout| *layout == current)
            .map(|index| layouts[(index + 1) % loaded as usize])
            .unwrap_or(layouts[0]);

        let flags = ACTIVATE_KEYBOARD_LAYOUT_FLAGS(KLF_ACTIVATE.0 | KLF_SETFORPROCESS.0);
        let _ = unsafe { ActivateKeyboardLayout(next, flags) };
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
                notifications: Mutex::new(Vec::new()),
                notification_capture_status: Mutex::new(NotificationCaptureStatus::default()),
                captured_windows_notification_keys: Mutex::new(HashSet::new()),
            });
            apply_runtime_settings(app.handle(), &settings)?;
            keyboard::apply_settings(&settings.caps_lock_language_switch);
            sync_overlay_debug_visibility(app.handle(), &settings);
            notification_capture::start(app.handle().clone());
            Ok(())
        })
        .on_window_event(handle_window_event)
        .invoke_handler(tauri::generate_handler![
            clear_notifications,
            dismiss_notification,
            get_app_settings,
            get_notification_capture_status,
            get_notification_overlay_monitors,
            get_notifications,
            hide_toast_overlay,
            notification_listener_status,
            push_demo_notification,
            push_demo_toast,
            update_app_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
