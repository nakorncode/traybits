# TrayBits

Small Windows utilities for daily desktop flow.

TrayBits is a home for focused Windows tools that live quietly in the tray or
near the desktop and fix small workflow problems without becoming a heavy
control center.

The project is inspired by the practical shape of Microsoft PowerToys: a suite
of independent utilities, each useful on its own, with shared release,
documentation, and maintenance conventions over time.

## Planned Utilities

TrayBits starts as an umbrella for these existing or planned tools:

| Utility | Purpose | Current source |
| --- | --- | --- |
| ToastDesk | Persistent Windows notification cards that stay visible until acted on. | `G:\NakornCode\git\toastdeck` |
| 20-20-20 reminder | Periodic eye-rest notifications for the 20-20-20 rule. | Planned / to be located |
| CapsLang | Turn `CapsLock` into a safer input-language switch key. | `G:\Freespace\capslang-windows` |

The first goal is not to merge code blindly. TrayBits should make the utility
family understandable, then decide case by case whether each tool should stay
separate, become a package in this repository, or share common libraries and
release automation.

## Design Principles

- Small tools first.
- Native-feeling Windows behavior.
- Tray/background behavior should be predictable.
- No unnecessary cloud dependency.
- No broad system tweaks unless the user explicitly enables them.
- Prefer durable fixes over one-off scripts.
- Keep each utility understandable and independently testable.

## Repository Status

This repository is newly initialized. It currently contains project direction
and local development instructions only.

Expected next steps:

1. Inventory the existing ToastDesk and CapsLang codebases.
2. Find or define the current 20-20-20 reminder implementation.
3. Choose the first repository shape:
   - umbrella docs only,
   - monorepo containing all utilities,
   - or shared core plus separate app repos.
4. Decide the release strategy for Windows installers and portable packages.

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

