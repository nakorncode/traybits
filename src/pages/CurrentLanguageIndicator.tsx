import { For, Show, createSignal, onCleanup, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, CurrentLanguageIndicatorStatus } from "../types";

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
          Shows a compact language marker such as TH, EN, or JA when Windows reports that the
          foreground input language changed. The marker appears only when Windows exposes a real
          caret rectangle through UI Automation or Win32 caret APIs.
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
