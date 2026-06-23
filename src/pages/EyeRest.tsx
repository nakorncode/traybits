import { Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { formatDuration } from "../utils";
import type { AppSettings, EyeRestStatus } from "../types";

export function EyeRestPanel(props: {
  settings?: AppSettings["eyeRestReminder"];
  updateSettings: (patch: Partial<AppSettings["eyeRestReminder"]>) => void;
  settingsError?: string;
}) {
  const [status, setStatus] = createSignal<EyeRestStatus>();

  onMount(() => {
    void refresh();
    const poll = window.setInterval(refresh, 1_000);
    onCleanup(() => window.clearInterval(poll));
  });

  async function refresh() {
    const latest = await invoke<EyeRestStatus>("get_eye_rest_status").catch(() => undefined);
    if (latest) setStatus(latest);
  }

  async function runEyeRestAction(command: "start_eye_rest_timer" | "stop_eye_rest_timer" | "skip_eye_rest_timer") {
    const latest = await invoke<EyeRestStatus>(command).catch(() => undefined);
    if (latest) setStatus(latest);
  }

  const nextDueText = createMemo(() => {
    const current = status();
    if (!current?.enabled) return "Disabled";
    if (current.phase === "prompt") return "Ready to rest";
    if (current.phase === "resting") return "Resting now";
    if (current.phase === "done") return "Rest complete";
    if (!current.nextDueAt) return "Timer is preparing";
    const remaining = Math.max(0, current.nextDueAt - current.now);
    return `${formatDuration(remaining)} remaining`;
  });

  const elapsedText = createMemo(() => {
    const current = status();
    if (!current?.enabled || !current.nextDueAt) return "Not running";
    if (current.phase === "prompt" || current.phase === "resting" || current.phase === "done") {
      return "Interval complete";
    }
    const intervalMillis = current.intervalMinutes * 60_000;
    const remaining = Math.max(0, current.nextDueAt - current.now);
    return formatDuration(Math.max(0, intervalMillis - remaining));
  });

  const remainingText = createMemo(() => {
    const current = status();
    if (!current?.enabled || !current.nextDueAt) return "Not running";
    if (current.phase === "prompt") return "Waiting to start rest";
    if (current.phase === "resting") {
      return `${formatDuration(Math.max(0, (current.restReadyAt ?? current.now) - current.now))} rest left`;
    }
    if (current.phase === "done") return "Ready to continue";
    return formatDuration(Math.max(0, current.nextDueAt - current.now));
  });

  return (
    <div class="content-grid">
      <section class="panel primary-panel">
        <div class="section-title">
          <span>Timer</span>
          <strong>Eye rest reminder</strong>
        </div>
        <p>
          TrayBits opens a dedicated bottom-right overlay when the interval ends, then waits for a
          button click before starting the 20-second rest.
        </p>
        <div class="settings-list">
          <div class="button-row">
            <button
              type="button"
              disabled={status()?.enabled}
              onClick={() => runEyeRestAction("start_eye_rest_timer")}
            >
              Start
            </button>
            <button
              type="button"
              disabled={!status()?.enabled}
              onClick={() => runEyeRestAction("stop_eye_rest_timer")}
            >
              Stop
            </button>
            <button type="button" onClick={() => runEyeRestAction("skip_eye_rest_timer")}>
              Skip to rest now
            </button>
          </div>
          <label class="field-row">
            <span>Reminder interval</span>
            <div class="inline-input-row">
              <input
                type="number"
                min="1"
                max="240"
                step="1"
                value={props.settings?.intervalMinutes ?? 20}
                disabled={!props.settings}
                onChange={(event) =>
                  props.updateSettings({
                    enabled: status()?.enabled ?? props.settings?.enabled ?? false,
                    intervalMinutes: Math.max(
                      1,
                      Math.min(240, Number(event.currentTarget.value) || 20),
                    ),
                  })
                }
              />
              <span>minutes</span>
            </div>
          </label>
        </div>
        <Show when={props.settingsError}>
          <p class="error-text">{props.settingsError}</p>
        </Show>
      </section>

      <section class="panel">
        <div class="section-title">
          <span>Status</span>
          <strong>{nextDueText()}</strong>
        </div>
        <dl class="fact-list">
          <div>
            <dt>Elapsed</dt>
            <dd>{elapsedText()}</dd>
          </div>
          <div>
            <dt>Remaining</dt>
            <dd>{remainingText()}</dd>
          </div>
          <div>
            <dt>Rest duration</dt>
            <dd>20 seconds</dd>
          </div>
          <div>
            <dt>Sound</dt>
            <dd>Uses the selected notification sound setting.</dd>
          </div>
          <div>
            <dt>Overlay</dt>
            <dd>Dedicated bottom-right window until resumed.</dd>
          </div>
        </dl>
      </section>
    </div>
  );
}
