---
type: "Operation Contract"
title: "OC-09: Command-Line Target Handoff"
description: "Specifies IPC message dispatch, desktop tab management, window focus, and non-blocking CLI completion for target handoff."
id: "OC-09"
operation: "open_target(target, invocation_directory)"
traces: [UC-14, SSD-09, ADR-026]
status: "active"
tags: [analysis, operation-contract, cli, ipc, handoff]
---

# OC-09: Command-Line Target Handoff

Operation:
`open_target(target, invocation_directory)`

Cross References: `UC-14`, [`SSD-09`](ssd-09-command-line-handoff.md), and [`ADR-026`](../../decisions/adr-026-dioxus-desktop-application.md)

Scope: Lens

## Effect in Plain Language

A CLI command connects to the active desktop application's local IPC socket, transmits an `OpenTarget` message with the target path and invocation directory, and exits immediately. The running desktop application opens the document in a new tab (or focuses it if already open) and raises the window.

## Preconditions

- A Lens desktop application instance is running and listening on the designated local IPC socket for the current user.
- The target is a readable path within an authorized repository or directory root.

## Postconditions on Success

- An `OpenTarget` JSON message was sent across the local IPC socket.
- The desktop application validated the target relative to `invocation_directory`.
- If the target document was already open in a tab, the desktop application selected that tab.
- If the target document was not open, the desktop application appended a new tab, rendered its Markdown and diagrams, and made it the active tab.
- The desktop application requested the operating system window manager to bring the Lens window to the foreground and un-minimize it.
- The desktop application returned an `OpenTargetResponse` with `status: ok` over the socket.
- The CLI command printed a success confirmation and terminated with exit code `0`.

## Error Conditions

- **Socket Unreachable (Connection Refused):** If the socket file exists but the connection is refused, the CLI removes the stale socket file and falls back to launching a new desktop application instance (`OC-08`).
- **Target Validation Failure:** If the target path cannot be resolved or is unreadable, the CLI outputs an error message to stderr, does not forward the request, and exits with code `1`.
