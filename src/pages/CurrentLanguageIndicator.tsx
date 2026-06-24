import { For, Show, createSignal, onCleanup, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { overlayPlacements } from "../constants";
import type {
  AppSettings,
  CurrentLanguageIndicatorMode,
  CurrentLanguageIndicatorSize,
  CurrentLanguageIndicatorStatus,
} from "../types";

const indicatorSizes: Array<{ id: CurrentLanguageIndicatorSize; label: string }> = [
  { id: "small", label: "Small" },
  { id: "medium", label: "Medium" },
  { id: "large", label: "Large" },
];

export function CurrentLanguageIndicatorPanel(props: {
  settings?: AppSettings["currentLanguageIndicator"];
  updateSettings: (patch: Partial<AppSettings["currentLanguageIndicator"]>) => void;
  settingsError?: string;
}) {
  const [status, setStatus] = createSignal<CurrentLanguageIndicatorStatus>();
  const [previewError, setPreviewError] = createSignal<string>();

  onMount(() => {
    void refreshStatus();
    const timer = window.setInterval(refreshStatus, 1_000);
    onCleanup(() => window.clearInterval(timer));
  });

  async function refreshStatus() {
    setStatus(await invoke<CurrentLanguageIndicatorStatus>("current_language_indicator_status"));
  }

  async function previewIndicator() {
    setPreviewError(undefined);
    try {
      await invoke("preview_current_language_indicator");
      await refreshStatus();
    } catch (error) {
      setPreviewError(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <div class="content-grid">
      <section class="panel primary-panel">
        <div class="section-title">
          <span>Typing marker</span>
          <strong>Current input language near the caret</strong>
        </div>
        <p>
          Shows a compact language marker such as THA, ENG, or JPN when Windows reports that the
          foreground input language changed. Caret overlay is experimental; corner modes avoid
          caret-specific focus and provider issues.
        </p>
        <div class="settings-list">
          <label class="toggle-row">
            <input
              type="checkbox"
              checked={props.settings?.enabled ?? false}
              disabled={!props.settings}
              onChange={(event) => props.updateSettings({ enabled: event.currentTarget.checked })}
            />
            Enable Current Language Indicator
          </label>
          <label class="field-row">
            <span>Display mode</span>
            <select
              value={props.settings?.mode ?? "screenCorner"}
              disabled={!props.settings}
              onChange={(event) =>
                props.updateSettings({
                  mode: event.currentTarget.value as CurrentLanguageIndicatorMode,
                })
              }
            >
              <option value="screenCorner">Screen corner</option>
              <option value="focusedWindowCorner">Focused window corner</option>
              <option value="caretOverlay">Caret overlay (experimental)</option>
            </select>
          </label>
          <div>
            <span class="field-label">Corner placement</span>
            <div class="placement-grid" role="group" aria-label="Language indicator placement">
              <For each={overlayPlacements}>
                {(placement) => (
                  <button
                    type="button"
                    disabled={!props.settings || props.settings.mode === "caretOverlay"}
                    classList={{ selected: (props.settings?.placement ?? "topRight") === placement.id }}
                    title={placement.label}
                    onClick={() => props.updateSettings({ placement: placement.id })}
                  >
                    {placement.shortLabel}
                  </button>
                )}
              </For>
            </div>
          </div>
          <div>
            <span class="field-label">Indicator size</span>
            <div class="segmented-control" role="group" aria-label="Language indicator size">
              <For each={indicatorSizes}>
                {(size) => (
                  <button
                    type="button"
                    disabled={!props.settings}
                    classList={{ selected: (props.settings?.size ?? "medium") === size.id }}
                    onClick={() => props.updateSettings({ size: size.id })}
                  >
                    {size.label}
                  </button>
                )}
              </For>
            </div>
          </div>
          <label class="toggle-row">
            <input
              type="checkbox"
              checked={props.settings?.boundsVisible ?? false}
              disabled={!props.settings}
              onChange={(event) => props.updateSettings({ boundsVisible: event.currentTarget.checked })}
            />
            Show indicator overlay bounds
          </label>
          <button class="quiet-button" type="button" onClick={previewIndicator}>
            Preview indicator
          </button>
        </div>
        <Show when={props.settingsError}>
          <p class="error-text">{props.settingsError}</p>
        </Show>
        <Show when={previewError()}>
          <p class="error-text">{previewError()}</p>
        </Show>
      </section>

      <section class="panel">
        <div class="section-title">
          <span>Runtime status</span>
          <strong>{status()?.enabled ? "Enabled" : "Disabled"}</strong>
        </div>
        <dl class="fact-list compact-facts">
          <div>
            <dt>Display mode</dt>
            <dd>{status()?.mode ?? "Loading"}</dd>
          </div>
          <div>
            <dt>Current language</dt>
            <dd>
              {status()?.current
                ? `${status()?.current?.displayCode} - ${status()?.current?.label}`
                : "Not detected"}
            </dd>
          </div>
          <div>
            <dt>Caret position</dt>
            <dd>{status()?.caretAvailable ? "Available" : "Not available in the focused app"}</dd>
          </div>
          <div>
            <dt>Source</dt>
            <dd>{status()?.source ?? "Loading"}</dd>
          </div>
        </dl>
        <p class="hint-text">{status()?.message ?? "Loading indicator status..."}</p>
      </section>

      <section class="panel">
        <div class="section-title">
          <span>Debug</span>
          <strong>Caret provider probes</strong>
        </div>
        <dl class="fact-list compact-facts debug-facts">
          <div>
            <dt>Foreground window</dt>
            <dd>{status()?.debug.foregroundWindow ?? "Loading"}</dd>
          </div>
          <div>
            <dt>Foreground thread</dt>
            <dd>{status()?.debug.foregroundThreadId ?? "Not available"}</dd>
          </div>
          <div>
            <dt>Keyboard layout</dt>
            <dd>{status()?.debug.keyboardLayout ?? "Not available"}</dd>
          </div>
          <div>
            <dt>UI Automation</dt>
            <dd>{status()?.debug.uiAutomation ?? "Loading"}</dd>
          </div>
          <div>
            <dt>Win32 caret</dt>
            <dd>{status()?.debug.win32Caret ?? "Loading"}</dd>
          </div>
        </dl>
      </section>

      <section class="panel wide-panel">
        <div class="section-title">
          <span>Installed input languages</span>
          <strong>Windows layouts on this machine</strong>
        </div>
        <div class="language-list">
          <For each={status()?.installed ?? []} fallback={<p class="empty-text">No layouts detected yet.</p>}>
            {(language) => (
              <div class="language-row">
                <span>{language.displayCode}</span>
                <div>
                  <strong>{language.label}</strong>
                  <small>
                    {language.localeName} / ISO {language.languageCode}
                  </small>
                </div>
              </div>
            )}
          </For>
        </div>
      </section>
    </div>
  );
}
