use serde::Serialize;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, PhysicalPosition,
};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let show_main = MenuItemBuilder::with_id("show_main", "Open TrayBits").build(app)?;
            let demo_toast = MenuItemBuilder::with_id("demo_toast", "Show demo toast").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&show_main, &demo_toast, &quit])
                .build()?;

            let _tray = TrayIconBuilder::new()
                .tooltip("TrayBits")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show_main" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
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
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            hide_toast_overlay,
            notification_listener_status,
            push_demo_toast
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
