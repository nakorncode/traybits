import {
  type Accessor,
  type ParentProps,
  For,
  Show,
  createContext,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
  useContext,
} from "solid-js";
import { A, Navigate, Route, Router, useLocation } from "@solidjs/router";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Toaster, toast } from "solid-sonner";
import "solid-sonner/styles.css";
import "./App.css";

type ToolId = "persistent-notifications" | "eye-rest" | "caps-lock-language-switch" | "settings";

type AppNotification = {
  id: string;
  title: string;
  body: string;
  source: string;
  sourceAppUserModelId?: string;
  origin: "windows" | "demo";
  createdAt: string;
  tone: string;
  silent?: boolean;
};

type NotificationDeliveryStatus = {
  ok: boolean;
  message: string;
};

type PreviewNotificationResult = {
  notification: AppNotification;
  nativeNotification: NotificationDeliveryStatus;
  overlay: NotificationDeliveryStatus;
};

type NotificationListenerStatus = {
  feasible: boolean;
  api: string;
  permissionRequired: boolean;
  packagingRisk: string;
  prototypeStep: string;
};

type CloseBehavior = "minimizeToTray" | "exit";
type CapsLockFallbackHotkey = "ctrlCaps" | "shiftCaps" | "altCaps";
type OverlayPlacement =
  | "topLeft"
  | "topCenter"
  | "topRight"
  | "middleLeft"
  | "center"
  | "middleRight"
  | "bottomLeft"
  | "bottomCenter"
  | "bottomRight";

type NotificationCaptureStatus = {
  enabled: boolean;
  access: string;
  message: string;
  mode: string;
};

type OverlayMonitorOption = {
  id: string;
  label: string;
  isPrimary: boolean;
};

type AppSettings = {
  runOnStartup: boolean;
  runHighPriority: boolean;
  closeBehavior: CloseBehavior;
  enableTrayIcon: boolean;
  notificationSoundEnabled: boolean;
  notificationOverlayPlacement: OverlayPlacement;
  notificationOverlayMonitor: string;
  notificationOverlayDebugVisible: boolean;
  capsLockLanguageSwitch: {
    enabled: boolean;
    preserveCapsLockWith: CapsLockFallbackHotkey;
  };
};

type MainAppContextValue = {
  listenerStatus: Accessor<NotificationListenerStatus | undefined>;
  settings: Accessor<AppSettings | undefined>;
  settingsError: Accessor<string | undefined>;
  notificationError: Accessor<string | undefined>;
  notificationStatus: Accessor<string | undefined>;
  overlayMonitors: Accessor<OverlayMonitorOption[]>;
  notifications: Accessor<AppNotification[]>;
  captureStatus: Accessor<NotificationCaptureStatus | undefined>;
  pushToast: (tone: string) => Promise<void>;
  pushDemoNotification: () => Promise<void>;
  pushOverlaySonnerToast: () => Promise<void>;
  dismissNotification: (id: string) => Promise<void>;
  clearNotificationHistory: () => Promise<void>;
  showMainSonnerToast: () => void;
  updateSettings: (patch: Partial<AppSettings>) => void;
  updateCapsLockSettings: (patch: Partial<AppSettings["capsLockLanguageSwitch"]>) => void;
};

const MainAppContext = createContext<MainAppContextValue>();

const tools: Array<{
  id: ToolId;
  path: string;
  name: string;
  description: string;
  glyph: string;
  status: string;
}> = [
  {
    id: "persistent-notifications",
    path: "/notifications",
    name: "Persistent Notifications",
    description: "Keep Windows notifications visible as desktop cards until dismissed or handled.",
    glyph: "N",
    status: "Prototype",
  },
  {
    id: "eye-rest",
    path: "/eye-rest",
    name: "Eye Rest Reminder",
    description: "Show periodic 20-20-20 reminders for long desktop sessions.",
    glyph: "20",
    status: "Planned",
  },
  {
    id: "caps-lock-language-switch",
    path: "/caps-lock-language-switch",
    name: "Caps Lock Language Switch",
    description: "Use Caps Lock as a quick input-language switch while preserving clear lock behavior.",
    glyph: "C",
    status: "Spike",
  },
  {
    id: "settings",
    path: "/settings",
    name: "Settings",
    description: "Suite-wide startup, privacy, and release controls.",
    glyph: "S",
    status: "Draft",
  },
];

const overlayPlacements: Array<{
  id: OverlayPlacement;
  label: string;
  shortLabel: string;
}> = [
  { id: "topLeft", label: "Top left", shortLabel: "TL" },
  { id: "topCenter", label: "Top center", shortLabel: "TC" },
  { id: "topRight", label: "Top right", shortLabel: "TR" },
  { id: "middleLeft", label: "Middle left", shortLabel: "ML" },
  { id: "center", label: "Center", shortLabel: "C" },
  { id: "middleRight", label: "Middle right", shortLabel: "MR" },
  { id: "bottomLeft", label: "Bottom left", shortLabel: "BL" },
  { id: "bottomCenter", label: "Bottom center", shortLabel: "BC" },
  { id: "bottomRight", label: "Bottom right", shortLabel: "BR" },
];

function formatTimestamp(createdAt: string) {
  const millis = Number(createdAt);
  if (!Number.isFinite(millis)) return createdAt;

  return new Date(millis).toLocaleString();
}

function App() {
  const params = new URLSearchParams(window.location.search);
  const view = params.get("view");

  if (view === "toast") {
    document.documentElement.dataset.view = "toast";
    return <ToastOverlay />;
  }

  delete document.documentElement.dataset.view;
  return (
    <Router root={MainApp}>
      <Route path="/" component={() => <Navigate href="/notifications" />} />
      <Route path="/notifications" component={PersistentNotificationsRoute} />
      <Route path="/eye-rest" component={EyeRestRoute} />
      <Route path="/caps-lock-language-switch" component={CapsLockLanguageSwitchRoute} />
      <Route path="/settings" component={SettingsRoute} />
    </Router>
  );
}

function useMainApp() {
  const context = useContext(MainAppContext);
  if (!context) {
    throw new Error("MainApp context is missing.");
  }
  return context;
}

function MainApp(props: ParentProps) {
  const location = useLocation();
  const [listenerStatus, setListenerStatus] = createSignal<NotificationListenerStatus>();
  const [settings, setSettings] = createSignal<AppSettings>();
  const [settingsError, setSettingsError] = createSignal<string>();
  const [notificationError, setNotificationError] = createSignal<string>();
  const [notificationStatus, setNotificationStatus] = createSignal<string>();
  const [overlayMonitors, setOverlayMonitors] = createSignal<OverlayMonitorOption[]>([]);
  const [notifications, setNotifications] = createSignal<AppNotification[]>([]);
  const [captureStatus, setCaptureStatus] = createSignal<NotificationCaptureStatus>();

  onMount(async () => {
    const [listener, appSettings, notificationItems, status, monitors] = await Promise.all([
      invoke<NotificationListenerStatus>("notification_listener_status"),
      invoke<AppSettings>("get_app_settings"),
      invoke<AppNotification[]>("get_notifications"),
      invoke<NotificationCaptureStatus>("get_notification_capture_status"),
      invoke<OverlayMonitorOption[]>("get_notification_overlay_monitors"),
    ]);
    setListenerStatus(listener);
    setSettings(appSettings);
    setNotifications(notificationItems);
    setCaptureStatus(status);
    setOverlayMonitors(monitors);

    const unlistenAdded = await listen<AppNotification>("traybits://notification-added", (event) => {
      setNotifications((items) => [event.payload, ...items.filter((item) => item.id !== event.payload.id)]);
      if (!event.payload.silent && event.payload.origin === "windows") {
        toast.info(event.payload.title, {
          description: `${event.payload.source}: ${event.payload.body}`,
        });
      }
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

  const active = createMemo(
    () => tools.find((tool) => tool.path === location.pathname) ?? tools[0],
  );

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
      const status = `Windows: ${result.nativeNotification.message} Overlay: ${result.overlay.message}`;
      setNotificationStatus(status);
      if (result.nativeNotification.ok && result.overlay.ok) {
        toast.success("Preview notification requested", { description: status });
      } else {
        toast.warning("Preview notification has a delivery issue", { description: status });
      }
      await refreshCaptureStatus();
    } catch (error) {
      setNotificationError(error instanceof Error ? error.message : String(error));
    }
  }

  async function pushOverlaySonnerToast() {
    setNotificationError(undefined);
    setNotificationStatus(undefined);
    try {
      const notification = await invoke<AppNotification>("push_overlay_debug_notification");
      setNotifications((items) => [
        notification,
        ...items.filter((item) => item.id !== notification.id),
      ]);
      setNotificationStatus("Overlay debug notification was added to the shared store.");
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

  function showMainSonnerToast() {
    toast.success("Solid Sonner rendered in the main window", {
      description: "This proves browser-window toast rendering works separately from the transparent overlay.",
    });
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

  const context: MainAppContextValue = {
    listenerStatus,
    settings,
    settingsError,
    notificationError,
    notificationStatus,
    overlayMonitors,
    notifications,
    captureStatus,
    pushToast,
    pushDemoNotification,
    pushOverlaySonnerToast,
    dismissNotification,
    clearNotificationHistory,
    showMainSonnerToast,
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
                <A activeClass="selected" href={tool.path} end>
                  <span class="nav-glyph">{tool.glyph}</span>
                  <span>
                    <strong>{tool.name}</strong>
                    <small>{tool.status}</small>
                  </span>
                </A>
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
          <Toaster position="top-right" richColors closeButton expand visibleToasts={5} />
        </section>
      </main>
    </MainAppContext.Provider>
  );
}

function PersistentNotificationsRoute() {
  const app = useMainApp();
  return (
    <PersistentNotificationsPanel
      captureStatus={app.captureStatus()}
      listenerStatus={app.listenerStatus()}
      notifications={app.notifications()}
      overlayMonitors={app.overlayMonitors()}
      pushDemoNotification={app.pushDemoNotification}
      pushOverlaySonnerToast={app.pushOverlaySonnerToast}
      clearNotifications={app.clearNotificationHistory}
      dismissNotification={app.dismissNotification}
      showMainSonnerToast={app.showMainSonnerToast}
      settings={app.settings()}
      updateSettings={app.updateSettings}
      settingsError={app.settingsError()}
      notificationError={app.notificationError()}
      notificationStatus={app.notificationStatus()}
    />
  );
}

function EyeRestRoute() {
  const app = useMainApp();
  return <EyeRestPanel pushToast={app.pushToast} />;
}

function CapsLockLanguageSwitchRoute() {
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

function SettingsRoute() {
  const app = useMainApp();
  return (
    <SettingsPanel
      settings={app.settings()}
      updateSettings={app.updateSettings}
      settingsError={app.settingsError()}
    />
  );
}

function PersistentNotificationsPanel(props: {
  captureStatus?: NotificationCaptureStatus;
  listenerStatus?: NotificationListenerStatus;
  notifications: AppNotification[];
  overlayMonitors: OverlayMonitorOption[];
  pushDemoNotification: () => Promise<void>;
  pushOverlaySonnerToast: () => Promise<void>;
  clearNotifications: () => Promise<void>;
  dismissNotification: (id: string) => Promise<void>;
  showMainSonnerToast: () => void;
  settings?: AppSettings;
  updateSettings: (patch: Partial<AppSettings>) => void;
  settingsError?: string;
  notificationError?: string;
  notificationStatus?: string;
}) {
  return (
    <div class="content-grid">
      <section class="panel primary-panel">
        <div class="section-title">
          <span>Windows capture</span>
          <strong>{props.captureStatus?.enabled ? "Capturing notifications" : "Waiting for access"}</strong>
        </div>
        <p>
          TrayBits polls Windows notifications in the background and mirrors new readable toasts into
          this local history and the persistent overlay.
        </p>
        <dl class="fact-list compact-facts">
          <div>
            <dt>Access</dt>
            <dd>{props.captureStatus?.access ?? "Loading..."}</dd>
          </div>
          <div>
            <dt>Mode</dt>
            <dd>{props.captureStatus?.mode ?? "Loading..."}</dd>
          </div>
          <div>
            <dt>Status</dt>
            <dd>{props.captureStatus?.message ?? "Starting Windows notification capture..."}</dd>
          </div>
          <div>
            <dt>Sound</dt>
            <dd>Short interface tone plays when a persistent notification is added.</dd>
          </div>
        </dl>
        <div class="button-row">
          <button type="button" onClick={props.pushDemoNotification}>
            Run preview demo notification
          </button>
          <button type="button" onClick={props.showMainSonnerToast}>
            Run main-window sonner toast
          </button>
          <button type="button" onClick={props.pushOverlaySonnerToast}>
            Run overlay-window sonner toast
          </button>
          <button type="button" onClick={props.clearNotifications}>
            Clear history
          </button>
        </div>
      </section>

      <section class="panel">
        <div class="section-title">
          <span>Overlay position</span>
          <strong>Transparent window placement</strong>
        </div>
        <div class="placement-grid" role="group" aria-label="Overlay placement">
          <For each={overlayPlacements}>
            {(placement) => (
              <button
                type="button"
                classList={{ selected: props.settings?.notificationOverlayPlacement === placement.id }}
                disabled={!props.settings}
                onClick={() => props.updateSettings({ notificationOverlayPlacement: placement.id })}
                title={placement.label}
              >
                {placement.shortLabel}
              </button>
            )}
          </For>
        </div>
        <label class="field-row overlay-monitor-field">
          <span>Target screen</span>
          <select
            value={props.settings?.notificationOverlayMonitor ?? "primary"}
            disabled={!props.settings || props.overlayMonitors.length === 0}
            onChange={(event) =>
              props.updateSettings({ notificationOverlayMonitor: event.currentTarget.value })
            }
          >
            <For each={props.overlayMonitors}>
              {(monitor) => (
                <option value={monitor.id}>
                  {monitor.label}
                  {monitor.isPrimary && monitor.id !== "primary" ? " (primary)" : ""}
                </option>
              )}
            </For>
          </select>
        </label>
        <label class="toggle-row">
          <input
            type="checkbox"
            checked={props.settings?.notificationSoundEnabled ?? true}
            disabled={!props.settings}
            onChange={(event) =>
              props.updateSettings({ notificationSoundEnabled: event.currentTarget.checked })
            }
          />
          Enable notification sound
        </label>
        <label class="toggle-row">
          <input
            type="checkbox"
            checked={props.settings?.notificationOverlayDebugVisible ?? true}
            disabled={!props.settings}
            onChange={(event) =>
              props.updateSettings({ notificationOverlayDebugVisible: event.currentTarget.checked })
            }
          />
          Show overlay debug background
        </label>
        <Show when={props.settingsError}>
          <p class="error-text">{props.settingsError}</p>
        </Show>
        <Show when={props.notificationError}>
          <p class="error-text">{props.notificationError}</p>
        </Show>
        <Show when={props.notificationStatus}>
          <p class="hint-text">{props.notificationStatus}</p>
        </Show>
      </section>

      <section class="panel wide-panel">
        <div class="section-title">
          <span>History</span>
          <strong>Windows notifications seen by TrayBits</strong>
        </div>
        <Show
          when={props.notifications.length > 0}
          fallback={<p class="empty-text">No notifications captured yet. Run a preview or trigger a Windows toast.</p>}
        >
          <div class="notification-list">
            <For each={props.notifications}>
              {(notification) => (
                <article class="notification-row">
                  <div>
                    <span>{notification.source}</span>
                    <strong>{notification.title}</strong>
                    <p>{notification.body}</p>
                    <small>
                      {notification.origin} · {formatTimestamp(notification.createdAt)}
                    </small>
                  </div>
                  <button
                    type="button"
                    aria-label={`Dismiss ${notification.title}`}
                    onClick={() => props.dismissNotification(notification.id)}
                  >
                    Dismiss
                  </button>
                </article>
              )}
            </For>
          </div>
        </Show>
      </section>
    </div>
  );
}

function EyeRestPanel(props: { pushToast: (tone: string) => Promise<void> }) {
  return (
    <section class="panel primary-panel">
      <div class="section-title">
        <span>Reminder preview</span>
        <strong>20-20-20 reminder flow</strong>
      </div>
      <p>
        This utility can stay mostly Rust-owned: a timer decides when to notify, then emits a
        toast-render event to the Solid overlay.
      </p>
      <button type="button" onClick={() => props.pushToast("rest")}>
        Preview reminder toast
      </button>
    </section>
  );
}

function CapsLockLanguageSwitchPanel(props: {
  pushToast: (tone: string) => Promise<void>;
  settings?: AppSettings["capsLockLanguageSwitch"];
  updateSettings: (patch: Partial<AppSettings["capsLockLanguageSwitch"]>) => void;
  settingsError?: string;
}) {
  return (
    <div class="content-grid">
      <section class="panel primary-panel">
        <div class="section-title">
          <span>Keyboard hook</span>
          <strong>Caps Lock to input-language switch</strong>
        </div>
        <p>
          When enabled, Caps Lock alone is intercepted by Rust, switches to the next Windows input
          language, and does not toggle Caps Lock state.
        </p>
        <div class="settings-list">
          <label class="toggle-row">
            <input
              type="checkbox"
              checked={props.settings?.enabled ?? false}
              disabled={!props.settings}
              onChange={(event) => props.updateSettings({ enabled: event.currentTarget.checked })}
            />
            Enable Caps Lock Language Switch
          </label>
          <label class="field-row">
            <span>Original Caps Lock behavior</span>
            <select
              value={props.settings?.preserveCapsLockWith ?? "ctrlCaps"}
              disabled={!props.settings}
              onChange={(event) =>
                props.updateSettings({
                  preserveCapsLockWith: event.currentTarget.value as CapsLockFallbackHotkey,
                })
              }
            >
              <option value="ctrlCaps">Ctrl + Caps Lock</option>
              <option value="shiftCaps">Shift + Caps Lock</option>
              <option value="altCaps">Alt + Caps Lock</option>
            </select>
          </label>
        </div>
        <Show when={props.settingsError}>
          <p class="error-text">{props.settingsError}</p>
        </Show>
        <button type="button" onClick={() => props.pushToast("language-switch")}>
          Preview language switch toast
        </button>
      </section>

      <section class="panel">
        <div class="section-title">
          <span>Behavior</span>
          <strong>Fixed shortcuts for now</strong>
        </div>
        <dl class="fact-list">
          <div>
            <dt>Caps Lock</dt>
            <dd>Switches to the next installed Windows input language.</dd>
          </div>
          <div>
            <dt>Caps Lock state</dt>
            <dd>Blocked for Caps Lock alone so accidental uppercase mode is disabled.</dd>
          </div>
          <div>
            <dt>Fallback shortcut</dt>
            <dd>The selected modifier combo passes through to Windows for normal Caps Lock.</dd>
          </div>
        </dl>
      </section>
    </div>
  );
}

function SettingsPanel(props: {
  settings?: AppSettings;
  updateSettings: (patch: Partial<AppSettings>) => void;
  settingsError?: string;
}) {
  return (
    <section class="panel primary-panel">
      <div class="section-title">
        <span>Suite settings</span>
        <strong>Shared desktop behavior</strong>
      </div>
      <div class="settings-list">
        <label class="toggle-row">
          <input
            type="checkbox"
            checked={props.settings?.runOnStartup ?? false}
            disabled={!props.settings}
            onChange={(event) => props.updateSettings({ runOnStartup: event.currentTarget.checked })}
          />
          Run on startup
        </label>
        <label class="toggle-row">
          <input
            type="checkbox"
            checked={props.settings?.runHighPriority ?? false}
            disabled={!props.settings}
            onChange={(event) =>
              props.updateSettings({ runHighPriority: event.currentTarget.checked })
            }
          />
          Run high priority
        </label>
        <label class="toggle-row">
          <input
            type="checkbox"
            checked={props.settings?.enableTrayIcon ?? false}
            disabled={!props.settings}
            onChange={(event) =>
              props.updateSettings({ enableTrayIcon: event.currentTarget.checked })
            }
          />
          Enable tray icon
        </label>
        <label class="field-row">
          <span>When closing the window</span>
          <select
            value={props.settings?.closeBehavior ?? "minimizeToTray"}
            disabled={!props.settings}
            onChange={(event) =>
              props.updateSettings({ closeBehavior: event.currentTarget.value as CloseBehavior })
            }
          >
            <option value="minimizeToTray">Minimize to tray</option>
            <option value="exit">Exit program</option>
          </select>
        </label>
      </div>
      <p class="hint-text">
        If the tray icon is disabled, closing the window exits the app so it cannot disappear in the
        background.
      </p>
      <Show when={props.settingsError}>
        <p class="error-text">{props.settingsError}</p>
      </Show>
    </section>
  );
}

function ToastOverlay() {
  const [toasts, setToasts] = createSignal<AppNotification[]>([]);
  const [settings, setSettings] = createSignal<AppSettings>();
  const [lastPollAt, setLastPollAt] = createSignal<string>();
  const [lastPollError, setLastPollError] = createSignal<string>();
  const [windowApiError, setWindowApiError] = createSignal<string>();
  const announcedIds = new Set<string>();
  let initialSyncDone = false;

  onMount(() => {
    void syncOverlayState();
    const poll = window.setInterval(syncOverlayState, 750);
    void getCurrentWindow().setAlwaysOnTop(true).catch((error) => {
      setWindowApiError(error instanceof Error ? error.message : String(error));
    });

    onCleanup(() => {
      window.clearInterval(poll);
    });
  });

  async function syncOverlayState() {
    try {
      const [latest, appSettings] = await Promise.all([
        invoke<AppNotification[]>("get_notifications"),
        invoke<AppSettings>("get_app_settings"),
      ]);
      const visible = latest.filter((notification) => !notification.silent).slice(0, 4);
      setSettings(appSettings);
      setLastPollAt(new Date().toLocaleTimeString());
      setLastPollError(undefined);

      if (!initialSyncDone) {
        for (const notification of visible) {
          announcedIds.add(notification.id);
        }
        initialSyncDone = true;
      } else {
        for (const notification of visible) {
          showOverlaySonner(notification);
        }
      }

      setToasts(visible);
      if (visible.length === 0) {
        hideWhenEmpty();
      }
    } catch (error) {
      setLastPollError(error instanceof Error ? error.message : String(error));
    }
  }

  function showOverlaySonner(notification: AppNotification) {
    if (announcedIds.has(notification.id)) return;
    announcedIds.add(notification.id);
    toast.info(notification.title, {
      toasterId: "overlay",
      description: `${notification.source}: ${notification.body}`,
      duration: 8000,
    });
  }

  function dismiss(id: string) {
    invoke("dismiss_notification", { id }).catch(() => undefined);
    setToasts((items) => items.filter((toast) => toast.id !== id));
    hideWhenEmpty();
  }

  function hideWhenEmpty() {
    window.setTimeout(() => {
      if (toasts().length === 0 && !settings()?.notificationOverlayDebugVisible) {
        invoke("hide_toast_overlay").catch(() => undefined);
      }
    }, 180);
  }

  return (
    <div
      class="toast-stage"
      classList={{ "debug-overlay": settings()?.notificationOverlayDebugVisible ?? true }}
    >
      <Show when={settings()?.notificationOverlayDebugVisible ?? true}>
        <div class="overlay-debug-card">
          <strong>TrayBits overlay debug</strong>
          <span>Transparent overlay window is visible.</span>
          <span>Store polling is active{lastPollAt() ? `, last checked ${lastPollAt()}` : ""}.</span>
          <span>Visible notifications in overlay store: {toasts().length}</span>
          <Show when={toasts()[0]}>
            {(notification) => <span>Newest: {notification().title}</span>}
          </Show>
          <Show when={lastPollError()}>
            {(error) => <span class="overlay-debug-error">Poll error: {error()}</span>}
          </Show>
          <Show when={windowApiError()}>
            {(error) => <span class="overlay-debug-error">Window API error: {error()}</span>}
          </Show>
        </div>
      </Show>
      <Toaster
        id="overlay"
        position="top-right"
        richColors
        closeButton
        expand
        visibleToasts={4}
      />
      <For each={toasts()}>
        {(toast) => (
          <article class={`toast-card tone-${toast.tone}`}>
            <div class="toast-icon">{toast.source.slice(0, 1)}</div>
            <div class="toast-copy">
              <span>{toast.source}</span>
              <strong>{toast.title}</strong>
              <p>{toast.body}</p>
              <small>{formatTimestamp(toast.createdAt)}</small>
            </div>
            <button type="button" aria-label="Dismiss toast" onClick={() => dismiss(toast.id)}>
              ×
            </button>
          </article>
        )}
      </For>
    </div>
  );
}

export default App;
