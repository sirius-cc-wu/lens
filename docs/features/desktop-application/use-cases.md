---
type: "Use Case Model"
title: "FEAT-05: Dioxus Desktop Application"
description: "Defines use cases for the native Dioxus desktop application, multi-tab workspace management, and CLI-to-application handoff."
id: "FEAT-05"
status: "active"
scope: "Lens"
tags: [requirements, use-case, desktop, dioxus, ui]
---

# FEAT-05: Dioxus Desktop Application

Lens currently operates as a CLI tool that delegates browser viewing to external web browser tabs managed across loopback HTTP ports. This model induces browser tab sprawl, lacks OS-level desktop ergonomics, and relies on loopback authentication tokens.

`FEAT-05` introduces a native desktop application built with Dioxus (`dioxus-desktop`), providing an integrated, focused developer workspace for Markdown and diagram inspection, while retaining the frictionless terminal command workflow through single-instance IPC handoff.

---

## System Boundary

Lens is the system under discussion. It includes:
1. The **Lens Desktop Application**: A native OS window powered by Dioxus, `tao`, and `wry`, providing tabbed document viewing, live document refresh, and diagram rendering.
2. The **Lens CLI**: A command-line client that either communicates with a running Lens Desktop instance via a local domain socket or launches the application if none is running.
3. The **Local IPC Bus**: An authenticated per-user local socket (Unix domain socket or Windows named pipe) that coordinates single-instance application control.

External to the system:
* The host operating system's display server (X11 / Wayland on Linux, Win32 on Windows, AppKit on macOS).
* The configured PlantUML server (public or local).
* The user's file system containing Markdown, PlantUML, and Mermaid documents.

---

## Actors

| Actor | Description |
|---|---|
| Developer or Technical Writer | Interacts with Lens via the terminal (`lens [TARGET]`) or directly within the desktop UI to review repository documentation and architecture diagrams. |
| Desktop Window Manager | Renders the native application window, passes user input events, and manages window lifecycle. |
| Operating System Display Server | Provides graphical display context (Wayland, X11, Windows DWM, macOS Quartz). |

---

## Use-Case List

| ID | Use Case | Priority |
|---|---|---|
| `UC-13` | Launch and Interact with Lens Desktop Application | High |
| `UC-14` | Forward Command-Line Target to Running Desktop Application | High |
| `UC-15` | Manage Multi-Tab Document Workspace | Medium |

---

## UC-13: Launch and Interact with Lens Desktop Application

### Primary Actor
Developer or Technical Writer.

### Goal
Open an integrated desktop window displaying a resolved target document or repository documentation catalog, with live refresh and diagram rendering.

### Trigger
The developer runs `lens [TARGET]` when no desktop application instance is currently running, or launches Lens from the desktop application launcher.

### Preconditions
1. The operating system provides an active graphical display environment (active Wayland or X11 session on Linux, native AppKit on macOS, desktop session on Windows).
2. The resolved target is valid and readable within the document root.

### Main Success Scenario

1. The developer invokes `lens [TARGET]` or opens the desktop application directly.
2. Lens validates the target, discovers the document root, and resolves the initial document.
3. Lens verifies that no existing single-instance socket is actively listening.
4. Lens atomically establishes the single-instance IPC listener in a private runtime directory (`0700`) for the invoking operating-system user.
5. Lens initializes the Dioxus desktop runtime (`tao` window and `wry` webview).
6. Lens opens the application window with an initial tab displaying the rendered Markdown document and any associated diagrams (PlantUML via passive `<img>` boundaries, Mermaid via bundled JS).
7. Lens encapsulates the document root, discovered document set, and source-link resolver in an independent `WorkspaceContext` for the tab.
8. Lens establishes file-system watchers for the active document root to support automatic refresh.
9. The developer reviews the document, navigates relative links, and interacts with diagrams directly inside the application window.

### Extensions

* **3a. An existing desktop instance is already active:**
  * Lens branches to `UC-14` (Forward Command-Line Target to Running Desktop Application).
* **3b. Concurrent cold start:**
  * Another instance is concurrently winning the atomic socket election. Lens awaits endpoint readiness with exponential backoff, verifies peer OS credentials, and branches to `UC-14`.
* **5a. No graphical display environment is detected (e.g., Linux SSH session without X11/Wayland):**
  * On Linux, Lens detects that both `DISPLAY` and `WAYLAND_DISPLAY` are unset.
  * If `--server` is specified, Lens falls back to the headless loopback HTTP server (`FEAT-04`); otherwise, it exits cleanly with exit code `1` and actionable diagnostic guidance.
* **6a. Initial target contains PlantUML diagrams:**
  * Lens requests rendered SVGs from the configured PlantUML server and mounts them strictly behind a passive `<img>` element (e.g., `data:image/svg+xml;base64,...`), preventing active script execution.
* **6b. Initial target contains Mermaid diagrams:**
  * The webview executes bundled Mermaid.js in the DOM and replaces diagram blocks with interactive SVGs.

---

## UC-14: Forward Command-Line Target to Running Desktop Application

### Primary Actor
Developer or Technical Writer.

### Goal
Pass a new documentation target or repository path from a terminal into an already-open Lens desktop application without creating duplicate application windows.

### Trigger
The developer runs `lens <TARGET>` in a shell while a Lens desktop application window is already open.

### Preconditions
1. A Lens desktop application instance is running under the current user's session and listening on the local IPC socket.
2. The new target resolves to a valid, readable document or directory.

### Main Success Scenario

1. The developer runs `lens <TARGET>` in a terminal.
2. The CLI client connects to the active per-user IPC socket in the verified private runtime directory.
3. Both client and server verify operating-system peer credentials (`SO_PEERCRED` on Linux, `getpeereid` on macOS, current-user ACL on Windows), rejecting mismatched UIDs.
4. The CLI client transmits a 4-byte length-prefixed `OpenRequest` frame (bounded to 64 KB) containing the lossless native `WirePath`, scope, and invocation directory.
5. The running desktop application receives the request, resolves the target, creates an isolated `WorkspaceContext` for the target's project, and opens the document in a new workspace tab (or focuses the existing tab if already open).
6. The running desktop application requests the OS window manager to bring the Lens window to the foreground and focus the newly opened tab.
7. The desktop application returns a success acknowledgment frame over the IPC socket.
8. The CLI command prints a confirmation to stdout and exits immediately with code `0`, returning control to the invoking terminal.

### Extensions

* **2a. Socket exists but connection is refused (stale socket file from abrupt termination):**
  * The CLI client detects the stale socket, unlinks the stale file, and falls back to `UC-13` (launching a new desktop application instance).
* **3a. Peer credentials mismatch:**
  * The CLI or application terminates the connection immediately without processing payloads.
* **4a. Target path fails validation (missing file or outside allowed root):**
  * The CLI client or application rejects the target.
  * The CLI client prints an actionable validation error to stderr and exits with code `1`. The running application window state is unaffected.

---

## UC-15: Manage Multi-Tab Document Workspace

### Primary Actor
Developer or Technical Writer.

### Goal
Organize multiple open documentation files—even from different repositories—within a single Lens desktop window using tabs, preserving independent project roots and link resolution.

### Trigger
The developer clicks a document link, opens files via the drawer, or closes an active tab.

### Preconditions
The Lens desktop application is open and visible.

### Main Success Scenario

1. The developer clicks a link to another discovered Markdown document inside the active view, or opens a document via CLI handoff (`UC-14`).
2. Lens assigns the tab its corresponding `WorkspaceContext` (preserving independent document roots for cross-repo tabs).
3. The tab bar reflects all active documents with their titles and dirty/modified indicators.
4. The developer switches between tabs; Lens preserves scroll position and switches the active workspace scope and file catalog.
5. The developer closes a tab; Lens releases file watchers associated solely with that document, focusing the adjacent active tab.

### Extensions

* **5a. The developer closes the last remaining tab:**
  * Lens displays an empty workspace landing page with a target finder and recent repository roots, remaining ready for subsequent `lens <target>` CLI handoffs.
