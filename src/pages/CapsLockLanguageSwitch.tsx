import { Show } from "solid-js";
import type { AppSettings, CapsLockFallbackHotkey } from "../types";

export function CapsLockLanguageSwitchPanel(props: {
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
