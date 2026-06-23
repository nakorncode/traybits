import { type ParentProps, For, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { useLocation, useNavigate } from "@solidjs/router";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { tools } from "./constants";
import { MainAppContext, useMainApp } from "./app-context";
import { PersistentNotificationsPanel } from "./pages/PersistentNotifications";
import { EyeRestPanel } from "./pages/EyeRest";
import { CapsLockLanguageSwitchPanel } from "./pages/CapsLockLanguageSwitch";
import { CurrentLanguageIndicatorPanel } from "./pages/CurrentLanguageIndicator";
import { SettingsPanel } from "./pages/Settings";
import type { AppNotification, AppSettings, NotificationCaptureStatus, NotificationListenerStatus, NotificationSoundPreset, PreviewNotificationResult, OverlayMonitorOption } from "./types";

export function MainApp(props: ParentProps) {
  const location = useLocation();
  const navigate = useNavigate();
  const [listenerStatus, setListenerStatus] = createSignal<NotificationListenerStatus>();
  const [settings, setSettings] = createSignal<AppSettings>();
  const [settingsError, setSettingsError] = createSignal<string>();
  const [notificationError, setNotificationError] = createSignal<string>();
  const [notificationStatus, setNotificationStatus] = createSignal<string>();
  const [notificationSoundPresets, setNotificationSoundPresets] = createSignal<NotificationSoundPreset[]>([]);
  const [overlayMonitors, setOverlayMonitors] = createSignal<OverlayMonitorOption[]>([]);
  const [notifications, setNotifications] = createSignal<AppNotification[]>([]);
  const [captureStatus, setCaptureStatus] = createSignal<NotificationCaptureStatus>();

  onMount(async () => {
    const [listener, appSettings, notificationItems, status, monitors, soundPresets] = await Promise.all([
      invoke<NotificationListenerStatus>("notification_listener_status"),
      invoke<AppSettings>("get_app_settings"),
      invoke<AppNotification[]>("get_notifications"),
      invoke<NotificationCaptureStatus>("get_notification_capture_status"),
      invoke<OverlayMonitorOption[]>("get_notification_overlay_monitors"),
      invoke<NotificationSoundPreset[]>("get_notification_sound_presets"),
    ]);
    setListenerStatus(listener);
    setSettings(appSettings);
    setNotifications(notificationItems);
    setCaptureStatus(status);
    setOverlayMonitors(monitors);
    setNotificationSoundPresets(soundPresets);

    const unlistenAdded = await listen<AppNotification>("traybits://notification-added", (event) => {
      setNotifications((items) => [event.payload, ...items.filter((item) => item.id !== event.payload.id)]);
      void refreshCaptureStatus();
    });
    const unlistenDismissed = await listen<string>("traybits://notification-dismissed", (event) => {
      setNotifications((items) => items.filter((item) => item.id !== event.payload));
    });
    const unlistenCleared = await listen("traybits://notifications-cleared", () => {
      setNotifications([]);
    });

    onCleanup(() => {
      unlistenAdded();
      unlistenDismissed();
      unlistenCleared();
    });
  });

  const active = createMemo(() => tools.find((tool) => tool.path === location.pathname) ?? tools[0]);

  async function pushToast(tone: string) {
    await invoke("push_demo_toast", { tone });
  }

  async function pushDemoNotification() {
    setNotificationError(undefined);
    setNotificationStatus(undefined);
    try {
      const result = await invoke<PreviewNotificationResult>("push_demo_notification");
      setNotifications((items) => [
        result.notification,
        ...items.filter((item) => item.id !== result.notification.id),
      ]);
      const status = "Windows: " + result.nativeNotification.message + " Overlay: " + result.overlay.message;
      setNotificationStatus(status);
      await refreshCaptureStatus();
    } catch (error) {
      setNotificationError(error instanceof Error ? error.message : String(error));
    }
  }

  async function dismissNotification(id: string) {
    await invoke("dismiss_notification", { id });
    setNotifications((items) => items.filter((item) => item.id !== id));
  }

  async function clearNotificationHistory() {
    await invoke("clear_notifications");
    setNotifications([]);
  }

  async function refreshCaptureStatus() {
    setCaptureStatus(await invoke<NotificationCaptureStatus>("get_notification_capture_status"));
  }

  async function saveSettings(next: AppSettings) {
    setSettingsError(undefined);
    try {
      const saved = await invoke<AppSettings>("update_app_settings", { settings: next });
      setSettings(saved);
    } catch (error) {
      setSettingsError(error instanceof Error ? error.message : String(error));
    }
  }

  function updateSettings(patch: Partial<AppSettings>) {
    const current = settings();
    if (!current) return;
    void saveSettings({ ...current, ...patch });
  }

  function updateCapsLockSettings(patch: Partial<AppSettings["capsLockLanguageSwitch"]>) {
    const current = settings();
    if (!current) return;
    void saveSettings({
      ...current,
      capsLockLanguageSwitch: {
        ...current.capsLockLanguageSwitch,
        ...patch,
      },
    });
  }

  const context = {
    listenerStatus,
    settings,
    settingsError,
    notificationError,
    notificationStatus,
    notificationSoundPresets,
    overlayMonitors,
    notifications,
    captureStatus,
    pushToast,
    pushDemoNotification,
    dismissNotification,
    clearNotificationHistory,
    updateSettings,
    updateCapsLockSettings,
  };

  return (
    <MainAppContext.Provider value={context}>
      <main class="app-shell">
        <aside class="sidebar">
          <div class="brand">
            <div class="brand-mark">TB</div>
            <div>
              <strong>TrayBits</strong>
              <span>Windows utilities</span>
            </div>
          </div>

          <nav class="tool-nav" aria-label="Utilities">
            <For each={tools}>
              {(tool) => (
                <button
                  type="button"
                  classList={{ selected: location.pathname === tool.path }}
                  onClick={() => navigate(tool.path)}
                >
                  <span class="nav-glyph">{tool.glyph}</span>
                  <span>
                    <strong>{tool.name}</strong>
                    <small>{tool.status}</small>
                  </span>
                </button>
              )}
            </For>
          </nav>
        </aside>

        <section class="workspace">
          <header class="topbar">
            <div>
              <h1>{active().name}</h1>
              <p>{active().description}</p>
            </div>
            <button class="quiet-button" type="button" onClick={pushDemoNotification}>
              Preview notification
            </button>
          </header>

          {props.children}
        </section>
      </main>
    </MainAppContext.Provider>
  );
}

export function PersistentNotificationsRoute() {
  const app = useMainApp();
  return (
    <PersistentNotificationsPanel
      captureStatus={app.captureStatus()}
      listenerStatus={app.listenerStatus()}
      notifications={app.notifications()}
      notificationSoundPresets={app.notificationSoundPresets()}
      overlayMonitors={app.overlayMonitors()}
      pushDemoNotification={app.pushDemoNotification}
      clearNotifications={app.clearNotificationHistory}
      dismissNotification={app.dismissNotification}
      settings={app.settings()}
      updateSettings={app.updateSettings}
      settingsError={app.settingsError()}
      notificationError={app.notificationError()}
      notificationStatus={app.notificationStatus()}
    />
  );
}

export function EyeRestRoute() {
  const app = useMainApp();
  return (
    <EyeRestPanel
      settings={app.settings()?.eyeRestReminder}
      updateSettings={(patch) =>
        app.updateSettings({
          eyeRestReminder: {
            enabled: app.settings()?.eyeRestReminder.enabled ?? false,
            intervalMinutes: app.settings()?.eyeRestReminder.intervalMinutes ?? 20,
            ...patch,
          },
        })
      }
      settingsError={app.settingsError()}
    />
  );
}

export function CapsLockLanguageSwitchRoute() {
  const app = useMainApp();
  return (
    <CapsLockLanguageSwitchPanel
      pushToast={app.pushToast}
      settings={app.settings()?.capsLockLanguageSwitch}
      updateSettings={app.updateCapsLockSettings}
      settingsError={app.settingsError()}
    />
  );
}

export function CurrentLanguageIndicatorRoute() {
  const app = useMainApp();
  return (
    <CurrentLanguageIndicatorPanel
      settings={app.settings()?.currentLanguageIndicator}
      updateSettings={(patch) =>
        app.updateSettings({
          currentLanguageIndicator: {
            enabled: app.settings()?.currentLanguageIndicator.enabled ?? false,
            ...patch,
          },
        })
      }
      settingsError={app.settingsError()}
    />
  );
}

export function SettingsRoute() {
  const app = useMainApp();
  return (
    <SettingsPanel
      settings={app.settings()}
      updateSettings={app.updateSettings}
      settingsError={app.settingsError()}
    />
  );
}
