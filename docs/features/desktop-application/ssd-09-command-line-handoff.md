---
type: "System Sequence Diagram"
title: "SSD-09: Command-Line Target Handoff"
description: "Shows a CLI command forwarding a target to a running Lens desktop application via local IPC and exiting immediately."
id: "SSD-09"
use_case: "UC-14"
scenario: "Forward a target path from a terminal to an existing desktop application window."
status: "active"
tags: [analysis, ssd, cli, ipc, handoff]
---

# SSD-09: Command-Line Target Handoff

## Scenario Context

When a Lens desktop window is already running, a subsequent CLI command forwards the target to the running instance over the local IPC socket, requests window focus, and returns control to the shell immediately.

## Actors

- Developer or Technical Writer
- Desktop Window Manager

```plantuml
@startuml
actor Developer
participant ":LensCLI" as CLI
participant ":LensApp" as App
actor "Window Manager" as WM

Developer -> CLI: open_target(target, invocation_directory)
activate CLI
CLI -> App: send_ipc(OpenTarget)
activate App
App -> App: resolve_target(target, invocation_directory)
App -> App: append_or_focus_tab(document_id)
App -> WM: request_window_focus()
App --> CLI: OpenTargetResponse(ok)
deactivate App
CLI --> Developer: target_opened
deactivate CLI
@enduml
```

## System Events

1. Developer -> CLI: `open_target(target, invocation_directory)`
2. CLI -> App: `send_ipc(OpenTarget)`
3. App -> App: `resolve_target(target, invocation_directory)`
4. App -> App: `append_or_focus_tab(document_id)`
5. App -> Window Manager: `request_window_focus()`
6. App -> CLI: `OpenTargetResponse(ok)`
7. CLI -> Developer: `target_opened`
