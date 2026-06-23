import { Show, createEffect, createSignal, onCleanup, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Toaster, toast } from "solid-sonner";
import type { AppNotification, AppSettings } from "../types";

export function ToastOverlay() {
  const [toasts, setToasts] = createSignal<AppNotification[]>([]);
  const [settings, setSettings] = createSignal<AppSettings>();
  const [lastPollAt, setLastPollAt] = createSignal<string>();
  const [lastPollError, setLastPollError] = createSignal<string>();
  const announcedIds = new Set<string>();
  const activeSonnerIds = new Set<string>();
  let stageRef: HTMLDivElement | undefined;
  let lastMeasuredOverlayHeight = 0;

  onMount(() => {
    void syncOverlayState();
    const poll = window.setInterval(syncOverlayState, 750);
    let unlistenAdded: (() => void) | undefined;
    const resizeObserver = new ResizeObserver(() => requestOverlayResize());
    const mutationObserver = new MutationObserver(() => requestOverlayResize());

    void listen<AppNotification>("traybits://notification-added", (event) => {
      if (event.payload.silent) return;
      setToasts((items) => [event.payload, ...items.filter((item) => item.id !== event.payload.id)].slice(0, 4));
      showOverlaySonner(event.payload);
    }).then((unlisten) => {
      unlistenAdded = unlisten;
    });

    if (stageRef) {
      resizeObserver.observe(stageRef);
      mutationObserver.observe(stageRef, { childList: true, subtree: true, attributes: true });
    }

    onCleanup(() => {
      window.clearInterval(poll);
      unlistenAdded?.();
      resizeObserver.disconnect();
      mutationObserver.disconnect();
    });
  });

  createEffect(() => {
    toasts();
    settings();
    requestOverlayResize();
  });

  async function syncOverlayState() {
    try {
      const [latest, appSettings] = await Promise.all([
        invoke<AppNotification[]>("get_notifications"),
        invoke<AppSettings>("get_app_settings"),
      ]);
      const visible = latest.filter((notification) => !notification.silent).slice(0, 4);
      setSettings(appSettings);
      setLastPollAt(new Date().toLocaleTimeString());
      setLastPollError(undefined);
      const visibleIds = new Set(visible.map((notification) => notification.id));

      for (const notification of visible) {
        showOverlaySonner(notification);
        markOverlaySonnerInteractivity(notification);
      }

      for (const id of Array.from(activeSonnerIds)) {
        if (!visibleIds.has(id)) {
          activeSonnerIds.delete(id);
          toast.dismiss(id);
        }
      }
      setToasts(visible);
      if (activeSonnerIds.size === 0) {
        scheduleOverlayHide();
      }
    } catch (error) {
      setLastPollError(error instanceof Error ? error.message : String(error));
    }
  }

  function showOverlaySonner(notification: AppNotification) {
    if (announcedIds.has(notification.id)) return;
    announcedIds.add(notification.id);
    activeSonnerIds.add(notification.id);
    toast.info(notification.title, {
      id: notification.id,
      toasterId: "overlay",
      description: `${notification.source}: ${notification.body}`,
      duration: Number.POSITIVE_INFINITY,
      closeButton: true,
      onDismiss: () => {
        activeSonnerIds.delete(notification.id);
        void invoke("dismiss_notification", { id: notification.id });
        requestOverlayResize();
        scheduleOverlayHide();
      },
    });
    markOverlaySonnerInteractivity(notification);
    requestOverlayResize();
  }

  function markOverlaySonnerInteractivity(notification: AppNotification) {
    window.requestAnimationFrame(() => {
      const element = document.querySelector<HTMLElement>(
        `[data-sonner-toast][data-id="${CSS.escape(notification.id)}"]`,
      );
      if (!element) return;
      element.dataset.traybitsClickable = notification.sourceAppUserModelId ? "true" : "false";
      if (!notification.sourceAppUserModelId || element.dataset.traybitsOpenBound === "true") {
        return;
      }
      element.dataset.traybitsOpenBound = "true";
      element.addEventListener("click", (event) => {
        const target = event.target as HTMLElement | null;
        if (target?.closest("button")) return;
        void openNotificationSource(notification.id);
      });
    });
  }

  async function openNotificationSource(id: string) {
    await invoke("open_notification_source", { id }).catch(() => undefined);
    activeSonnerIds.delete(id);
    toast.dismiss(id);
    await invoke("dismiss_notification", { id }).catch(() => undefined);
    requestOverlayResize();
    scheduleOverlayHide();
  }

  function requestOverlayResize() {
    window.requestAnimationFrame(() => {
      measureAndResizeOverlay();
      window.setTimeout(measureAndResizeOverlay, 120);
      window.setTimeout(measureAndResizeOverlay, 320);
    });
  }

  function measureAndResizeOverlay() {
    if (!stageRef) return;
    const measuredElements = [
      ...Array.from(stageRef.querySelectorAll<HTMLElement>("[data-sonner-toast]")),
      ...Array.from(stageRef.querySelectorAll<HTMLElement>(".overlay-debug-card")),
    ];
    if (measuredElements.length === 0) return;

    const stageTop = stageRef.getBoundingClientRect().top;
    const bounds = measuredElements.map((element) => {
      const rect = element.getBoundingClientRect();
      return {
        bottom: rect.bottom - stageTop,
        top: rect.top - stageTop,
      };
    });
    const contentTop = Math.min(...bounds.map((rect) => rect.top));
    const contentBottom = Math.max(...bounds.map((rect) => rect.bottom));
    const contentHeight = Math.ceil(contentBottom - contentTop + 36);
    if (!Number.isFinite(contentHeight) || contentHeight <= 0) return;
    if (Math.abs(contentHeight - lastMeasuredOverlayHeight) < 8) return;
    lastMeasuredOverlayHeight = contentHeight;
    invoke("resize_toast_overlay_for_content", { contentHeight }).catch(() => undefined);
  }

  function scheduleOverlayHide() {
    window.setTimeout(() => {
      if (activeSonnerIds.size === 0 && !settings()?.notificationOverlayDebugVisible) {
        invoke("hide_toast_overlay").catch(() => undefined);
      }
    }, 250);
  }

  return (
    <div
      ref={(element) => {
        stageRef = element;
      }}
      class="toast-stage"
      onMouseEnter={requestOverlayResize}
      onMouseLeave={requestOverlayResize}
      classList={{
        "debug-overlay": settings()?.notificationOverlayDebugVisible ?? true,
        "bounds-overlay": settings()?.notificationOverlayBoundsVisible ?? false,
      }}
    >
      <Show when={settings()?.notificationOverlayDebugVisible ?? true}>
        <div class="overlay-debug-card">
          <strong>TrayBits overlay debug</strong>
          <span>Transparent overlay window is visible.</span>
          <span>Store polling is active{lastPollAt() ? `, last checked ${lastPollAt()}` : ""}.</span>
          <span>Visible notifications in overlay store: {toasts().length}</span>
          <Show when={toasts()[0]}>
            {(notification) => <span>Newest: {notification().title}</span>}
          </Show>
          <Show when={lastPollError()}>
            {(error) => <span class="overlay-debug-error">Poll error: {error()}</span>}
          </Show>
        </div>
      </Show>
      <Toaster
        id="overlay"
        position="top-right"
        richColors
        closeButton
        expand={false}
        visibleToasts={4}
        duration={Number.POSITIVE_INFINITY}
        pauseWhenPageIsHidden={false}
        toastOptions={{
          closeButtonAriaLabel: "Close notification",
          classNames: {
            toast: "overlay-sonner-toast",
            description: "overlay-sonner-description",
          },
        }}
      />
    </div>
  );
}
