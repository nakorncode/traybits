# TrayBits Agent Notes

These instructions apply to work in `G:\NakornCode\git\traybits`.

## Project Intent

TrayBits is planned as a public-facing Windows utility suite, similar in spirit
to PowerToys but smaller and focused on the user's daily desktop tools.

Initial scope:

- `ToastDesk`: persistent Windows notification cards.
- `20-20-20 reminder`: eye-rest notifications.
- `CapsLang`: CapsLock-based input-language switching.

Do not assume these tools should be merged into one executable. First inspect
the current code and release flow, then recommend the smallest durable repo
shape that preserves working behavior.

## Local Machine Context

Known related paths on this PC:

- `G:\NakornCode\git\toastdeck`
- `G:\Freespace\capslang-windows`
- `G:\NakornCode\git\traybits`

`toastdeck` and `capslang-windows` are existing .NET Windows app projects with
their own `AGENTS.md`, release scripts, assets, and build outputs. Read their
local instructions before copying or modifying code.

## Working Style

- Keep changes small and reviewable.
- Prefer PowerShell commands on Windows.
- Prefer `pnpm` only for JavaScript projects; current related tools are .NET.
- Do not introduce Electron, Tauri, or another app framework without comparing
  it against the current .NET/native Windows approach.
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

For existing .NET utility projects, prefer their local scripts first, such as:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\run-build.ps1
```

If a utility lacks scripts, use targeted .NET commands such as:

```powershell
dotnet build
```

Do not run installer builds, release packaging, or broad UI/browser automation
unless the task requires it or the user explicitly asks.

