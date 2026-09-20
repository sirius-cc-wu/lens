---
type: "Operation Contract"
title: "OC-08: Launch Desktop Application"
description: "Specifies window creation, IPC listener binding, webview initialization, and document rendering for launching the Dioxus desktop application."
id: "OC-08"
operation: "launch_desktop_application(target?, invocation_directory, scope)"
traces: [UC-13, SSD-08, ADR-026]
status: "active"
tags: [analysis, operation-contract, desktop, dioxus]
---

# OC-08: Launch Desktop Application

Operation:
`launch_desktop_application(target?, invocation_directory, scope)`

Cross References: `UC-13`, [`SSD-08`](ssd-08-open-target-in-application.md), and [`ADR-026`](../../decisions/adr-026-dioxus-desktop-application.md)

Scope: Lens

## Effect in Plain Language

When no Lens desktop instance is active, launching the desktop application creates the single-instance IPC listener, constructs a native desktop window via Dioxus, mounts the in-process webview, and displays the initial resolved document in Tab 1.

## Preconditions

- The host environment provides a graphical display server (`DISPLAY`, `WAYLAND_DISPLAY`, or native desktop environment).
- No active process is listening on the current user's designated IPC endpoint.

## Postconditions on Success

- The optional target was resolved relative to the canonical form of `invocation_directory`; an omitted target resolved to `invocation_directory`.
- Target resolution produced a canonical document root, a discovered document set, and an initial selected document.
- Exactly one local IPC listener (Unix domain socket or Windows named pipe) was bound under user-exclusive permissions (`0600`).
- A native desktop window was created via `tao` and surfaced to the display server.
- The webview was initialized via `wry` with the bundled Mermaid.js script injected.
- The initial document was parsed by `lens-core` into sanitized HTML and rendered into Tab 1 of the Dioxus application shell.
- File-system watchers were registered for the active document root.

## Error Conditions

- **No Display Server:** If no display environment is detected, Lens does not panic; it outputs an actionable diagnostic message directing the user to start a headless server session or run in a graphical environment.
- **Stale Socket:** If an existing socket file cannot be reached (connection refused), the stale file is automatically removed and a new listener established.
- **Target Unreadable:** If the target does not exist or is unreadable, an error toast is surfaced in the application shell and an empty workspace catalog is displayed.
