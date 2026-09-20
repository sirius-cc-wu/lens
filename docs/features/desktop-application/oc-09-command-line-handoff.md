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

- A Lens desktop application instance is running and listening on the designated local IPC socket in the user's private runtime directory (`0700`).
- The target is a readable path within an authorized repository or directory root.

## Postconditions on Success

- The client and server verified matching operating-system peer credentials (`SO_PEERCRED` on Linux, `getpeereid` on macOS, Windows named pipe current-user ACL).
- A 4-byte length-prefixed `OpenRequest` frame (bounded to 64 KB) was sent across the local IPC socket, containing lossless native `WirePath` representations of the target and invocation directory.
- The desktop application resolved the target relative to `invocation_directory`.
- If the target document was already open in a tab, the desktop application selected that tab.
- If the target document was not open, the desktop application appended a new tab, initialized its independent `WorkspaceContext` for its repository root, rendered its Markdown and passive diagrams, and made it the active tab.
- The desktop application requested the operating system window manager to bring the Lens window to the foreground and un-minimize it.
- The desktop application returned an `OpenResponse` with `status: ok` over the socket.
- The CLI command printed a success confirmation and terminated with exit code `0`.

## Error Conditions

- **Peer Credential Mismatch:** If peer UID does not match the invoking user's UID, the connection is closed immediately without reading or sending payloads.
- **Oversized or Malformed Frame:** If the frame length exceeds 64 KB or fails framing validation, the connection is terminated immediately.
- **Socket Unreachable (Connection Refused):** If the socket file exists but the connection is refused, the CLI removes the stale socket file and falls back to launching a new desktop application instance (`OC-08`).
- **Target Validation Failure:** If the target path cannot be resolved or is unreadable, the CLI outputs an error message to stderr, does not forward the request, and exits with code `1`.
