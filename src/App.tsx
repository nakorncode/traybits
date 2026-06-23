import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { A, Navigate, Route, Router, useLocation } from "@solidjs/router";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";

type ToolId = "persistent-notifications" | "eye-rest" | "caps-lock-language-switch" | "settings";

type ToastPayload = {
  id: number;
  source: string;
  title: string;
  body: string;
  tone: string;
};

type AppNotification = {
  id: string;
  title: string;
  body: string;
  source: string;
  sourceAppUserModelId?: string;
  origin: "windows" | "demo";
  createdAt: string;
  tone: string;
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

type AppSettings = {
  runOnStartup: boolean;
  runHighPriority: boolean;
  closeBehavior: CloseBehavior;
  enableTrayIcon: boolean;
  notificationOverlayPlacement: OverlayPlacement;
  capsLockLanguageSwitch: {
    enabled: boolean;
    preserveCapsLockWith: CapsLockFallbackHotkey;
  };
};

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
    return <ToastOverlay />;
  }

  return (
    <Router>
      <MainApp />
    </Router>
  );
}

function MainApp() {
  const location = useLocation();
  const [listenerStatus, setListenerStatus] = createSignal<NotificationListenerStatus>();
  const [settings, setSettings] = createSignal<AppSettings>();
  const [settingsError, setSettingsError] = createSignal<string>();
  const [notifications, setNotifications] = createSignal<AppNotification[]>([]);
  const [captureStatus, setCaptureStatus] = createSignal<NotificationCaptureStatus>();

  onMount(async () => {
    const [listener, appSettings, notificationItems, status] = await Promise.all([
      invoke<NotificationListenerStatus>("notification_listener_status"),
      invoke<AppSettings>("get_app_settings"),
      invoke<AppNotification[]>("get_notifications"),
      invoke<NotificationCaptureStatus>("get_notification_capture_status"),
    ]);
    setListenerStatus(listener);
    setSettings(appSettings);
    setNotifications(notificationItems);
    setCaptureStatus(status);

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

  const active = createMemo(
    () => tools.find((tool) => tool.path === location.pathname) ?? tools[0],
  );

  async function pushToast(tone: string) {
    await invoke("push_demo_toast", { tone });
  }

  async function pushDemoNotification() {
    const notification = await invoke<AppNotification>("push_demo_notification");
    setNotifications((items) => [notification, ...items.filter((item) => item.id !== notification.id)]);
    await refreshCaptureStatus();
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

  return (
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
              <A
                activeClass="selected"
                href={tool.path}
                end
              >
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

        <Route path="/" component={() => <Navigate href="/notifications" />} />
        <Route
          path="/notifications"
          component={() => (
            <PersistentNotificationsPanel
              captureStatus={captureStatus()}
              listenerStatus={listenerStatus()}
              notifications={notifications()}
              pushDemoNotification={pushDemoNotification}
              clearNotifications={clearNotificationHistory}
              dismissNotification={dismissNotification}
              settings={settings()}
              updateSettings={updateSettings}
              settingsError={settingsError()}
            />
          )}
        />
        <Route path="/eye-rest" component={() => <EyeRestPanel pushToast={pushToast} />} />
        <Route
          path="/caps-lock-language-switch"
          component={() => (
            <CapsLockLanguageSwitchPanel
              pushToast={pushToast}
              settings={settings()?.capsLockLanguageSwitch}
              updateSettings={updateCapsLockSettings}
              settingsError={settingsError()}
            />
          )}
        />
        <Route
          path="/settings"
          component={() => (
            <SettingsPanel
              settings={settings()}
              updateSettings={updateSettings}
              settingsError={settingsError()}
            />
          )}
        />
      </section>
    </main>
  );
}

function PersistentNotificationsPanel(props: {
  captureStatus?: NotificationCaptureStatus;
  listenerStatus?: NotificationListenerStatus;
  notifications: AppNotification[];
  pushDemoNotification: () => Promise<void>;
  clearNotifications: () => Promise<void>;
  dismissNotification: (id: string) => Promise<void>;
  settings?: AppSettings;
  updateSettings: (patch: Partial<AppSettings>) => void;
  settingsError?: string;
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
        <Show when={props.settingsError}>
          <p class="error-text">{props.settingsError}</p>
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

  onMount(async () => {
    await getCurrentWindow().setAlwaysOnTop(true);

    const existing = await invoke<AppNotification[]>("get_notifications");
    setToasts(existing.slice(0, 4));

    const unlistenAdded = await listen<AppNotification>("traybits://notification-added", (event) => {
      setToasts((items) => [event.payload, ...items.filter((item) => item.id !== event.payload.id)].slice(0, 4));
    });
    const unlistenDismissed = await listen<string>("traybits://notification-dismissed", (event) => {
      setToasts((items) => items.filter((toast) => toast.id !== event.payload));
      hideWhenEmpty();
    });
    const unlistenCleared = await listen("traybits://notifications-cleared", () => {
      setToasts([]);
      hideWhenEmpty();
    });
    const unlistenLegacy = await listen<ToastPayload>("traybits://toast", (event) => {
      const payload = event.payload;
      const notification: AppNotification = {
        id: `legacy-${payload.id}`,
        title: payload.title,
        body: payload.body,
        source: payload.source,
        origin: "demo",
        createdAt: String(Date.now()),
        tone: payload.tone,
      };
      setToasts((items) => [notification, ...items].slice(0, 4));
    });

    onCleanup(() => {
      unlistenAdded();
      unlistenDismissed();
      unlistenCleared();
      unlistenLegacy();
    });
  });

  function dismiss(id: string) {
    invoke("dismiss_notification", { id }).catch(() => undefined);
    setToasts((items) => items.filter((toast) => toast.id !== id));
    hideWhenEmpty();
  }

  function hideWhenEmpty() {
    window.setTimeout(() => {
      if (toasts().length === 0) {
        invoke("hide_toast_overlay").catch(() => undefined);
      }
    }, 180);
  }

  return (
    <div class="toast-stage">
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
