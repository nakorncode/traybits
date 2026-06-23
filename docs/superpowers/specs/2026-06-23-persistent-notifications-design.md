# Persistent Notifications Design

Date: 2026-06-23

## Goal

Prepare the TrayBits Persistent Notifications module so it can prove real
Windows notification capture and display persistent desktop cards from the
same local store used by demo previews.

## Approved Approach

Use a pragmatic port from ToastDesk:

- Add Solid routing so each module can have its own page route.
- Keep the existing transparent Tauri `toast` window and make its placement
  configurable across the 9 screen positions.
- Add a Rust-owned notification store exposed to Solid through Tauri commands
  and events.
- Start Windows notification capture when the app starts.
- Use `UserNotificationListener.GetNotificationsAsync(NotificationKinds.Toast)`
  polling as the reliable baseline. Subscribe to `NotificationChanged` only if
  the Windows API allows it, matching the ToastDesk lesson.
- Show captured Windows notifications on the Persistent Notifications page as
  proof that capture works.
- Add demo preview notifications that flow through the same store and overlay
  path.
- Add one short, production-safe notification sound under `assets/sounds/`,
  with source/license notes.

## Routes and Pages

Add `@solidjs/router` and define routes for:

- `/notifications`
- `/eye-rest`
- `/caps-lock-language-switch`
- `/settings`

The old single-page sidebar remains the main shell. Navigation changes the
route instead of switching local component state. The transparent overlay window
continues to use `index.html?view=toast` so it does not need to render the app
router.

## Persistent Notification Data

Rust owns the canonical notification list.

Each notification record should include:

- `id`
- `title`
- `body`
- `source`
- `source_app_user_model_id`
- `origin`: `windows` or `demo`
- `created_at`
- `tone`

Commands:

- `get_notifications`
- `push_demo_notification`
- `dismiss_notification`
- `clear_notifications`
- `get_notification_capture_status`

Events:

- `traybits://notification-added`
- `traybits://notification-dismissed`
- `traybits://notifications-cleared`

## Windows Capture

On app startup, Rust starts a Windows-only capture service.

Behavior:

- Request `UserNotificationListener` access when needed.
- Poll current notifications and append newly seen Windows notifications.
- Prefer polling because ToastDesk found event subscription unreliable in
  unpackaged desktop processes.
- If event subscription works, use it only as a faster trigger with polling as
  backup.
- If access is denied or unsupported, expose the status on the notifications
  page instead of failing app startup.

## Overlay Placement

Add an overlay placement setting with 9 positions:

- top-left
- top-center
- top-right
- middle-left
- center
- middle-right
- bottom-left
- bottom-center
- bottom-right

Rust should position the transparent `toast` window based on the current
monitor work area. Solid still handles card stacking inside the overlay
window. The first implementation can keep one overlay window sized for a
stack of cards.

## Solid Toast UI

Use a Solid implementation shaped like `solid-sonner`:

- persistent cards
- dismiss button
- source/title/body
- stacked display
- preview demo action

If `solid-sonner` fits cleanly, use it. If it assumes a normal page DOM and
does not fit the transparent overlay window, implement a small local Sonner-like
component and keep the dependency out.

## Sound

Bundle one short free notification sound in `assets/sounds/`.

Requirements:

- short alert tone, not long music
- free for app use
- source URL and license recorded in `assets/sounds/README.md`
- played for both captured Windows notifications and demo preview unless later
  settings disable it

## Error Handling

- Notification access denied: show status and retry action.
- Windows metadata missing: show best-effort title/body/source.
- Sound missing or playback failure: do not block notification display.
- Overlay positioning failure: fall back to top-right.

## Verification

Focused checks:

- `pnpm build`
- `cargo check --manifest-path .\src-tauri\Cargo.toml`
- `pnpm tauri build --no-bundle`

Manual smoke after build:

- Open TrayBits and confirm `/notifications` loads.
- Run demo preview and confirm page list, overlay card, and sound.
- Trigger a real Windows notification and confirm it appears in the page list.
- Change overlay placement and confirm card position changes.
