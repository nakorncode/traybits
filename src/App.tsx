import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";

type ToolId = "toastdesk" | "eye-rest" | "capslang" | "settings";

type ToastPayload = {
  id: number;
  source: string;
  title: string;
  body: string;
  tone: string;
};

type NotificationListenerStatus = {
  feasible: boolean;
  api: string;
  permissionRequired: boolean;
  packagingRisk: string;
  prototypeStep: string;
};

const tools: Array<{
  id: ToolId;
  name: string;
  description: string;
  glyph: string;
  status: string;
}> = [
  {
    id: "toastdesk",
    name: "ToastDesk",
    description: "Persistent desktop toast cards rendered by a Tauri webview.",
    glyph: "T",
    status: "Prototype",
  },
  {
    id: "eye-rest",
    name: "20-20-20",
    description: "Eye-rest reminders for long desktop sessions.",
    glyph: "20",
    status: "Planned",
  },
  {
    id: "capslang",
    name: "CapsLang",
    description: "Use CapsLock as a safer input-language switch key.",
    glyph: "C",
    status: "Spike",
  },
  {
    id: "settings",
    name: "Settings",
    description: "Suite-wide startup, privacy, and release controls.",
    glyph: "S",
    status: "Draft",
  },
];

function App() {
  const params = new URLSearchParams(window.location.search);
  const view = params.get("view");

  if (view === "toast") {
    return <ToastOverlay />;
  }

  return <MainApp />;
}

function MainApp() {
  const [activeTool, setActiveTool] = createSignal<ToolId>("toastdesk");
  const [listenerStatus, setListenerStatus] = createSignal<NotificationListenerStatus>();

  onMount(async () => {
    setListenerStatus(await invoke<NotificationListenerStatus>("notification_listener_status"));
  });

  const active = createMemo(() => tools.find((tool) => tool.id === activeTool()) ?? tools[0]);

  async function pushToast(tone: string) {
    await invoke("push_demo_toast", { tone });
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
              <button
                classList={{ selected: activeTool() === tool.id }}
                onClick={() => setActiveTool(tool.id)}
                type="button"
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
          <button class="quiet-button" type="button" onClick={() => pushToast("windows")}>
            Show desktop toast
          </button>
        </header>

        <Show when={activeTool() === "toastdesk"}>
          <ToastDeskPanel pushToast={pushToast} listenerStatus={listenerStatus()} />
        </Show>
        <Show when={activeTool() === "eye-rest"}>
          <EyeRestPanel pushToast={pushToast} />
        </Show>
        <Show when={activeTool() === "capslang"}>
          <CapsLangPanel pushToast={pushToast} />
        </Show>
        <Show when={activeTool() === "settings"}>
          <SettingsPanel />
        </Show>
      </section>
    </main>
  );
}

function ToastDeskPanel(props: {
  pushToast: (tone: string) => Promise<void>;
  listenerStatus?: NotificationListenerStatus;
}) {
  return (
    <div class="content-grid">
      <section class="panel primary-panel">
        <div class="section-title">
          <span>Toast renderer</span>
          <strong>Solid overlay window</strong>
        </div>
        <p>
          The button below sends an event from Rust to a separate transparent Tauri window. That
          window is frameless, skipped from the taskbar, and always on top.
        </p>
        <div class="button-row">
          <button type="button" onClick={() => props.pushToast("windows")}>
            Windows notification
          </button>
          <button type="button" onClick={() => props.pushToast("rest")}>
            20-20-20 reminder
          </button>
          <button type="button" onClick={() => props.pushToast("capslang")}>
            CapsLang event
          </button>
        </div>
      </section>

      <section class="panel">
        <div class="section-title">
          <span>Windows notification listener</span>
          <strong>{props.listenerStatus?.feasible ? "Feasible with caveats" : "Checking"}</strong>
        </div>
        <dl class="fact-list">
          <div>
            <dt>API</dt>
            <dd>{props.listenerStatus?.api ?? "Loading..."}</dd>
          </div>
          <div>
            <dt>Permission</dt>
            <dd>{props.listenerStatus?.permissionRequired ? "User permission required" : "Unknown"}</dd>
          </div>
          <div>
            <dt>Risk</dt>
            <dd>{props.listenerStatus?.packagingRisk ?? "Waiting for Rust status..."}</dd>
          </div>
        </dl>
      </section>

      <section class="panel wide-panel">
        <div class="section-title">
          <span>Native spike plan</span>
          <strong>How Rust should capture real-time notifications</strong>
        </div>
        <ol class="steps">
          <li>Add the Windows notification listener capability to the packaged app identity.</li>
          <li>Call UserNotificationListener.RequestAccessAsync from a UI-owned path.</li>
          <li>Subscribe to NotificationChanged for foreground changes.</li>
          <li>Sync current toast notifications with GetNotificationsAsync(NotificationKinds.Toast).</li>
          <li>Emit normalized notification payloads to the Solid overlay window.</li>
        </ol>
      </section>
    </div>
  );
}

function EyeRestPanel(props: { pushToast: (tone: string) => Promise<void> }) {
  return (
    <section class="panel primary-panel">
      <div class="section-title">
        <span>Reminder preview</span>
        <strong>20-20-20 notification flow</strong>
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

function CapsLangPanel(props: { pushToast: (tone: string) => Promise<void> }) {
  return (
    <section class="panel primary-panel">
      <div class="section-title">
        <span>Keyboard spike</span>
        <strong>CapsLock to input-language switch</strong>
      </div>
      <p>
        Rust should own the low-level keyboard hook and input-language API calls. The UI only shows
        status, settings, and event feedback.
      </p>
      <button type="button" onClick={() => props.pushToast("capslang")}>
        Preview CapsLang toast
      </button>
    </section>
  );
}

function SettingsPanel() {
  return (
    <section class="panel primary-panel">
      <div class="section-title">
        <span>Suite settings</span>
        <strong>Shared behavior later</strong>
      </div>
      <div class="settings-list">
        <label>
          <input type="checkbox" checked readOnly />
          Start TrayBits in the system tray
        </label>
        <label>
          <input type="checkbox" checked readOnly />
          Use always-on-top toast overlay
        </label>
        <label>
          <input type="checkbox" readOnly />
          Capture real Windows notifications
        </label>
      </div>
    </section>
  );
}

function ToastOverlay() {
  const [toasts, setToasts] = createSignal<ToastPayload[]>([]);

  onMount(async () => {
    await getCurrentWindow().setAlwaysOnTop(true);
    const unlisten = await listen<ToastPayload>("traybits://toast", (event) => {
      setToasts((items) => [event.payload, ...items].slice(0, 4));
      window.setTimeout(() => dismiss(event.payload.id), 9000);
    });

    onCleanup(() => {
      unlisten();
    });
  });

  function dismiss(id: number) {
    setToasts((items) => items.filter((toast) => toast.id !== id));
    if (toasts().length <= 1) {
      window.setTimeout(() => {
        if (toasts().length === 0) {
          invoke("hide_toast_overlay").catch(() => undefined);
        }
      }, 180);
    }
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
