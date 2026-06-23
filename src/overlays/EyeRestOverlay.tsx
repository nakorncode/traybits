import { Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { formatDuration } from "../utils";
import type { EyeRestStatus } from "../types";

export function EyeRestOverlay() {
  const [status, setStatus] = createSignal<EyeRestStatus>();
  const [error, setError] = createSignal<string>();

  onMount(() => {
    void refresh();
    const poll = window.setInterval(refresh, 500);
    let unlistenStarted: (() => void) | undefined;
    void listen<EyeRestStatus>("traybits://eye-rest-updated", (event) => {
      setStatus(event.payload);
    }).then((unlisten) => {
      unlistenStarted = unlisten;
    });

    onCleanup(() => {
      window.clearInterval(poll);
      unlistenStarted?.();
    });
  });

  async function refresh() {
    const latest = await invoke<EyeRestStatus>("get_eye_rest_status").catch((reason) => {
      setError(String(reason));
      return undefined;
    });
    if (latest) {
      setStatus(latest);
      setError(undefined);
    }
  }

  const remainingSeconds = createMemo(() => {
    const current = status();
    if (current?.phase !== "resting" || !current.restReadyAt) return 20;
    return Math.max(0, Math.ceil((current.restReadyAt - current.now) / 1_000));
  });

  const titleText = createMemo(() => {
    const phase = status()?.phase;
    if (phase === "resting") return "Rest your eyes";
    if (phase === "done") return "Rest complete";
    return "Ready for an eye break";
  });

  const bodyText = createMemo(() => {
    const phase = status()?.phase;
    if (phase === "resting") return "Look away, blink slowly, and keep your focus off the screen.";
    if (phase === "done") return "Good. Continue when you are ready to start the next interval.";
    return "Press start when you are ready, then rest for a full 20 seconds.";
  });

  const canStartRest = createMemo(() => status()?.phase === "prompt");
  const canResume = createMemo(() => status()?.phase === "done");

  async function startRest() {
    const latest = await invoke<EyeRestStatus>("start_eye_rest_break").catch((reason) => {
      setError(String(reason));
      return undefined;
    });
    if (latest) setStatus(latest);
  }

  async function resume() {
    const latest = await invoke<EyeRestStatus>("complete_eye_rest").catch((reason) => {
      setError(String(reason));
      return undefined;
    });
    if (latest) setStatus(latest);
  }

  return (
    <main class="eye-rest-stage">
      <section class="eye-rest-card">
        <div>
          <span>Eye Rest Reminder</span>
          <strong>{titleText()}</strong>
        </div>
        <p>{bodyText()}</p>
        <Show when={status()?.phase === "resting"}>
          <div class="eye-rest-countdown">
            <strong>{remainingSeconds()}</strong>
            <span>seconds</span>
          </div>
        </Show>
        <div class="eye-rest-actions">
          <button type="button" disabled={!canStartRest()} onClick={startRest}>
            Start 20-second rest
          </button>
          <button type="button" disabled={!canResume()} onClick={resume}>
            Continue timer
          </button>
          <button type="button" class="secondary" onClick={resume}>
            Skip rest
          </button>
        </div>
        <Show when={error()}>
          {(message) => <small class="eye-rest-error">{message()}</small>}
        </Show>
      </section>
    </main>
  );
}
