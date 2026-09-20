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

- The host environment provides a graphical display server (active Wayland or X11 session on Linux, native AppKit on macOS, desktop session on Windows).
- No active process is listening on the current user's designated IPC endpoint.

## Postconditions on Success

- The optional target was resolved relative to the canonical form of `invocation_directory`; an omitted target resolved to `invocation_directory`.
- Target resolution produced a canonical document root, a discovered document set, and an initial selected document.
- The IPC listener was bound in a private user-owned runtime directory (`0700`) and verified to enforce peer UID authentication (`SO_PEERCRED` / `getpeereid`).
- A native desktop window was created via `tao` and surfaced to the display server.
- The webview was initialized via `wry` with the bundled Mermaid.js script injected.
- PlantUML diagrams were mounted strictly behind passive `<img>` elements (e.g. data URIs), neutralizing active SVG scripts and event handlers.
- The initial document was parsed by `lens-core` into sanitized HTML, attached to an independent `WorkspaceContext`, and rendered into Tab 1 of the Dioxus application shell.
- File-system watchers were registered for the active document root.

## Error Conditions

- **No Display Server (Linux):** If `DISPLAY` and `WAYLAND_DISPLAY` are unset on Linux, Lens does not panic; it outputs an actionable diagnostic message directing the user to start a headless server session (`--server`).
- **Concurrent Cold Start:** If another instance wins the atomic socket election, this instance waits for endpoint readiness, converts to a client invocation, and executes `OC-09`.
- **Stale Socket:** If an existing socket file cannot be reached (connection refused), the stale file is automatically removed and a new listener established.
- **Target Unreadable:** If the target does not exist or is unreadable, an error toast is surfaced in the application shell and an empty workspace catalog is displayed.
