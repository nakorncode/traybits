import { Show } from "solid-js";
import type { AppSettings, CloseBehavior, SettingsStorageStatus } from "../types";

export function SettingsPanel(props: {
  settings?: AppSettings;
  updateSettings: (patch: Partial<AppSettings>) => void;
  storageStatus?: SettingsStorageStatus;
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
      <Show when={props.storageStatus}>
        {(status) => (
          <dl class="fact-list compact-facts debug-facts">
            <div>
              <dt>Settings file</dt>
              <dd>{status().path}</dd>
            </div>
            <div>
              <dt>Storage status</dt>
              <dd>{status().message}</dd>
            </div>
            <div>
              <dt>File size</dt>
              <dd>{status().exists ? `${status().bytes ?? 0} bytes` : "Not created yet"}</dd>
            </div>
            <div>
              <dt>Modified</dt>
              <dd>{status().modifiedAt ?? "Not available"}</dd>
            </div>
          </dl>
        )}
      </Show>
      <Show when={props.settingsError}>
        <p class="error-text">{props.settingsError}</p>
      </Show>
    </section>
  );
}
