# Persistent Notifications Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Persistent Notifications slice with routed pages, Windows notification capture, persistent overlay cards, 9-position placement, demo preview, and a bundled short notification sound.

**Architecture:** Solid renders the shell, routed module pages, and the transparent overlay UI. Rust owns app settings, notification capture, notification storage, overlay placement, sound playback, and Tauri commands/events. Windows notification capture follows ToastDesk's proven baseline: poll `UserNotificationListener.GetNotificationsAsync(NotificationKinds.Toast)` and treat change events as optional acceleration.

**Tech Stack:** Tauri 2, Rust, SolidJS, `@solidjs/router`, Windows `UserNotificationListener`, bundled WAV sound asset.

## Global Constraints

- Use direct behavior names for TrayBits modules.
- Keep the transparent Tauri `toast` window and Rust-to-Solid event bridge.
- Overlay placement supports exactly 9 positions: top-left, top-center, top-right, middle-left, center, middle-right, bottom-left, bottom-center, bottom-right.
- Windows notification capture must not block app startup when permission is denied or unsupported.
- Prefer polling `UserNotificationListener.GetNotificationsAsync(NotificationKinds.Toast)` as the reliable baseline.
- Do not simulate Windows shortcuts for notification capture or display.
- Bundle only production-safe audio assets with source/license notes.
- Verification commands: `pnpm build`, `cargo check --manifest-path .\src-tauri\Cargo.toml`, `pnpm tauri build --no-bundle`.

---

## File Structure

- `package.json`, `pnpm-lock.yaml`: add `@solidjs/router`.
- `src\App.tsx`: install router, replace local active-tool state with route-aware navigation, add routed module pages, subscribe to notification events.
- `src\App.css`: styles for routed shell, notification history, placement controls, and Sonner-like toast cards.
- `src-tauri\Cargo.toml`, `src-tauri\Cargo.lock`: add Windows notification and audio support dependencies.
- `src-tauri\src\lib.rs`: add notification records, capture status, commands/events, overlay placement, Windows capture worker, and sound playback.
- `assets\sounds\traybits-notification.wav`: short free notification sound.
- `assets\sounds\README.md`: sound source and license.
- `README.md`: update status and manual smoke instructions.
- `AGENTS.md`: durable guidance for notification capture polling.

---

### Task 1: Router and Page Shell

**Files:**
- Modify: `package.json`
- Modify: `pnpm-lock.yaml`
- Modify: `src\App.tsx`
- Modify: `src\App.css`

**Interfaces:**
- Produces route paths: `/notifications`, `/eye-rest`, `/caps-lock-language-switch`, `/settings`.
- Keeps overlay query mode: `index.html?view=toast`.

- [ ] **Step 1: Add router dependency**

Run:

```powershell
pnpm add @solidjs/router
```

Expected: `package.json` includes `@solidjs/router`, and `pnpm-lock.yaml` updates.

- [ ] **Step 2: Wrap the main shell in a router**

In `src\App.tsx`, import:

```ts
import { A, Navigate, Route, Router } from "@solidjs/router";
```

Keep `view === "toast"` returning `<ToastOverlay />`. For the normal app, render:

```tsx
return (
  <Router>
    <MainApp />
  </Router>
);
```

- [ ] **Step 3: Replace local active-tool state with route navigation**

Use tool metadata with `path`:

```ts
type Tool = {
  id: ToolId;
  path: string;
  name: string;
  description: string;
  glyph: string;
  status: string;
};
```

Use `<A href={tool.path}>` for sidebar buttons and derive the active tool by matching `location.pathname`.

- [ ] **Step 4: Add routes**

Inside the workspace body, define:

```tsx
<Route path="/" component={() => <Navigate href="/notifications" />} />
<Route path="/notifications" component={() => <PersistentNotificationsPanel ... />} />
<Route path="/eye-rest" component={() => <EyeRestPanel pushToast={pushToast} />} />
<Route path="/caps-lock-language-switch" component={() => <CapsLockLanguageSwitchPanel ... />} />
<Route path="/settings" component={() => <SettingsPanel ... />} />
```

- [ ] **Step 5: Verify**

Run:

```powershell
pnpm build
```

Expected: Vite build succeeds.

---

### Task 2: Rust Notification Store and Capture

**Files:**
- Modify: `src-tauri\Cargo.toml`
- Modify: `src-tauri\Cargo.lock`
- Modify: `src-tauri\src\lib.rs`

**Interfaces:**
- Produces commands:
  - `get_notifications() -> Vec<AppNotification>`
  - `push_demo_notification(app: AppHandle) -> Result<AppNotification, String>`
  - `dismiss_notification(app: AppHandle, state: State<AppState>, id: String) -> Result<(), String>`
  - `clear_notifications(app: AppHandle, state: State<AppState>) -> Result<(), String>`
  - `get_notification_capture_status(state: State<AppState>) -> Result<NotificationCaptureStatus, String>`
- Produces events:
  - `traybits://notification-added`
  - `traybits://notification-dismissed`
  - `traybits://notifications-cleared`

- [ ] **Step 1: Add Rust types**

In `src-tauri\src\lib.rs`, add:

```rust
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppNotification {
    id: String,
    title: String,
    body: String,
    source: String,
    source_app_user_model_id: Option<String>,
    origin: NotificationOrigin,
    created_at: String,
    tone: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
enum NotificationOrigin {
    Windows,
    Demo,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NotificationCaptureStatus {
    enabled: bool,
    access: String,
    message: String,
    mode: String,
}
```

- [ ] **Step 2: Extend `AppState`**

Add:

```rust
notifications: Mutex<Vec<AppNotification>>,
notification_capture_status: Mutex<NotificationCaptureStatus>,
captured_windows_notification_ids: Mutex<std::collections::HashSet<u32>>,
```

- [ ] **Step 3: Add store helpers**

Add helpers:

```rust
fn add_notification(app: &AppHandle, state: &AppState, notification: AppNotification) -> Result<(), String>
fn dismiss_notification_by_id(app: &AppHandle, state: &AppState, id: &str) -> Result<(), String>
fn emit_notification_added(app: &AppHandle, notification: &AppNotification)
```

`add_notification` inserts at the front, caps history to 100, emits `traybits://notification-added`, calls `show_toast_window(app)`, and plays sound.

- [ ] **Step 4: Add commands**

Register these commands in `tauri::generate_handler!`:

```rust
get_notifications,
push_demo_notification,
dismiss_notification,
clear_notifications,
get_notification_capture_status
```

- [ ] **Step 5: Add Windows capture module**

Under `#[cfg(target_os = "windows")]`, use `windows` crate APIs:

```rust
Windows::UI::Notifications::Management::{
    UserNotificationListener,
    UserNotificationListenerAccessStatus,
}
Windows::UI::Notifications::NotificationKinds
Windows::UI::Notifications::KnownNotificationBindings
```

Start a background polling thread during setup. On success, request access when status is `Unspecified`, poll current toasts, skip already captured IDs, extract text elements, and add notifications.

- [ ] **Step 6: Add non-Windows stub**

Under `#[cfg(not(target_os = "windows"))]`, set status to disabled with message `Windows notification capture is only available on Windows.`

- [ ] **Step 7: Verify**

Run:

```powershell
cargo check --manifest-path .\src-tauri\Cargo.toml
```

Expected: Rust check succeeds without warnings.

---

### Task 3: Overlay Placement and Solid Notification UI

**Files:**
- Modify: `src-tauri\src\lib.rs`
- Modify: `src\App.tsx`
- Modify: `src\App.css`

**Interfaces:**
- Extends `AppSettings` with `notification_overlay_placement: OverlayPlacement`.
- Frontend setting type uses `notificationOverlayPlacement`.
- Overlay window receives `AppNotification` through `traybits://notification-added`.

- [ ] **Step 1: Add placement enum to Rust settings**

Add:

```rust
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum OverlayPlacement {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    Center,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}
```

Default: `TopRight`.

- [ ] **Step 2: Reposition overlay from Rust**

Change `show_toast_window` to read current settings and compute x/y from placement. Use monitor size, scale, overlay width `440`, overlay height `520`, and margin `24`.

- [ ] **Step 3: Add frontend placement controls**

In Persistent Notifications page, render a 3x3 segmented grid using values:

```ts
const overlayPlacements = [
  "topLeft", "topCenter", "topRight",
  "middleLeft", "center", "middleRight",
  "bottomLeft", "bottomCenter", "bottomRight",
] as const;
```

Saving updates `notificationOverlayPlacement`.

- [ ] **Step 4: Replace overlay demo-only toasts with notification cards**

In `ToastOverlay`, store `AppNotification[]`, listen to:

```ts
listen<AppNotification>("traybits://notification-added", ...)
listen<string>("traybits://notification-dismissed", ...)
listen("traybits://notifications-cleared", ...)
```

Do not auto-dismiss cards. Dismiss calls `dismiss_notification`.

- [ ] **Step 5: Add notification page history**

Persistent Notifications page loads `get_notifications`, displays status from `get_notification_capture_status`, shows notification history, and has buttons for `push_demo_notification` and `clear_notifications`.

- [ ] **Step 6: Verify**

Run:

```powershell
pnpm build
cargo check --manifest-path .\src-tauri\Cargo.toml
```

Expected: both succeed.

---

### Task 4: Sound Asset, Preview, Docs, Final Build

**Files:**
- Create: `assets\sounds\traybits-notification.wav`
- Create: `assets\sounds\README.md`
- Modify: `src-tauri\src\lib.rs`
- Modify: `README.md`
- Modify: `AGENTS.md`

**Interfaces:**
- Sound plays from Rust on notification add.
- `push_demo_notification` triggers history card, overlay card, and sound.

- [ ] **Step 1: Add a free sound asset**

Use a short notification WAV from a source that permits app bundling. Save:

```text
assets/sounds/traybits-notification.wav
```

Record source URL and license in:

```text
assets/sounds/README.md
```

- [ ] **Step 2: Add sound playback**

Use a small Rust audio dependency if needed. Play the bundled WAV when `add_notification` succeeds. If playback fails, ignore the error and keep the notification visible.

- [ ] **Step 3: Update docs**

Update README to mention:

- routed Persistent Notifications page
- Windows capture status/history
- 9-position overlay placement
- demo preview notification
- bundled notification sound

Update AGENTS with:

- prefer polling `GetNotificationsAsync`
- keep sound assets license-documented

- [ ] **Step 4: Final verification**

Run:

```powershell
pnpm build
cargo check --manifest-path .\src-tauri\Cargo.toml
pnpm tauri build --no-bundle
```

Expected: all pass, and executable is written to `src-tauri\target\release\traybits.exe`.

- [ ] **Step 5: Commit**

Run:

```powershell
git add package.json pnpm-lock.yaml src src-tauri assets README.md AGENTS.md
git commit -m "Implement persistent notifications"
```

Do not push from `main`.
