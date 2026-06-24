import { Show, createSignal, onCleanup, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { LanguageIndicatorPayload } from "../types";

export function LanguageIndicatorOverlay() {
  const [payload, setPayload] = createSignal<LanguageIndicatorPayload>();
  let hideTimer: number | undefined;

  onMount(() => {
    let unlisten: (() => void) | undefined;
    void listen<LanguageIndicatorPayload>("traybits://language-indicator", (event) => {
      setPayload(event.payload);
      if (hideTimer) {
        window.clearTimeout(hideTimer);
      }
      hideTimer = window.setTimeout(() => {
        setPayload(undefined);
        invoke("hide_language_indicator_overlay").catch(() => undefined);
      }, 1_100);
    }).then((cleanup) => {
      unlisten = cleanup;
    });

    onCleanup(() => {
      unlisten?.();
      if (hideTimer) {
        window.clearTimeout(hideTimer);
      }
    });
  });

  return (
    <main
      class="language-indicator-stage"
      classList={{
        "bounds-overlay": payload()?.boundsVisible ?? false,
      }}
    >
      <Show when={payload()}>
        {(item) => (
          <section
            classList={{
              "language-indicator-card": true,
              [`size-${item().size}`]: true,
              fallback: !item().caretAvailable,
            }}
          >
            <strong>{item().code}</strong>
          </section>
        )}
      </Show>
    </main>
  );
}
