import { For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { overlayPlacements } from "../constants";
import { formatTimestamp } from "../utils";
import type { AppNotification, AppSettings, NotificationCaptureStatus, NotificationListenerStatus, NotificationSoundPreset, OverlayMonitorOption } from "../types";

export function PersistentNotificationsPanel(props: {
  captureStatus?: NotificationCaptureStatus;
  listenerStatus?: NotificationListenerStatus;
  notifications: AppNotification[];
  notificationSoundPresets: NotificationSoundPreset[];
  overlayMonitors: OverlayMonitorOption[];
  pushDemoNotification: () => Promise<void>;
  clearNotifications: () => Promise<void>;
  dismissNotification: (id: string) => Promise<void>;
  settings?: AppSettings;
  updateSettings: (patch: Partial<AppSettings>) => void;
  settingsError?: string;
  notificationError?: string;
  notificationStatus?: string;
}) {
  const persistentNotificationsEnabled = props.settings?.persistentNotificationsEnabled ?? true;

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
        <label class="toggle-row">
          <input
            type="checkbox"
            checked={persistentNotificationsEnabled}
            disabled={!props.settings}
            onChange={(event) =>
              props.updateSettings({ persistentNotificationsEnabled: event.currentTarget.checked })
            }
          />
          Enable Persistent Notifications
        </label>
        <div class="button-row">
          <button
            type="button"
            disabled={!props.settings || !persistentNotificationsEnabled}
            onClick={props.pushDemoNotification}
          >
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
                disabled={!props.settings || !persistentNotificationsEnabled}
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
            disabled={!props.settings || !persistentNotificationsEnabled || props.overlayMonitors.length === 0}
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
            checked={props.settings?.nativeNotificationEnabled ?? true}
            disabled={!props.settings || !persistentNotificationsEnabled}
            onChange={(event) =>
              props.updateSettings({ nativeNotificationEnabled: event.currentTarget.checked })
            }
          />
          Show TrayBits native Windows notifications
        </label>
        <label class="toggle-row">
          <input
            type="checkbox"
            checked={props.settings?.dismissMirroredWindowsNotifications ?? false}
            disabled={!props.settings || !persistentNotificationsEnabled}
            onChange={(event) =>
              props.updateSettings({
                dismissMirroredWindowsNotifications: event.currentTarget.checked,
              })
            }
          />
          Dismiss captured Windows notifications after mirroring
        </label>
        <label class="toggle-row">
          <input
            type="checkbox"
            checked={props.settings?.notificationSoundEnabled ?? true}
            disabled={!props.settings || !persistentNotificationsEnabled}
            onChange={(event) =>
              props.updateSettings({ notificationSoundEnabled: event.currentTarget.checked })
            }
          />
          Enable notification sound
        </label>
        <label class="field-row overlay-monitor-field">
          <span>Notification sound</span>
          <select
            value={props.settings?.notificationSoundPreset ?? "aosp-argon"}
            disabled={!props.settings || !persistentNotificationsEnabled || props.notificationSoundPresets.length === 0}
            onChange={(event) => {
              const presetId = event.currentTarget.value;
              props.updateSettings({ notificationSoundPreset: presetId });
              void invoke("preview_notification_sound", { presetId }).catch(() => undefined);
            }}
          >
            <For each={props.notificationSoundPresets}>
              {(preset) => <option value={preset.id}>{preset.label}</option>}
            </For>
          </select>
        </label>
        <label class="toggle-row">
          <input
            type="checkbox"
            checked={props.settings?.notificationOverlayDebugVisible ?? true}
            disabled={!props.settings || !persistentNotificationsEnabled}
            onChange={(event) =>
              props.updateSettings({ notificationOverlayDebugVisible: event.currentTarget.checked })
            }
          />
          Show overlay debug background
        </label>
        <label class="toggle-row">
          <input
            type="checkbox"
            checked={props.settings?.notificationOverlayBoundsVisible ?? false}
            disabled={!props.settings || !persistentNotificationsEnabled}
            onChange={(event) =>
              props.updateSettings({ notificationOverlayBoundsVisible: event.currentTarget.checked })
            }
          />
          Show toast overlay bounds
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
