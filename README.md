# TrayBits

Small Windows utilities for daily desktop flow.

TrayBits is a home for focused Windows tools that live quietly in the tray or
near the desktop and fix small workflow problems without becoming a heavy
control center.

The project is inspired by the practical shape of Microsoft PowerToys: a suite
of independent utilities, each useful on its own, with shared release,
documentation, and maintenance conventions over time.

## Current Prototype

TrayBits now has an initial Tauri 2 + Rust + SolidJS prototype.

It includes:

- A PowerToys-style main window with a left sidebar for utilities.
- A transparent, frameless, always-on-top toast overlay window.
- Rust commands that emit demo toast events into the SolidJS overlay.
- A tray icon menu for opening the app and triggering a demo toast.
- A Windows notification listener status panel that documents the native spike
  needed for real notification capture.

The toast overlay is intentionally separate from the main app window. This keeps
desktop toast rendering close to Sonner-style web UI while letting Rust own the
native Windows integration.

## Planned Utilities

TrayBits starts as an umbrella for these existing or planned modules:

| Module | Purpose | Reference source |
| --- | --- | --- |
| Persistent Notifications | Keep Windows notifications visible as desktop cards until dismissed or handled. | `G:\NakornCode\git\toastdeck` |
| Eye Rest Reminder | Show periodic 20-20-20 reminders for long desktop sessions. | Planned / to be located |
| Caps Lock Language Switch | Use Caps Lock as a quick input-language switch while preserving clear lock behavior. | `G:\Freespace\capslang-windows` |

The first goal is not to merge code blindly. TrayBits should prove the Tauri
shape first, then decide case by case whether each tool should be ported,
wrapped, or kept separate.

Names inside TrayBits should describe the behavior directly. Branded names from
the standalone tools are reference names only, not product-facing module names.

## Architecture Direction

```text
SolidJS UI
  - PowerToys-style settings shell
  - Sonner-style toast overlay

Tauri bridge
  - commands from UI to Rust
  - events from Rust to overlay window
  - tray lifecycle

Rust Windows core
  - future notification listener
  - future Caps Lock hook and input-language switching
  - future 20-20-20 scheduler
```

Real Windows notification capture should use
`Windows.UI.Notifications.Management.UserNotificationListener` through Rust's
Windows bindings. It requires a packaged app identity, a notification listener
capability, and explicit user permission.

## Design Principles

- Small tools first.
- Native-feeling Windows behavior.
- Tray/background behavior should be predictable.
- No unnecessary cloud dependency.
- No broad system tweaks unless the user explicitly enables them.
- Prefer durable fixes over one-off scripts.
- Keep each utility understandable and independently testable.

## Repository Status

This repository is an early prototype. The UI shell and demo overlay compile,
but real Windows notification capture and Caps Lock switching are not
implemented yet.

Expected next steps:

1. Prove real `UserNotificationListener` access from a packaged Tauri app.
2. Prototype Caps Lock interception and language switching in Rust.
3. Port the 20-20-20 reminder timer into the Rust core.
4. Decide whether the standalone notification and Caps Lock tools are ported
   into this repo or kept as reference implementations.

## Development

Install dependencies:

```powershell
pnpm install
```

Run frontend build:

```powershell
pnpm build
```

Run Rust check:

```powershell
cargo check --manifest-path .\src-tauri\Cargo.toml
```

Build the Tauri desktop app without bundling installers:

```powershell
pnpm tauri build --no-bundle
```

The no-bundle executable is written to:

```text
src-tauri\target\release\traybits.exe
```

## Local Paths

Primary workspace on this PC:

```text
G:\NakornCode\git\traybits
```

Related local projects:

```text
G:\NakornCode\git\toastdeck
G:\Freespace\capslang-windows
```

## Contributing Locally

Use PowerShell on Windows by default.

Before moving code into this repository, inspect the existing tool's README,
AGENTS.md, scripts, installer files, and release workflow. Preserve working
behavior first, then simplify once the boundary is clear.
