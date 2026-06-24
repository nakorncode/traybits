use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::PathBuf, process::Command, sync::Mutex};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Monitor, PhysicalPosition, PhysicalSize, State, WindowEvent,
};

const TRAY_ID: &str = "main";
const SETTINGS_FILE: &str = "settings.json";
const DEFAULT_NOTIFICATION_SOUND_PRESET: &str = "aosp-argon";
const EYE_REST_REST_MILLIS: u64 = 20_000;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NotificationSoundPreset {
    id: &'static str,
    label: &'static str,
    file: &'static str,
}

const NOTIFICATION_SOUND_PRESETS: &[NotificationSoundPreset] = &[
    NotificationSoundPreset {
        id: "aosp-acrux",
        label: "AOSP Acrux",
        file: "aosp-acrux.wav",
    },
    NotificationSoundPreset {
        id: "aosp-adara",
        label: "AOSP Adara",
        file: "aosp-adara.wav",
    },
    NotificationSoundPreset {
        id: "aosp-altair",
        label: "AOSP Altair",
        file: "aosp-altair.wav",
    },
    NotificationSoundPreset {
        id: "aosp-alya",
        label: "AOSP Alya",
        file: "aosp-alya.wav",
    },
    NotificationSoundPreset {
        id: "aosp-antares",
        label: "AOSP Antares",
        file: "aosp-antares.wav",
    },
    NotificationSoundPreset {
        id: "aosp-antimony",
        label: "AOSP Antimony",
        file: "aosp-antimony.wav",
    },
    NotificationSoundPreset {
        id: "aosp-arcturus",
        label: "AOSP Arcturus",
        file: "aosp-arcturus.wav",
    },
    NotificationSoundPreset {
        id: "aosp-argon",
        label: "AOSP Argon",
        file: "aosp-argon.wav",
    },
    NotificationSoundPreset {
        id: "aosp-ariel",
        label: "AOSP Ariel",
        file: "aosp-ariel.wav",
    },
    NotificationSoundPreset {
        id: "aosp-bellatrix",
        label: "AOSP Bellatrix",
        file: "aosp-bellatrix.wav",
    },
    NotificationSoundPreset {
        id: "aosp-beryllium",
        label: "AOSP Beryllium",
        file: "aosp-beryllium.wav",
    },
    NotificationSoundPreset {
        id: "aosp-betelgeuse",
        label: "AOSP Betelgeuse",
        file: "aosp-betelgeuse.wav",
    },
    NotificationSoundPreset {
        id: "aosp-capella",
        label: "AOSP Capella",
        file: "aosp-capella.wav",
    },
    NotificationSoundPreset {
        id: "aosp-carme",
        label: "AOSP Carme",
        file: "aosp-carme.wav",
    },
    NotificationSoundPreset {
        id: "aosp-ceres",
        label: "AOSP Ceres",
        file: "aosp-ceres.wav",
    },
    NotificationSoundPreset {
        id: "aosp-ceti-alpha",
        label: "AOSP Ceti Alpha",
        file: "aosp-ceti-alpha.wav",
    },
    NotificationSoundPreset {
        id: "aosp-cobalt",
        label: "AOSP Cobalt",
        file: "aosp-cobalt.wav",
    },
    NotificationSoundPreset {
        id: "aosp-deneb",
        label: "AOSP Deneb",
        file: "aosp-deneb.wav",
    },
    NotificationSoundPreset {
        id: "aosp-elara",
        label: "AOSP Elara",
        file: "aosp-elara.wav",
    },
    NotificationSoundPreset {
        id: "aosp-europa",
        label: "AOSP Europa",
        file: "aosp-europa.wav",
    },
    NotificationSoundPreset {
        id: "aosp-fluorine",
        label: "AOSP Fluorine",
        file: "aosp-fluorine.wav",
    },
    NotificationSoundPreset {
        id: "aosp-gallium",
        label: "AOSP Gallium",
        file: "aosp-gallium.wav",
    },
    NotificationSoundPreset {
        id: "aosp-helium",
        label: "AOSP Helium",
        file: "aosp-helium.wav",
    },
    NotificationSoundPreset {
        id: "aosp-hojus",
        label: "AOSP Hojus",
        file: "aosp-hojus.wav",
    },
    NotificationSoundPreset {
        id: "aosp-iapetus",
        label: "AOSP Iapetus",
        file: "aosp-iapetus.wav",
    },
    NotificationSoundPreset {
        id: "aosp-io",
        label: "AOSP Io",
        file: "aosp-io.wav",
    },
    NotificationSoundPreset {
        id: "aosp-iridium",
        label: "AOSP Iridium",
        file: "aosp-iridium.wav",
    },
    NotificationSoundPreset {
        id: "aosp-krypton",
        label: "AOSP Krypton",
        file: "aosp-krypton.wav",
    },
    NotificationSoundPreset {
        id: "aosp-lalande",
        label: "AOSP Lalande",
        file: "aosp-lalande.wav",
    },
    NotificationSoundPreset {
        id: "aosp-mira",
        label: "AOSP Mira",
        file: "aosp-mira.wav",
    },
    NotificationSoundPreset {
        id: "aosp-palladium",
        label: "AOSP Palladium",
        file: "aosp-palladium.wav",
    },
    NotificationSoundPreset {
        id: "aosp-polaris",
        label: "AOSP Polaris",
        file: "aosp-polaris.wav",
    },
    NotificationSoundPreset {
        id: "aosp-pollux",
        label: "AOSP Pollux",
        file: "aosp-pollux.wav",
    },
    NotificationSoundPreset {
        id: "aosp-procyon",
        label: "AOSP Procyon",
        file: "aosp-procyon.wav",
    },
    NotificationSoundPreset {
        id: "aosp-proxima",
        label: "AOSP Proxima",
        file: "aosp-proxima.wav",
    },
    NotificationSoundPreset {
        id: "aosp-radon",
        label: "AOSP Radon",
        file: "aosp-radon.wav",
    },
    NotificationSoundPreset {
        id: "aosp-rhea",
        label: "AOSP Rhea",
        file: "aosp-rhea.wav",
    },
    NotificationSoundPreset {
        id: "aosp-rubidium",
        label: "AOSP Rubidium",
        file: "aosp-rubidium.wav",
    },
    NotificationSoundPreset {
        id: "aosp-salacia",
        label: "AOSP Salacia",
        file: "aosp-salacia.wav",
    },
    NotificationSoundPreset {
        id: "aosp-selenium",
        label: "AOSP Selenium",
        file: "aosp-selenium.wav",
    },
    NotificationSoundPreset {
        id: "aosp-shaula",
        label: "AOSP Shaula",
        file: "aosp-shaula.wav",
    },
    NotificationSoundPreset {
        id: "aosp-spica",
        label: "AOSP Spica",
        file: "aosp-spica.wav",
    },
    NotificationSoundPreset {
        id: "aosp-strontium",
        label: "AOSP Strontium",
        file: "aosp-strontium.wav",
    },
    NotificationSoundPreset {
        id: "aosp-syrma",
        label: "AOSP Syrma",
        file: "aosp-syrma.wav",
    },
    NotificationSoundPreset {
        id: "aosp-talitha",
        label: "AOSP Talitha",
        file: "aosp-talitha.wav",
    },
    NotificationSoundPreset {
        id: "aosp-tejat",
        label: "AOSP Tejat",
        file: "aosp-tejat.wav",
    },
    NotificationSoundPreset {
        id: "aosp-tethys",
        label: "AOSP Tethys",
        file: "aosp-tethys.wav",
    },
    NotificationSoundPreset {
        id: "aosp-thallium",
        label: "AOSP Thallium",
        file: "aosp-thallium.wav",
    },
    NotificationSoundPreset {
        id: "aosp-titan",
        label: "AOSP Titan",
        file: "aosp-titan.wav",
    },
    NotificationSoundPreset {
        id: "aosp-vega",
        label: "AOSP Vega",
        file: "aosp-vega.wav",
    },
];

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

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InputLanguageInfo {
    id: String,
    label: String,
    language_code: String,
    display_code: String,
    locale_name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CurrentLanguageIndicatorStatus {
    enabled: bool,
    mode: CurrentLanguageIndicatorMode,
    current: Option<InputLanguageInfo>,
    installed: Vec<InputLanguageInfo>,
    caret_available: bool,
    source: String,
    message: String,
    debug: CurrentLanguageIndicatorDebug,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CurrentLanguageIndicatorDebug {
    foreground_window: String,
    foreground_thread_id: Option<u32>,
    keyboard_layout: Option<String>,
    ui_automation: String,
    win32_caret: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LanguageIndicatorPayload {
    code: String,
    label: String,
    locale_name: String,
    mode: CurrentLanguageIndicatorMode,
    x: i32,
    y: i32,
    caret_available: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    run_on_startup: bool,
    run_high_priority: bool,
    close_behavior: CloseBehavior,
    enable_tray_icon: bool,
    #[serde(default = "default_native_notification_enabled")]
    native_notification_enabled: bool,
    #[serde(default = "default_dismiss_mirrored_windows_notifications")]
    dismiss_mirrored_windows_notifications: bool,
    #[serde(default = "default_notification_sound_enabled")]
    notification_sound_enabled: bool,
    #[serde(default = "default_notification_sound_preset")]
    notification_sound_preset: String,
    #[serde(default = "default_notification_overlay_placement")]
    notification_overlay_placement: OverlayPlacement,
    #[serde(default = "default_notification_overlay_monitor")]
    notification_overlay_monitor: String,
    #[serde(default = "default_notification_overlay_debug_visible")]
    notification_overlay_debug_visible: bool,
    #[serde(default = "default_notification_overlay_bounds_visible")]
    notification_overlay_bounds_visible: bool,
    #[serde(default)]
    eye_rest_reminder: EyeRestReminderSettings,
    caps_lock_language_switch: CapsLockLanguageSwitchSettings,
    #[serde(default)]
    current_language_indicator: CurrentLanguageIndicatorSettings,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum CloseBehavior {
    MinimizeToTray,
    Exit,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct CurrentLanguageIndicatorSettings {
    enabled: bool,
    #[serde(default = "default_current_language_indicator_mode")]
    mode: CurrentLanguageIndicatorMode,
    #[serde(default = "default_current_language_indicator_placement")]
    placement: OverlayPlacement,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum CurrentLanguageIndicatorMode {
    CaretOverlay,
    ScreenCorner,
    FocusedWindowCorner,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct EyeRestReminderSettings {
    enabled: bool,
    #[serde(default = "default_eye_rest_interval_minutes")]
    interval_minutes: u32,
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
    eye_rest: Mutex<EyeRestState>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            settings: Mutex::new(AppSettings::default()),
            notifications: Mutex::new(Vec::new()),
            notification_capture_status: Mutex::new(NotificationCaptureStatus::default()),
            captured_windows_notification_keys: Mutex::new(HashSet::new()),
            eye_rest: Mutex::new(EyeRestState::default()),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct EyeRestState {
    next_due_at: u64,
    phase: EyeRestPhase,
    rest_started_at: Option<u64>,
    rest_ready_at: Option<u64>,
    completion_sound_played: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
enum EyeRestPhase {
    #[default]
    Idle,
    Prompt,
    Resting,
    Done,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct EyeRestStatus {
    enabled: bool,
    interval_minutes: u32,
    active: bool,
    phase: EyeRestPhase,
    next_due_at: u64,
    rest_started_at: Option<u64>,
    rest_ready_at: Option<u64>,
    now: u64,
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

fn default_native_notification_enabled() -> bool {
    true
}

fn default_dismiss_mirrored_windows_notifications() -> bool {
    false
}

fn default_notification_sound_preset() -> String {
    DEFAULT_NOTIFICATION_SOUND_PRESET.into()
}

fn default_notification_overlay_debug_visible() -> bool {
    false
}

fn default_notification_overlay_bounds_visible() -> bool {
    false
}

fn default_eye_rest_interval_minutes() -> u32 {
    20
}

fn default_current_language_indicator_mode() -> CurrentLanguageIndicatorMode {
    CurrentLanguageIndicatorMode::ScreenCorner
}

fn default_current_language_indicator_placement() -> OverlayPlacement {
    OverlayPlacement::TopRight
}

impl Default for EyeRestReminderSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_minutes: default_eye_rest_interval_minutes(),
        }
    }
}

impl Default for CurrentLanguageIndicatorSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: default_current_language_indicator_mode(),
            placement: default_current_language_indicator_placement(),
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            run_on_startup: false,
            run_high_priority: false,
            close_behavior: CloseBehavior::MinimizeToTray,
            enable_tray_icon: true,
            native_notification_enabled: default_native_notification_enabled(),
            dismiss_mirrored_windows_notifications: default_dismiss_mirrored_windows_notifications(
            ),
            notification_sound_enabled: default_notification_sound_enabled(),
            notification_sound_preset: default_notification_sound_preset(),
            notification_overlay_placement: default_notification_overlay_placement(),
            notification_overlay_monitor: default_notification_overlay_monitor(),
            notification_overlay_debug_visible: default_notification_overlay_debug_visible(),
            notification_overlay_bounds_visible: default_notification_overlay_bounds_visible(),
            eye_rest_reminder: EyeRestReminderSettings::default(),
            caps_lock_language_switch: CapsLockLanguageSwitchSettings {
                enabled: false,
                preserve_caps_lock_with: CapsLockFallbackHotkey::CtrlCaps,
            },
            current_language_indicator: CurrentLanguageIndicatorSettings::default(),
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
fn get_notification_sound_presets() -> Vec<NotificationSoundPreset> {
    NOTIFICATION_SOUND_PRESETS.to_vec()
}

#[tauri::command]
fn preview_notification_sound(app: AppHandle, preset_id: String) -> Result<(), String> {
    notification_sound::play(&app, preset_id.as_str())
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
    language_indicator::apply_settings(&app, &settings.current_language_indicator);

    let previous_eye_rest_settings = {
        let mut current_settings = state.settings.lock().map_err(|error| error.to_string())?;
        let previous = current_settings.eye_rest_reminder.clone();
        *current_settings = settings.clone();
        previous
    };
    if previous_eye_rest_settings != settings.eye_rest_reminder {
        sync_eye_rest_settings(state.inner(), &settings.eye_rest_reminder);
        let _ = hide_eye_rest_overlay(app.clone());
    }
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
fn get_eye_rest_status(state: State<'_, AppState>) -> Result<EyeRestStatus, String> {
    eye_rest_status(state.inner())
}

#[tauri::command]
fn current_language_indicator_status(
    state: State<'_, AppState>,
) -> Result<CurrentLanguageIndicatorStatus, String> {
    let enabled = state
        .settings
        .lock()
        .map(|settings| settings.current_language_indicator.enabled)
        .map_err(|error| error.to_string())?;
    Ok(language_indicator::status(enabled))
}

#[tauri::command]
fn preview_current_language_indicator(app: AppHandle) -> Result<(), String> {
    language_indicator::preview(&app)
}

#[tauri::command]
fn hide_language_indicator_overlay(app: AppHandle) -> Result<(), String> {
    language_indicator::hide(&app);
    Ok(())
}

#[tauri::command]
fn start_eye_rest_timer(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<EyeRestStatus, String> {
    let mut settings = state
        .settings
        .lock()
        .map(|settings| settings.clone())
        .map_err(|error| error.to_string())?;
    settings.eye_rest_reminder.enabled = true;
    settings.eye_rest_reminder.interval_minutes =
        settings.eye_rest_reminder.interval_minutes.clamp(1, 240);
    save_settings(&app, &settings)?;
    {
        let mut current_settings = state.settings.lock().map_err(|error| error.to_string())?;
        *current_settings = settings.clone();
    }
    sync_eye_rest_settings(state.inner(), &settings.eye_rest_reminder);
    hide_eye_rest_overlay(app)?;
    eye_rest_status(state.inner())
}

#[tauri::command]
fn stop_eye_rest_timer(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<EyeRestStatus, String> {
    let mut settings = state
        .settings
        .lock()
        .map(|settings| settings.clone())
        .map_err(|error| error.to_string())?;
    settings.eye_rest_reminder.enabled = false;
    save_settings(&app, &settings)?;
    {
        let mut current_settings = state.settings.lock().map_err(|error| error.to_string())?;
        *current_settings = settings.clone();
    }
    sync_eye_rest_settings(state.inner(), &settings.eye_rest_reminder);
    hide_eye_rest_overlay(app)?;
    eye_rest_status(state.inner())
}

#[tauri::command]
fn skip_eye_rest_timer(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<EyeRestStatus, String> {
    let mut settings = state
        .settings
        .lock()
        .map(|settings| settings.clone())
        .map_err(|error| error.to_string())?;
    settings.eye_rest_reminder.enabled = true;
    save_settings(&app, &settings)?;
    {
        let mut current_settings = state.settings.lock().map_err(|error| error.to_string())?;
        *current_settings = settings.clone();
    }
    open_eye_rest_prompt(&app, state.inner(), true)?;
    eye_rest_status(state.inner())
}

#[tauri::command]
fn start_eye_rest_break(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<EyeRestStatus, String> {
    let now = monotonic_millis();
    {
        let mut eye_rest = state.eye_rest.lock().map_err(|error| error.to_string())?;
        eye_rest.phase = EyeRestPhase::Resting;
        eye_rest.rest_started_at = Some(now);
        eye_rest.rest_ready_at = Some(now + EYE_REST_REST_MILLIS);
        eye_rest.completion_sound_played = false;
    }
    let status = eye_rest_status(state.inner())?;
    app.emit_to("eye-rest", "traybits://eye-rest-updated", status.clone())
        .map_err(|error| error.to_string())?;
    Ok(status)
}

#[tauri::command]
fn complete_eye_rest(app: AppHandle, state: State<'_, AppState>) -> Result<EyeRestStatus, String> {
    let settings = state
        .settings
        .lock()
        .map(|settings| settings.clone())
        .map_err(|error| error.to_string())?;
    let now = monotonic_millis();
    {
        let mut eye_rest = state.eye_rest.lock().map_err(|error| error.to_string())?;
        eye_rest.phase = EyeRestPhase::Idle;
        eye_rest.rest_started_at = None;
        eye_rest.rest_ready_at = None;
        eye_rest.completion_sound_played = false;
        eye_rest.next_due_at = if settings.eye_rest_reminder.enabled {
            now + eye_rest_interval_millis(&settings.eye_rest_reminder)
        } else {
            0
        };
    }
    hide_eye_rest_overlay(app)?;
    eye_rest_status(state.inner())
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

    let native_notification = if settings.native_notification_enabled {
        match show_native_notification(&app, &notification, settings.notification_sound_enabled) {
            Ok(()) => NotificationDeliveryStatus {
                ok: true,
                message: "Native Windows notification sent through WinRT.".into(),
            },
            Err(error) => NotificationDeliveryStatus {
                ok: false,
                message: error,
            },
        }
    } else {
        NotificationDeliveryStatus {
            ok: true,
            message: "Native Windows notification skipped by TrayBits setting.".into(),
        }
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
fn open_notification_source(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let source_app_user_model_id = state
        .notifications
        .lock()
        .map_err(|error| error.to_string())?
        .iter()
        .find(|notification| notification.id == id)
        .and_then(|notification| notification.source_app_user_model_id.clone());

    if let Some(app_user_model_id) = source_app_user_model_id {
        if open_source_app(&app_user_model_id) {
            return Ok(());
        }
    }

    show_main_window(&app);
    Ok(())
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
        let preset_id = state
            .settings
            .lock()
            .ok()
            .map(|settings| settings.notification_sound_preset.clone())
            .unwrap_or_else(default_notification_sound_preset);
        let _ = notification_sound::play(app, preset_id.as_str());
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

    let _ = window.set_ignore_cursor_events(true);
    window.hide().map_err(|error| error.to_string())
}

#[tauri::command]
fn resize_toast_overlay_for_content(app: AppHandle, content_height: f64) -> Result<(), String> {
    resize_toast_overlay(&app, content_height)
}

fn hide_eye_rest_overlay(app: AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("eye-rest") else {
        return Ok(());
    };

    window.hide().map_err(|error| error.to_string())
}

fn normalize_settings(mut settings: AppSettings) -> AppSettings {
    if !settings.enable_tray_icon && settings.close_behavior == CloseBehavior::MinimizeToTray {
        settings.close_behavior = CloseBehavior::Exit;
    }
    if notification_sound_preset(settings.notification_sound_preset.as_str()).is_none() {
        settings.notification_sound_preset = default_notification_sound_preset();
    }
    settings.eye_rest_reminder.interval_minutes =
        settings.eye_rest_reminder.interval_minutes.clamp(1, 240);
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

fn sync_eye_rest_settings(state: &AppState, settings: &EyeRestReminderSettings) {
    let now = monotonic_millis();
    if let Ok(mut eye_rest) = state.eye_rest.lock() {
        if settings.enabled {
            eye_rest.phase = EyeRestPhase::Idle;
            eye_rest.rest_started_at = None;
            eye_rest.rest_ready_at = None;
            eye_rest.completion_sound_played = false;
            eye_rest.next_due_at = now + eye_rest_interval_millis(settings);
        } else {
            eye_rest.next_due_at = 0;
            eye_rest.phase = EyeRestPhase::Idle;
            eye_rest.rest_started_at = None;
            eye_rest.rest_ready_at = None;
            eye_rest.completion_sound_played = false;
        }
    }
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

fn open_source_app(app_user_model_id: &str) -> bool {
    if app_user_model_id.trim().is_empty() {
        return false;
    }

    Command::new("explorer.exe")
        .arg(format!("shell:AppsFolder\\{app_user_model_id}"))
        .spawn()
        .is_ok()
}

fn notification_sound_preset(id: &str) -> Option<&'static NotificationSoundPreset> {
    NOTIFICATION_SOUND_PRESETS
        .iter()
        .find(|preset| preset.id == id)
}

fn default_notification_sound_file() -> &'static str {
    notification_sound_preset(DEFAULT_NOTIFICATION_SOUND_PRESET)
        .map(|preset| preset.file)
        .unwrap_or("aosp-argon.wav")
}

fn eye_rest_interval_millis(settings: &EyeRestReminderSettings) -> u64 {
    settings.interval_minutes.clamp(1, 240) as u64 * 60_000
}

fn eye_rest_status(state: &AppState) -> Result<EyeRestStatus, String> {
    let settings = state
        .settings
        .lock()
        .map(|settings| settings.eye_rest_reminder.clone())
        .map_err(|error| error.to_string())?;
    let eye_rest = state.eye_rest.lock().map_err(|error| error.to_string())?;
    Ok(EyeRestStatus {
        enabled: settings.enabled,
        interval_minutes: settings.interval_minutes,
        active: eye_rest.phase != EyeRestPhase::Idle,
        phase: eye_rest.phase,
        next_due_at: eye_rest.next_due_at,
        rest_started_at: eye_rest.rest_started_at,
        rest_ready_at: eye_rest.rest_ready_at,
        now: monotonic_millis(),
    })
}

fn show_eye_rest_window(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("eye-rest") else {
        return Err("eye-rest window is not configured".into());
    };

    let monitor = app
        .primary_monitor()
        .map_err(|error| error.to_string())?
        .or_else(|| window.current_monitor().ok().flatten());
    if let Some(monitor) = monitor {
        let work_area = monitor.work_area();
        let scale = monitor.scale_factor();
        let logical_width = 380.0 * scale;
        let logical_height = 340.0 * scale;
        let margin = 24.0 * scale;
        let x = work_area.position.x as f64 + work_area.size.width as f64 - logical_width - margin;
        let y =
            work_area.position.y as f64 + work_area.size.height as f64 - logical_height - margin;
        let _ = window.set_size(PhysicalSize::new(
            logical_width.round() as u32,
            logical_height.round() as u32,
        ));
        let _ = window.set_position(PhysicalPosition::new(x as i32, y as i32));
    }

    window
        .set_always_on_top(true)
        .map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    refresh_overlay_topmost(&window)
}

fn open_eye_rest_prompt(app: &AppHandle, state: &AppState, play_sound: bool) -> Result<(), String> {
    {
        let mut eye_rest = state.eye_rest.lock().map_err(|error| error.to_string())?;
        eye_rest.phase = EyeRestPhase::Prompt;
        eye_rest.rest_started_at = None;
        eye_rest.rest_ready_at = None;
        eye_rest.completion_sound_played = false;
    }
    show_eye_rest_window(app)?;
    if play_sound {
        let preset_id = state
            .settings
            .lock()
            .ok()
            .map(|settings| settings.notification_sound_preset.clone())
            .unwrap_or_else(default_notification_sound_preset);
        let _ = notification_sound::play(app, preset_id.as_str());
    }
    let status = eye_rest_status(state)?;
    app.emit_to("eye-rest", "traybits://eye-rest-updated", status)
        .map_err(|error| error.to_string())
}

fn show_toast_window(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("toast") else {
        return Err("toast window is not configured".into());
    };

    if let Some(monitor) = selected_overlay_monitor(app, &window) {
        let work_area = monitor.work_area();
        let scale = monitor.scale_factor();
        let logical_width = 440.0 * scale;
        let settings = app
            .try_state::<AppState>()
            .and_then(|state| state.settings.lock().ok().map(|settings| settings.clone()))
            .unwrap_or_default();
        let logical_height = if settings.notification_overlay_debug_visible {
            520.0 * scale
        } else {
            96.0 * scale
        };
        set_toast_overlay_bounds(
            &window,
            settings.notification_overlay_placement,
            work_area.position.x as f64,
            work_area.position.y as f64,
            work_area.size.width as f64,
            work_area.size.height as f64,
            logical_width,
            logical_height,
            24.0 * scale,
        );
    }

    window
        .set_always_on_top(true)
        .map_err(|error| error.to_string())?;
    let _ = window.set_ignore_cursor_events(false);
    window.show().map_err(|error| error.to_string())?;
    refresh_overlay_topmost(&window)
}

fn resize_toast_overlay(app: &AppHandle, content_height: f64) -> Result<(), String> {
    let Some(window) = app.get_webview_window("toast") else {
        return Ok(());
    };
    let Some(monitor) = selected_overlay_monitor(app, &window) else {
        return Ok(());
    };

    let settings = app
        .try_state::<AppState>()
        .and_then(|state| state.settings.lock().ok().map(|settings| settings.clone()))
        .unwrap_or_default();
    let work_area = monitor.work_area();
    let scale = monitor.scale_factor();
    let margin = 24.0 * scale;
    let logical_width = 440.0 * scale;
    let min_height = if settings.notification_overlay_debug_visible {
        520.0 * scale
    } else {
        72.0 * scale
    };
    let max_height = (work_area.size.height as f64 - margin * 2.0).max(min_height);
    let requested_height = (content_height * scale).ceil();
    let logical_height = requested_height.clamp(min_height, max_height);

    set_toast_overlay_bounds(
        &window,
        settings.notification_overlay_placement,
        work_area.position.x as f64,
        work_area.position.y as f64,
        work_area.size.width as f64,
        work_area.size.height as f64,
        logical_width,
        logical_height,
        margin,
    );
    let _ = window.set_ignore_cursor_events(false);
    window.show().map_err(|error| error.to_string())?;
    refresh_overlay_topmost(&window)
}

fn set_toast_overlay_bounds(
    window: &tauri::WebviewWindow,
    placement: OverlayPlacement,
    screen_x: f64,
    screen_y: f64,
    screen_width: f64,
    screen_height: f64,
    logical_width: f64,
    logical_height: f64,
    margin: f64,
) {
    let (x, y) = overlay_position(
        placement,
        screen_x,
        screen_y,
        screen_width,
        screen_height,
        logical_width,
        logical_height,
        margin,
    );
    let _ = window.set_size(PhysicalSize::new(
        logical_width.round() as u32,
        logical_height.round() as u32,
    ));
    let _ = window.set_position(PhysicalPosition::new(x as i32, y as i32));
}

#[cfg(windows)]
fn refresh_overlay_topmost(window: &tauri::WebviewWindow) -> Result<(), String> {
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
    };

    let hwnd = window.hwnd().map_err(|error| error.to_string())?;
    unsafe {
        SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        )
    }
    .map_err(|error| error.to_string())
}

#[cfg(not(windows))]
fn refresh_overlay_topmost(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Ok(())
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
    use super::{default_notification_sound_file, notification_sound_preset, AppHandle};
    use std::{os::windows::ffi::OsStrExt, path::PathBuf};
    use tauri::Manager;
    use windows::core::PCWSTR;
    use windows::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_NODEFAULT};

    pub fn play(app: &AppHandle, preset_id: &str) -> Result<(), String> {
        let file = notification_sound_preset(preset_id)
            .map(|preset| preset.file)
            .unwrap_or_else(default_notification_sound_file);
        let Some(path) = sound_path(app, file) else {
            return Err(format!("Notification sound file was not found: {file}"));
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
        Ok(())
    }

    fn sound_path(app: &AppHandle, file: &str) -> Option<PathBuf> {
        let mut candidates = Vec::new();

        if let Ok(resource_dir) = app.path().resource_dir() {
            candidates.push(resource_dir.join(file));
            candidates.push(resource_dir.join("assets").join("sounds").join(file));
        }

        if let Ok(current_dir) = std::env::current_dir() {
            candidates.push(current_dir.join("assets").join("sounds").join(file));
            candidates.push(
                current_dir
                    .join("..")
                    .join("assets")
                    .join("sounds")
                    .join(file),
            );
        }

        candidates.into_iter().find(|path| path.is_file())
    }
}

#[cfg(not(target_os = "windows"))]
mod notification_sound {
    use super::AppHandle;

    pub fn play(_app: &AppHandle, _preset_id: &str) -> Result<(), String> {
        Ok(())
    }
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

mod eye_rest_timer {
    use super::{
        eye_rest_interval_millis, eye_rest_status, monotonic_millis, notification_sound,
        open_eye_rest_prompt, AppHandle, AppState, EyeRestPhase,
    };
    use std::{thread, time::Duration};
    use tauri::{Emitter, Manager};

    pub fn start(app: AppHandle) {
        thread::spawn(move || run_timer_loop(app));
    }

    fn run_timer_loop(app: AppHandle) {
        loop {
            let state = app.state::<AppState>();
            let action = {
                let settings = state
                    .settings
                    .lock()
                    .map(|settings| settings.clone())
                    .unwrap_or_default();
                let now = monotonic_millis();
                let mut should_prompt = false;
                let mut should_complete_rest = false;

                if let Ok(mut eye_rest) = state.eye_rest.lock() {
                    if settings.eye_rest_reminder.enabled {
                        if eye_rest.next_due_at == 0 {
                            eye_rest.next_due_at =
                                now + eye_rest_interval_millis(&settings.eye_rest_reminder);
                        }
                        if eye_rest.phase == EyeRestPhase::Idle && now >= eye_rest.next_due_at {
                            should_prompt = true;
                        }
                        if eye_rest.phase == EyeRestPhase::Resting
                            && eye_rest
                                .rest_ready_at
                                .is_some_and(|ready_at| now >= ready_at)
                            && !eye_rest.completion_sound_played
                        {
                            eye_rest.phase = EyeRestPhase::Done;
                            eye_rest.completion_sound_played = true;
                            should_complete_rest = true;
                        }
                    } else {
                        eye_rest.next_due_at = 0;
                        eye_rest.phase = EyeRestPhase::Idle;
                        eye_rest.rest_started_at = None;
                        eye_rest.rest_ready_at = None;
                        eye_rest.completion_sound_played = false;
                    }
                }

                if should_prompt {
                    EyeRestTimerAction::Prompt(settings.notification_sound_enabled)
                } else if should_complete_rest {
                    EyeRestTimerAction::CompleteRest((
                        settings.notification_sound_enabled,
                        settings.notification_sound_preset,
                    ))
                } else {
                    EyeRestTimerAction::None
                }
            };

            match action {
                EyeRestTimerAction::Prompt(sound_enabled) => {
                    let _ = open_eye_rest_prompt(&app, state.inner(), sound_enabled);
                }
                EyeRestTimerAction::CompleteRest((sound_enabled, sound_preset)) => {
                    if sound_enabled {
                        let _ = notification_sound::play(&app, sound_preset.as_str());
                    }
                    if let Ok(status) = eye_rest_status(state.inner()) {
                        let _ = app.emit_to("eye-rest", "traybits://eye-rest-updated", status);
                    }
                }
                EyeRestTimerAction::None => {}
            }

            thread::sleep(Duration::from_millis(1_000));
        }
    }

    enum EyeRestTimerAction {
        None,
        Prompt(bool),
        CompleteRest((bool, String)),
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

    const NOTIFICATION_CAPTURE_POLL_MS: u64 = 250;

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
            thread::sleep(Duration::from_millis(NOTIFICATION_CAPTURE_POLL_MS));
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

            let (sound_enabled, dismiss_after_mirror) = state
                .settings
                .lock()
                .map(|settings| {
                    (
                        settings.notification_sound_enabled,
                        settings.dismiss_mirrored_windows_notifications,
                    )
                })
                .unwrap_or((true, false));
            let _ = add_notification(
                app,
                state,
                app_notification,
                !initial_sync,
                !initial_sync && sound_enabled,
            );
            if !initial_sync && dismiss_after_mirror {
                let _ = listener.RemoveNotification(notification_id);
            }
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
mod language_indicator {
    use super::{
        overlay_position, CurrentLanguageIndicatorDebug, CurrentLanguageIndicatorMode,
        CurrentLanguageIndicatorSettings, CurrentLanguageIndicatorStatus, InputLanguageInfo,
        LanguageIndicatorPayload, OverlayPlacement,
    };
    use std::{
        sync::{Mutex, OnceLock},
        thread,
        time::Duration,
    };
    use tauri::{AppHandle, Emitter, Manager};
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::POINT,
            Globalization::{
                GetLocaleInfoEx, LCIDToLocaleName, LOCALE_ALLOW_NEUTRAL_NAMES,
                LOCALE_SISO639LANGNAME2, LOCALE_SLOCALIZEDDISPLAYNAME,
            },
            Graphics::Gdi::ClientToScreen,
            System::{
                Com::{
                    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
                },
                Ole::{
                    SafeArrayDestroy, SafeArrayGetElement, SafeArrayGetLBound, SafeArrayGetUBound,
                },
            },
            UI::{
                Accessibility::{
                    CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationTextPattern,
                    IUIAutomationTextPattern2, IUIAutomationTextRange, UIA_TextPattern2Id,
                    UIA_TextPatternId,
                },
                Input::KeyboardAndMouse::{GetKeyboardLayout, GetKeyboardLayoutList, HKL},
                WindowsAndMessaging::{
                    GetForegroundWindow, GetGUIThreadInfo, GetWindowRect, GetWindowThreadProcessId,
                    SetWindowPos, ShowWindow, GUITHREADINFO, GUI_CARETBLINKING, HWND_TOPMOST,
                    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SW_SHOWNOACTIVATE,
                },
            },
        },
    };

    const INDICATOR_WINDOW_WIDTH: i32 = 48;
    const INDICATOR_WINDOW_HEIGHT: i32 = 30;

    #[derive(Clone, Copy)]
    struct IndicatorSettings {
        enabled: bool,
        mode: CurrentLanguageIndicatorMode,
        placement: OverlayPlacement,
    }

    impl Default for IndicatorSettings {
        fn default() -> Self {
            Self {
                enabled: false,
                mode: CurrentLanguageIndicatorMode::ScreenCorner,
                placement: OverlayPlacement::TopRight,
            }
        }
    }

    #[derive(Clone)]
    struct IndicatorSnapshot {
        language: InputLanguageInfo,
        mode: CurrentLanguageIndicatorMode,
        x: i32,
        y: i32,
        caret_available: bool,
        signature: String,
    }

    static SETTINGS: OnceLock<Mutex<IndicatorSettings>> = OnceLock::new();
    static MONITOR_STARTED: OnceLock<()> = OnceLock::new();
    const LANGUAGE_INDICATOR_POLL_MS: u64 = 75;

    pub fn apply_settings(app: &AppHandle, settings: &CurrentLanguageIndicatorSettings) {
        let _ = SETTINGS.set(Mutex::new(IndicatorSettings::default()));
        if let Some(lock) = SETTINGS.get() {
            if let Ok(mut current) = lock.lock() {
                current.enabled = settings.enabled;
                current.mode = settings.mode;
                current.placement = settings.placement;
            }
        }

        start(app.clone());

        if settings.enabled {
            let _ = preview(app);
        } else {
            hide(app);
        }
    }

    pub fn status(enabled: bool) -> CurrentLanguageIndicatorStatus {
        let settings = current_settings();
        let mode = settings.mode;
        let current = active_input_language();
        let caret_probe = caret_position_probe();
        let caret = caret_probe.position;
        let caret_available = caret.is_some();
        CurrentLanguageIndicatorStatus {
            enabled,
            mode,
            current,
            installed: installed_languages(),
            caret_available,
            source: caret
                .map(|position| position.source.to_string())
                .unwrap_or_else(|| "No caret provider available".into()),
            message: if enabled {
                "The monitor is enabled. It shows the marker only when TrayBits can read the real caret position.".into()
            } else {
                "The monitor is disabled. Enable it to test the caret language marker.".into()
            },
            debug: CurrentLanguageIndicatorDebug {
                foreground_window: foreground_window_debug(),
                foreground_thread_id: foreground_thread_id(),
                keyboard_layout: active_keyboard_layout_debug(),
                ui_automation: caret_probe.ui_automation,
                win32_caret: caret_probe.win32_caret,
            },
        }
    }

    pub fn preview(app: &AppHandle) -> Result<(), String> {
        let settings = current_settings();
        let snapshot = active_language_snapshot(app, settings)
            .ok_or_else(|| "Could not read the active input language.".to_string())?;
        show(app, &snapshot)
    }

    fn start(app: AppHandle) {
        let _ = MONITOR_STARTED.get_or_init(|| {
            thread::spawn(move || {
                let mut last_signature = String::new();
                loop {
                    if current_settings().enabled {
                        let settings = current_settings();
                        if let Some(snapshot) = active_language_snapshot(&app, settings) {
                            if snapshot.signature != last_signature {
                                last_signature = snapshot.signature.clone();
                                let _ = show(&app, &snapshot);
                            }
                        }
                    }
                    thread::sleep(Duration::from_millis(LANGUAGE_INDICATOR_POLL_MS));
                }
            });
        });
    }

    fn current_settings() -> IndicatorSettings {
        SETTINGS
            .get()
            .and_then(|lock| lock.lock().ok().map(|settings| *settings))
            .unwrap_or_default()
    }

    fn show(app: &AppHandle, snapshot: &IndicatorSnapshot) -> Result<(), String> {
        let Some(window) = app.get_webview_window("language-indicator") else {
            return Err("language-indicator window is not configured".into());
        };
        let payload = LanguageIndicatorPayload {
            code: snapshot.language.display_code.clone(),
            label: snapshot.language.label.clone(),
            locale_name: snapshot.language.locale_name.clone(),
            mode: snapshot.mode,
            x: snapshot.x,
            y: snapshot.y,
            caret_available: snapshot.caret_available,
        };

        let (width, height) = indicator_window_size(snapshot.mode);
        window
            .set_size(tauri::PhysicalSize::new(width as u32, height as u32))
            .map_err(|error| error.to_string())?;
        window
            .set_position(tauri::PhysicalPosition::new(snapshot.x, snapshot.y))
            .map_err(|error| error.to_string())?;
        window
            .set_always_on_top(true)
            .map_err(|error| error.to_string())?;
        let _ = window.set_ignore_cursor_events(true);
        show_language_indicator_without_focus(&window)?;
        app.emit_to(
            "language-indicator",
            "traybits://language-indicator",
            payload,
        )
        .map_err(|error| error.to_string())
    }

    fn show_language_indicator_without_focus(window: &tauri::WebviewWindow) -> Result<(), String> {
        let hwnd = window.hwnd().map_err(|error| error.to_string())?;
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            SetWindowPos(
                hwnd,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
            )
            .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    pub fn hide(app: &AppHandle) {
        if let Some(window) = app.get_webview_window("language-indicator") {
            let _ = window.hide();
        }
    }

    fn active_language_snapshot(
        app: &AppHandle,
        settings: IndicatorSettings,
    ) -> Option<IndicatorSnapshot> {
        let language = active_input_language()?;
        let position = match settings.mode {
            CurrentLanguageIndicatorMode::CaretOverlay => caret_position()?,
            CurrentLanguageIndicatorMode::ScreenCorner => {
                screen_corner_position(app, settings.placement)?
            }
            CurrentLanguageIndicatorMode::FocusedWindowCorner => {
                focused_window_corner_position(settings.placement)
                    .or_else(|| screen_corner_position(app, settings.placement))?
            }
        };
        Some(IndicatorSnapshot {
            signature: format!(
                "{}:{:?}:{}:{}:{}",
                language.id, settings.mode, position.source, position.x, position.y
            ),
            language,
            mode: settings.mode,
            x: position.x,
            y: position.y,
            caret_available: position.caret_available,
        })
    }

    fn screen_corner_position(
        app: &AppHandle,
        placement: OverlayPlacement,
    ) -> Option<IndicatorPosition> {
        let monitor = app.primary_monitor().ok().flatten()?;
        let work_area = monitor.work_area();
        let (width, height) = indicator_window_size(CurrentLanguageIndicatorMode::ScreenCorner);
        let (x, y) = overlay_position(
            placement,
            work_area.position.x as f64,
            work_area.position.y as f64,
            work_area.size.width as f64,
            work_area.size.height as f64,
            width as f64,
            height as f64,
            24.0,
        );
        Some(IndicatorPosition {
            x: x.round() as i32,
            y: y.round() as i32,
            caret_available: true,
            source: "Screen corner",
        })
    }

    fn focused_window_corner_position(placement: OverlayPlacement) -> Option<IndicatorPosition> {
        let foreground_window = unsafe { GetForegroundWindow() };
        if foreground_window.is_invalid() {
            return None;
        }
        let mut rect = Default::default();
        if unsafe { GetWindowRect(foreground_window, &mut rect) }.is_err() {
            return None;
        }
        let (width, height) =
            indicator_window_size(CurrentLanguageIndicatorMode::FocusedWindowCorner);
        let (x, y) = overlay_position(
            placement,
            rect.left as f64,
            rect.top as f64,
            (rect.right - rect.left).max(width) as f64,
            (rect.bottom - rect.top).max(height) as f64,
            width as f64,
            height as f64,
            12.0,
        );
        Some(IndicatorPosition {
            x: x.round() as i32,
            y: y.round() as i32,
            caret_available: true,
            source: "Focused window corner",
        })
    }

    fn indicator_window_size(mode: CurrentLanguageIndicatorMode) -> (i32, i32) {
        match mode {
            CurrentLanguageIndicatorMode::CaretOverlay => {
                (INDICATOR_WINDOW_WIDTH, INDICATOR_WINDOW_HEIGHT)
            }
            CurrentLanguageIndicatorMode::ScreenCorner
            | CurrentLanguageIndicatorMode::FocusedWindowCorner => (116, 62),
        }
    }

    fn active_input_language() -> Option<InputLanguageInfo> {
        let foreground_window = unsafe { GetForegroundWindow() };
        if foreground_window.is_invalid() {
            return None;
        }

        let thread_id = unsafe { GetWindowThreadProcessId(foreground_window, None) };
        let layout = unsafe { GetKeyboardLayout(thread_id) };
        language_from_layout(layout)
    }

    #[derive(Clone, Copy)]
    struct IndicatorPosition {
        x: i32,
        y: i32,
        caret_available: bool,
        source: &'static str,
    }

    fn caret_position() -> Option<IndicatorPosition> {
        caret_position_probe().position
    }

    struct CaretPositionProbe {
        position: Option<IndicatorPosition>,
        ui_automation: String,
        win32_caret: String,
    }

    fn caret_position_probe() -> CaretPositionProbe {
        match uia_caret_position_probe() {
            (Some(position), message) => CaretPositionProbe {
                position: Some(position),
                ui_automation: message,
                win32_caret: "Skipped because UI Automation found an active caret.".into(),
            },
            (None, ui_automation) => {
                let (position, win32_caret) = win32_caret_position_probe();
                CaretPositionProbe {
                    position,
                    ui_automation,
                    win32_caret,
                }
            }
        }
    }

    fn uia_caret_position_probe() -> (Option<IndicatorPosition>, String) {
        unsafe {
            let mut notes = Vec::<String>::new();
            let com_init = CoInitializeEx(None, COINIT_MULTITHREADED);
            if com_init.is_err() {
                notes.push(format!("CoInitializeEx returned {com_init:?}"));
            }
            let automation: IUIAutomation =
                match CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) {
                    Ok(automation) => automation,
                    Err(error) => return (None, format!("CoCreateInstance failed: {error}")),
                };
            let element = match automation.GetFocusedElement() {
                Ok(element) => element,
                Err(error) => return (None, format!("GetFocusedElement failed: {error}")),
            };
            let text_pattern2: Result<IUIAutomationTextPattern2, _> =
                element.GetCurrentPatternAs(UIA_TextPattern2Id);
            let pattern = match text_pattern2 {
                Ok(pattern) => pattern,
                Err(error) => {
                    let (fallback_position, fallback_message) =
                        uia_text_pattern_selection_position(&element);
                    return (
                        fallback_position,
                        format!("TextPattern2 unavailable: {error}. {fallback_message}"),
                    );
                }
            };
            let mut is_active = windows::core::BOOL(0);
            let range = match pattern.GetCaretRange(&mut is_active) {
                Ok(range) => range,
                Err(error) => return (None, format!("GetCaretRange failed: {error}")),
            };
            if !is_active.as_bool() {
                return (None, "TextPattern2 caret range is not active.".into());
            }
            let rectangles = match range.GetBoundingRectangles() {
                Ok(rectangles) => rectangles,
                Err(error) => return (None, format!("GetBoundingRectangles failed: {error}")),
            };
            let Some(first) = first_uia_text_rectangle(rectangles) else {
                return (None, "TextPattern2 returned no caret rectangle.".into());
            };
            let mut message = format!(
                "Active caret rectangle x={} y={} width={} height={}.",
                first.x, first.y, first.width, first.height
            );
            if !notes.is_empty() {
                message.push_str(" Notes: ");
                message.push_str(notes.join("; ").as_str());
            }
            (
                Some(IndicatorPosition {
                    x: indicator_x_from_text_rect(first),
                    y: first.y.round() as i32 + first.height.max(1.0).round() as i32 + 8,
                    caret_available: true,
                    source: "UI Automation TextPattern2",
                }),
                message,
            )
        }
    }

    fn uia_text_pattern_selection_position(
        element: &IUIAutomationElement,
    ) -> (Option<IndicatorPosition>, String) {
        unsafe {
            let pattern: IUIAutomationTextPattern =
                match element.GetCurrentPatternAs(UIA_TextPatternId) {
                    Ok(pattern) => pattern,
                    Err(error) => {
                        return (None, format!("TextPattern unavailable: {error}"));
                    }
                };
            let selection = match pattern.GetSelection() {
                Ok(selection) => selection,
                Err(error) => return (None, format!("TextPattern.GetSelection failed: {error}")),
            };
            let selection_count = match selection.Length() {
                Ok(selection_count) => selection_count,
                Err(error) => {
                    return (
                        None,
                        format!("TextPattern selection length failed: {error}"),
                    )
                }
            };
            if selection_count <= 0 {
                return (None, "TextPattern selection array is empty.".into());
            }
            let range = match selection.GetElement(0) {
                Ok(range) => range,
                Err(error) => {
                    return (
                        None,
                        format!("TextPattern selection range access failed: {error}"),
                    )
                }
            };
            match text_range_position(&range, "UI Automation TextPattern selection") {
                (Some(position), message) => (
                    Some(position),
                    format!("TextPattern selection count={selection_count}. {message}"),
                ),
                (None, message) => (
                    None,
                    format!("TextPattern selection count={selection_count}. {message}"),
                ),
            }
        }
    }

    fn text_range_position(
        range: &IUIAutomationTextRange,
        source: &'static str,
    ) -> (Option<IndicatorPosition>, String) {
        let rectangles = match unsafe { range.GetBoundingRectangles() } {
            Ok(rectangles) => rectangles,
            Err(error) => return (None, format!("GetBoundingRectangles failed: {error}")),
        };
        let Some(first) = first_uia_text_rectangle(rectangles) else {
            return (None, "Text range returned no bounding rectangle.".into());
        };
        (
            Some(IndicatorPosition {
                x: indicator_x_from_text_rect(first),
                y: first.y.round() as i32 + first.height.max(1.0).round() as i32 + 8,
                caret_available: true,
                source,
            }),
            format!(
                "Text range rectangle x={} y={} width={} height={}.",
                first.x, first.y, first.width, first.height
            ),
        )
    }

    #[derive(Clone, Copy)]
    struct UiaTextRectangle {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    }

    fn indicator_x_from_text_rect(rect: UiaTextRectangle) -> i32 {
        let caret_like_width = rect.width.clamp(1.0, 18.0);
        rect.x.round() as i32 + caret_like_width.round() as i32 + 8
    }

    fn first_uia_text_rectangle(
        rectangles: *mut windows::Win32::System::Com::SAFEARRAY,
    ) -> Option<UiaTextRectangle> {
        if rectangles.is_null() {
            return None;
        }

        let result = unsafe {
            let lower = SafeArrayGetLBound(rectangles, 1).ok()?;
            let upper = SafeArrayGetUBound(rectangles, 1).ok()?;
            if upper - lower + 1 < 4 {
                return None;
            }

            let mut values = [0.0_f64; 4];
            for (offset, value) in values.iter_mut().enumerate() {
                let index = lower + offset as i32;
                SafeArrayGetElement(
                    rectangles,
                    &index,
                    value as *mut f64 as *mut core::ffi::c_void,
                )
                .ok()?;
            }

            Some(UiaTextRectangle {
                x: values[0],
                y: values[1],
                width: values[2],
                height: values[3],
            })
        };
        let _ = unsafe { SafeArrayDestroy(rectangles) };
        result
    }

    fn win32_caret_position_probe() -> (Option<IndicatorPosition>, String) {
        let mut info = GUITHREADINFO {
            cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        if let Err(error) = unsafe { GetGUIThreadInfo(0, &mut info) } {
            return (None, format!("GetGUIThreadInfo failed: {error}"));
        }
        if info.hwndCaret.is_invalid() || (info.flags & GUI_CARETBLINKING).0 == 0 {
            return (
                None,
                format!(
                    "No blinking Win32 caret. hwndCaret={:?}, flags=0x{:x}",
                    info.hwndCaret, info.flags.0
                ),
            );
        }

        let mut point = POINT {
            x: info.rcCaret.right + 8,
            y: info.rcCaret.bottom + 8,
        };
        if !unsafe { ClientToScreen(info.hwndCaret, &mut point) }.as_bool() {
            return (None, "ClientToScreen failed for hwndCaret.".into());
        }

        (
            Some(IndicatorPosition {
                x: point.x,
                y: point.y,
                caret_available: true,
                source: "Win32 GetGUIThreadInfo",
            }),
            format!(
                "Blinking caret hwnd={:?}, rect=({}, {}, {}, {}), screen=({}, {}).",
                info.hwndCaret,
                info.rcCaret.left,
                info.rcCaret.top,
                info.rcCaret.right,
                info.rcCaret.bottom,
                point.x,
                point.y
            ),
        )
    }

    fn foreground_window_debug() -> String {
        let foreground_window = unsafe { GetForegroundWindow() };
        if foreground_window.is_invalid() {
            "No foreground window.".into()
        } else {
            format!("{:?}", foreground_window)
        }
    }

    fn foreground_thread_id() -> Option<u32> {
        let foreground_window = unsafe { GetForegroundWindow() };
        if foreground_window.is_invalid() {
            return None;
        }
        let thread_id = unsafe { GetWindowThreadProcessId(foreground_window, None) };
        if thread_id == 0 {
            None
        } else {
            Some(thread_id)
        }
    }

    fn active_keyboard_layout_debug() -> Option<String> {
        let foreground_window = unsafe { GetForegroundWindow() };
        if foreground_window.is_invalid() {
            return None;
        }
        let thread_id = unsafe { GetWindowThreadProcessId(foreground_window, None) };
        if thread_id == 0 {
            return None;
        }
        let layout = unsafe { GetKeyboardLayout(thread_id) };
        Some(format!("0x{:x}", layout.0 as usize))
    }

    fn installed_languages() -> Vec<InputLanguageInfo> {
        let count = unsafe { GetKeyboardLayoutList(None) };
        if count <= 0 {
            return Vec::new();
        }
        let mut layouts = vec![HKL::default(); count as usize];
        let loaded = unsafe { GetKeyboardLayoutList(Some(&mut layouts)) };
        layouts
            .into_iter()
            .take(loaded.max(0) as usize)
            .filter_map(language_from_layout)
            .fold(Vec::<InputLanguageInfo>::new(), |mut acc, language| {
                if !acc.iter().any(|item| item.id == language.id) {
                    acc.push(language);
                }
                acc
            })
    }

    fn language_from_layout(layout: HKL) -> Option<InputLanguageInfo> {
        let lang_id = (layout.0 as u32 & 0xffff) as u32;
        let locale_name =
            locale_name_from_lang_id(lang_id).unwrap_or_else(|| format!("{lang_id:04x}"));
        let iso_code = locale_string(locale_name.as_str(), LOCALE_SISO639LANGNAME2)
            .or_else(|| locale_name.split('-').next().map(|value| value.to_string()))
            .unwrap_or_else(|| "und".into());
        let display_code = iso_code.to_uppercase();
        let label = locale_string(locale_name.as_str(), LOCALE_SLOCALIZEDDISPLAYNAME)
            .unwrap_or_else(|| locale_name.clone());
        Some(InputLanguageInfo {
            id: format!("{:x}", layout.0 as usize),
            label,
            language_code: iso_code,
            display_code,
            locale_name,
        })
    }

    fn locale_name_from_lang_id(lang_id: u32) -> Option<String> {
        let mut buffer = [0u16; 85];
        let length =
            unsafe { LCIDToLocaleName(lang_id, Some(&mut buffer), LOCALE_ALLOW_NEUTRAL_NAMES) };
        if length <= 1 {
            return None;
        }
        Some(String::from_utf16_lossy(&buffer[..length as usize - 1]))
    }

    fn locale_string(locale_name: &str, locale_type: u32) -> Option<String> {
        let locale_wide: Vec<u16> = locale_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut buffer = [0u16; 128];
        let length = unsafe {
            GetLocaleInfoEx(PCWSTR(locale_wide.as_ptr()), locale_type, Some(&mut buffer))
        };
        if length <= 1 {
            return None;
        }
        Some(String::from_utf16_lossy(&buffer[..length as usize - 1]))
    }
}

#[cfg(not(target_os = "windows"))]
mod language_indicator {
    use super::{
        CurrentLanguageIndicatorDebug, CurrentLanguageIndicatorMode,
        CurrentLanguageIndicatorSettings, CurrentLanguageIndicatorStatus, InputLanguageInfo,
    };
    use tauri::AppHandle;

    pub fn apply_settings(_app: &AppHandle, _settings: &CurrentLanguageIndicatorSettings) {}

    pub fn status(enabled: bool) -> CurrentLanguageIndicatorStatus {
        CurrentLanguageIndicatorStatus {
            enabled,
            mode: CurrentLanguageIndicatorMode::ScreenCorner,
            current: None,
            installed: Vec::<InputLanguageInfo>::new(),
            caret_available: false,
            source: "unsupported".into(),
            message: "Current Language Indicator is only implemented on Windows.".into(),
            debug: CurrentLanguageIndicatorDebug {
                foreground_window: "unsupported".into(),
                foreground_thread_id: None,
                keyboard_layout: None,
                ui_automation: "unsupported".into(),
                win32_caret: "unsupported".into(),
            },
        }
    }

    pub fn preview(_app: &AppHandle) -> Result<(), String> {
        Err("Current Language Indicator is only implemented on Windows.".into())
    }

    pub fn hide(_app: &AppHandle) {}
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
        .manage(AppState::default())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let settings = load_settings(app.handle());
            let state = app.state::<AppState>();
            if let Ok(mut current_settings) = state.settings.lock() {
                *current_settings = settings.clone();
            }
            apply_runtime_settings(app.handle(), &settings)?;
            keyboard::apply_settings(&settings.caps_lock_language_switch);
            language_indicator::apply_settings(app.handle(), &settings.current_language_indicator);
            sync_eye_rest_settings(state.inner(), &settings.eye_rest_reminder);
            sync_overlay_debug_visibility(app.handle(), &settings);
            notification_capture::start(app.handle().clone());
            eye_rest_timer::start(app.handle().clone());
            Ok(())
        })
        .on_window_event(handle_window_event)
        .invoke_handler(tauri::generate_handler![
            clear_notifications,
            complete_eye_rest,
            current_language_indicator_status,
            dismiss_notification,
            get_app_settings,
            get_eye_rest_status,
            get_notification_capture_status,
            get_notification_overlay_monitors,
            get_notification_sound_presets,
            get_notifications,
            hide_language_indicator_overlay,
            hide_toast_overlay,
            notification_listener_status,
            open_notification_source,
            preview_current_language_indicator,
            preview_notification_sound,
            push_demo_notification,
            push_demo_toast,
            resize_toast_overlay_for_content,
            skip_eye_rest_timer,
            start_eye_rest_break,
            start_eye_rest_timer,
            stop_eye_rest_timer,
            update_app_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
