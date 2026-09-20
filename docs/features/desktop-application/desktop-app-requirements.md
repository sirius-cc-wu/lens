---
type: "Feature Specification"
title: "FEAT-05: Dioxus Desktop Application Requirements"
description: "Specifies functional requirements, user interactions, business rules, edge cases, and acceptance criteria for the Dioxus desktop application."
id: "FEAT-05-REQ"
status: "active"
scope: "Lens"
tags: [requirements, specification, desktop, dioxus, ui]
---

# FEAT-05: Dioxus Desktop Application Requirements

## 1. Problem Statement & User Value

Lens currently serves documents through loopback HTTP ports and opens browser tabs in the system web browser. While this provides zero-dependency rendering, it incurs notable user frictions:
1. **Browser Tab Sprawl:** Every command invocation (`lens docs/spec.md`) opens an additional tab in the developer's default browser, cluttering unrelated browsing contexts and leaving zombie background sessions.
2. **Missing Desktop Ergonomics:** Developers lack a unified workspace window with native tab management, a collapsible file drawer, diagram pan/zoom controls, and window state persistence.
3. **Loopback Security Overhead:** Communicating between browser tabs and the background daemon requires loopback TCP port allocation, query string auth tokens, and CORS/Private Network workarounds.

### Product Goal

Transform Lens into a high-performance, single-instance **Desktop Application** powered by **Dioxus (`dioxus-desktop`)**:
- Provide an integrated, native window for Markdown reading, PlantUML rendering, and Mermaid diagram exploration.
- Preserve terminal agility via a fast, non-blocking CLI command that forwards new targets to the active desktop window via local IPC.
- Maintain a 100% pure Rust codebase without introducing Node.js, TypeScript, or npm packaging requirements.

---

## 2. User Stories

### Story 1: Desktop Reading Experience
**As a** developer or architect reading technical documentation,  
**I want** a dedicated desktop application window with tabs, file drawer, and high-fidelity diagram rendering,  
**So that** I can explore architecture specs, sequence diagrams, and source links without cluttering my web browser.

### Story 2: Frictionless CLI Handoff
**As a** terminal-focused developer,  
**I want to** run `lens docs/architecture.md` from my shell,  
**So that** the target immediately opens in a new tab inside my existing Lens desktop window and brings the window to focus, without waiting or spawning redundant processes.

---

## 3. Example Mapping (Three-Amigos Discovery)

```
                       ┌────────────────────────────────────────────────────────┐
                       │                       User Story                       │
                       │             Lens Dioxus Desktop Application            │
                       └──────────────────────────┬─────────────────────────────┘
                                                  │
         ┌───────────────────┬────────────────────┼───────────────────┬───────────────────┐
         ▼                   ▼                    ▼                   ▼                   ▼
    ┌──────────┐       ┌───────────┐        ┌───────────┐       ┌───────────┐       ┌───────────┐
    │  Rule 1  │       │  Rule 2   │        │  Rule 3   │       │  Rule 4   │       │  Rule 5   │
    │ Single   │       │ CLI Target│        │ In-Process│       │ Live File │       │ Headless  │
    │ Instance │       │ Handoff   │        │ Webview   │       │ Watch &   │       │ Fallback  │
    │ Window   │       │ IPC       │        │ Rendering │       │ Refresh   │       │ Handling  │
    └────┬─────┘       └─────┬─────┘        └─────┬─────┘       └─────┬─────┘       └─────┬─────┘
         │                   │                    │                   │                   │
         ├─────────┐         ├─────────┐          ├─────────┐         │                   │
         ▼         ▼         ▼         ▼          ▼         ▼         ▼                   ▼
     [Ex 1.1]  [Ex 1.2]  [Ex 2.1]  [Ex 2.2]   [Ex 3.1]  [Ex 3.2]  [Ex 4.1]            [Ex 5.1]
     Cold      Second    Target    Invalid    Markdown  Mermaid   File save           SSH / no
     start     invocation handed   path       rendered  JS runs   updates             display
     opens     reuses    over      reports    in DOM    in DOM    tab live            warns &
     window    window    socket    error      without   without   without             falls back
                         & focuses            HTTP      reload    reload
```

---

## 4. Detailed Business Rules

### Rule 1: Single-Instance Window & Endpoint Security
* **Rule 1.1:** Exactly one desktop application window executes per operating-system user.
* **Rule 1.2:** The local IPC endpoint MUST reside within a verified user-owned private runtime directory (`0700` permissions) at `$XDG_RUNTIME_DIR/lens/lens.sock` (or `/tmp/lens-$UID/lens.sock` on POSIX, `\\.\pipe\lens-$USERNAME` on Windows with current-user ACL).
* **Rule 1.3:** Both client and server MUST verify peer operating-system credentials (`SO_PEERCRED` on Linux, `getpeereid` on macOS) before exchanging frames; connections from differing UIDs are rejected immediately.
* **Rule 1.4:** When multiple `lens` commands start concurrently without a running instance, endpoint acquisition MUST be an atomic election. The winner initializes the desktop window; losing instances wait with bounded backoff for the endpoint to become ready, then forward their targets over IPC.
* **Rule 1.5:** Stale socket files from terminated instances MUST be recovered automatically after testing connection failure.

### Rule 2: CLI Command Handoff & Bounded Framing
* **Rule 2.1:** IPC communication MUST use 4-byte big-endian length-prefixed framing with a hard cap of `MAX_FRAME_BYTES = 64 * 1024` (64 KB). Oversized or malformed frames are rejected immediately without unbounded buffering.
* **Rule 2.2:** Filesystem paths (`target` and `invocation_directory`) MUST use lossless platform-native byte/unit representations (`WirePath`) rather than lossy Unicode strings.
* **Rule 2.3:** When the desktop application receives `OpenRequest`, it resolves the target and assigns it to the appropriate tab.
* **Rule 2.4:** If the target is already open in an existing tab, the application switches active focus to that tab; otherwise, it appends a new tab.
* **Rule 2.5:** The desktop application requests the window manager to un-minimize and focus the window.

### Rule 3: In-Process Presentation & Passive Diagram Security
* **Rule 3.1:** The Dioxus desktop window uses `tao` for window management and `wry` for webview rendering.
* **Rule 3.2:** Markdown parsing uses `pulldown-cmark`, generating sanitized HTML rendered directly into the Dioxus component tree via `dangerous_inner_html`.
* **Rule 3.3 (Passive SVG Boundary):** PlantUML diagrams MUST be rendered strictly behind a passive `<img>` boundary (e.g. `data:image/svg+xml;base64,...` or custom asset protocol) rather than injecting raw SVGs into the live DOM, strictly blocking script execution and event-handler injection from untrusted diagram responses.
* **Rule 3.4:** Mermaid diagram blocks (`<pre class="mermaid">`) trigger client-side Mermaid.js evaluation via Dioxus DOM evaluation hooks (`document::eval`) upon tab mount or document refresh without external network calls.
* **Rule 3.5:** No loopback HTTP server or query tokens are required for the desktop webview.

### Rule 4: Per-Tab Workspace Context & Live Watching
* **Rule 4.1:** Each open tab retains its own independent `WorkspaceContext` containing its canonical document root, discovered document set, target scope, source-link resolver, and PlantUML server configuration.
* **Rule 4.2:** Opening documents from multiple distinct repositories across separate tabs preserves independent link resolution and discovery sets without mutual interference.
* **Rule 4.3:** File-system watchers update tab content reactively upon on-disk changes without resetting scroll position or discarding open tab state.

### Rule 5: Platform Display Environment Handling
* **Rule 5.1:** On Linux, if both `DISPLAY` and `WAYLAND_DISPLAY` are unset, Lens does not attempt to initialize graphical windows; it outputs an actionable diagnostic message directing the user to `--server`.
* **Rule 5.2:** On macOS and Windows, platform-native GUI subsystem availability is used; macOS desktop launches MUST NOT require `DISPLAY` or `WAYLAND_DISPLAY`.
* **Rule 5.3:** If invoked with `--server` in any environment, Lens starts or forwards to the headless loopback HTTP service.

---

## 5. Concrete Examples & Counter-Examples

### Example 1.1: Cold Start Launches Desktop Application
* **Given:** No Lens process is running under the current user.
* **When:** User executes `lens docs/` from `/home/user/project`.
* **Then:** A single Lens desktop window opens, centered on the screen, displaying `docs/index.md` or the directory catalog in Tab 1.

### Example 1.2: Second Invocation Adds Tab and Focuses Window
* **Given:** Lens desktop window is currently open showing `docs/README.md`.
* **When:** User runs `lens src/architecture.puml` in a shell.
* **Then:** The shell returns within 200ms with `Opened 'src/architecture.puml' in active Lens window.`; the desktop window displays a new tab with the rendered PlantUML diagram and gains window focus.

### Example 1.3: Multi-Repository Tab Isolation
* **Given:** Tab 1 displays `README.md` in `/home/user/repo-a`.
* **When:** User runs `lens /home/user/repo-b/docs/api.md`.
* **Then:** Tab 2 opens with `docs/api.md` scoped to `/home/user/repo-b`. Relative links in Tab 1 resolve to `repo-a`, and relative links in Tab 2 resolve to `repo-b`.

### Example 1.4: Concurrent Cold Starts
* **Given:** Two `lens` commands run concurrently while the application is not running.
* **When:** Both commands execute startup.
* **Then:** One command wins the atomic endpoint election and launches the desktop window; the other waits for readiness, forwards its target over IPC, and exits code 0. Exactly one window is opened with two tabs.

### Example 1.5: Passive PlantUML SVG Sanitization
* **Given:** A PlantUML server returns an SVG containing `<script>alert(1)</script>` or `<svg onload="evil()">`.
* **When:** The document renders in the Dioxus webview.
* **Then:** The diagram renders through `<img src="data:image/svg+xml;base64,...">`; the browser engine disables script execution and event handlers, preventing DOM access.

---

## 6. Non-Functional Requirements

| Dimension | Requirement |
|---|---|
| **Performance** | CLI handoff to running instance MUST complete within 250ms end-to-end. Window launch on cold start MUST be responsive (< 1.5s on desktop hardware). |
| **Security** | The IPC endpoint MUST verify peer OS UID credentials (`SO_PEERCRED`/`getpeereid`), reject oversized frames (> 64KB), and isolate PlantUML SVGs behind passive `<img>` boundaries. |
| **Reliability** | Stale socket files from terminated instances MUST be recovered automatically. Concurrent cold starts coordinate atomically without lost targets or duplicate windows. |
| **Cross-Platform** | Native GUI availability correctly supports Linux (Wayland/X11 check), macOS (AppKit native), and Windows (Win32 session). Lossless `WirePath` preserves platform-native paths. |

---

## 7. Traceability

- **Use Cases:** [`docs/features/desktop-application/use-cases.md`](use-cases.md) (`UC-13`, `UC-14`, `UC-15`)
- **Technical Design:** [`docs/features/desktop-application/desktop-app-technical-design.md`](desktop-app-technical-design.md)
- **Architecture Decision:** [`docs/decisions/adr-026-dioxus-desktop-application.md`](../../decisions/adr-026-dioxus-desktop-application.md)
