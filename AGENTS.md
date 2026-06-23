# TrayBits Agent Notes

These instructions apply to work in `G:\NakornCode\git\traybits`.

## Project Intent

TrayBits is planned as a public-facing Windows utility suite, similar in spirit
to PowerToys but smaller and focused on the user's daily desktop tools.

Initial scope:

- `Persistent Notifications`: Windows notification cards that stay visible
  until dismissed or handled.
- `Eye Rest Reminder`: 20-20-20 eye-rest notifications.
- `Caps Lock Language Switch`: Caps Lock based input-language switching.

The current prototype uses Tauri 2, Rust, and SolidJS. Do not assume the
existing .NET tools should be merged directly. First inspect the current code
and release flow, then recommend the smallest durable porting path that
preserves working behavior.

## Local Machine Context

Known related paths on this PC:

- `G:\NakornCode\git\toastdeck`
- `G:\Freespace\capslang-windows`
- `G:\NakornCode\git\traybits`

`toastdeck` and `capslang-windows` are existing .NET Windows app projects with
their own `AGENTS.md`, release scripts, assets, and build outputs. Read their
local instructions before porting behavior.

Use direct behavior names for TrayBits modules. Keep standalone project names
such as ToastDesk and CapsLang only as reference-source or migration-context
labels.

## Current Stack

- Tauri 2 desktop app.
- Rust native backend under `src-tauri`.
- SolidJS frontend under `src`.
- pnpm package management.

The app currently has two windows:

- `main`: PowerToys-style utility shell.
- `toast`: transparent frameless always-on-top overlay for desktop toasts.

Rust emits toast payloads to the `toast` window with Tauri events. Keep this
bridge shape unless a better native constraint appears.

## Working Style

- Keep changes small and reviewable.
- Prefer PowerShell commands on Windows.
- Use `pnpm` for this repository's frontend commands.
- Do not introduce Electron or another app framework without comparing it
  against the current Tauri/Rust approach.
- Treat tray behavior, startup registration, notification permissions, keyboard
  hooks, and installer identity as user-impacting surfaces. Inspect existing
  behavior before changing them.
- Avoid creating duplicate app identities, startup entries, notification app
  IDs, or installer product IDs.

## Documentation Expectations

- Keep `README.md` understandable for a newcomer.
- Use this `AGENTS.md` for machine-specific workflow notes and technical
  guardrails.
- If future work reveals durable project guidance, update this file in the same
  change.

## Verification

Use focused checks for the touched surface.

For this Tauri prototype, use:

```powershell
pnpm build
cargo check --manifest-path .\src-tauri\Cargo.toml
pnpm tauri build --no-bundle
```

For related .NET utility projects, prefer their local scripts first, such as:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\run-build.ps1
```

Do not run installer builds, release packaging, or broad UI/browser automation
unless the task requires it or the user explicitly asks.

## Windows Native Spikes

Real notification capture should be proven before porting the standalone
notification-card behavior:

- Use `Windows.UI.Notifications.Management.UserNotificationListener`.
- Request access from a UI-owned path.
- Subscribe to `NotificationChanged`.
- Sync current notifications with `GetNotificationsAsync(NotificationKinds.Toast)`.
- Expect app identity, manifest capability, and user permission issues.

Caps Lock language switching should stay Rust-owned:

- Low-level keyboard hook or a safer Windows-native alternative.
- Explicit handling for real Caps Lock on/off behavior.
- No broad keystroke logging.
- UI only controls settings and displays state.
