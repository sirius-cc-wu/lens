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

### Rule 1: Single-Instance Window Enforcement
* **Rule 1.1:** Exactly one desktop application window may run per operating-system user.
* **Rule 1.2:** Single-instance coordination MUST use an authenticated local IPC channel (Unix domain socket on POSIX, Named Pipe on Windows) located in `$XDG_RUNTIME_DIR/lens.sock` (or `%LOCALAPPDATA%\lens\lens.ipc`).
* **Rule 1.3:** If an unlinked or stale socket file exists from a crashed instance, the launching process MUST test connectivity. If connection is refused, the stale socket file is removed and a new listener established.

### Rule 2: CLI Command Handoff
* **Rule 2.1:** Invoking `lens [TARGET]` when an instance is already running MUST serialize an `OpenTarget` payload to the IPC socket and exit with code `0` immediately after receiving acknowledgment.
* **Rule 2.2:** When the desktop application receives `OpenTarget`, it MUST resolve the target against the target's enclosing git repository or canonical directory root.
* **Rule 2.3:** If the target is already open in an existing tab, the application MUST switch active focus to that tab.
* **Rule 2.4:** If the target is not open, the application MUST append a new tab and focus it.
* **Rule 2.5:** The desktop application MUST request window focus and un-minimize if minimized.

### Rule 3: In-Process Webview & Diagram Rendering
* **Rule 3.1:** The Dioxus desktop window uses `tao` for window management and `wry` for webview rendering.
* **Rule 3.2:** Markdown parsing MUST continue using `pulldown-cmark`, generating sanitized HTML rendered directly into the Dioxus component tree via `dangerous_inner_html`.
* **Rule 3.3:** PlantUML diagrams MUST be fetched as SVGs from the configured PlantUML server (default: `https://www.plantuml.com/plantuml`) and embedded directly into the DOM.
* **Rule 3.4:** Mermaid diagram blocks (`<pre class="mermaid">`) MUST trigger client-side Mermaid.js evaluation via Dioxus DOM evaluation hooks (`document::eval`) upon tab mount or document refresh.
* **Rule 3.5:** No loopback HTTP server or query tokens are required for the desktop webview; asset delivery and RPC communication occur entirely in-process.

### Rule 4: Live File Watching & State Preservation
* **Rule 4.1:** The desktop application watches open files and their document roots for file-system modifications.
* **Rule 4.2:** Modifying a document on disk triggers reactive re-render of that document's tab without resetting scroll position or discarding open tab state.

### Rule 5: Headless & Non-Display Environment Handling
* **Rule 5.1:** If `lens` is invoked in an environment where no graphical display server is available (e.g., `DISPLAY` and `WAYLAND_DISPLAY` are unset on Linux), the command MUST NOT panic.
* **Rule 5.2:** If invoked with `--server` or when headless fallback is enabled, Lens starts or forwards to the loopback HTTP service. Otherwise, it exits with code `1` and outputs: `error: No graphical display server detected. Run with --server to start a loopback browser session.`

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

### Example 1.3: Target Already Open
* **Given:** Tab 1 is `docs/README.md` and Tab 2 is `docs/spec.md`. Active tab is Tab 1.
* **When:** User runs `lens docs/spec.md`.
* **Then:** No new tab is opened; Tab 2 becomes the active tab; the window is brought to the foreground.

### Example 1.4: Invalid Target Path
* **Given:** User executes `lens nonexistent.md`.
* **When:** Target validation executes.
* **Then:** The CLI prints `error: Target 'nonexistent.md' does not exist or is not readable.` to stderr and exits with code `1`. The running desktop window is not modified.

---

## 6. Non-Functional Requirements

| Dimension | Requirement |
|---|---|
| **Performance** | CLI handoff to running instance MUST complete within 250ms end-to-end. Window launch on cold start MUST be responsive (< 1.5s on desktop hardware). |
| **Security** | The IPC socket MUST enforce `0600` permissions (read/write only by the invoking user). Foreign users cannot send targets or trigger window events. |
| **Reliability** | Stale socket files from terminated instances MUST be recovered automatically without user intervention or manual cleanup commands. |
| **Dependencies** | Pure Rust workspace; no Node.js, npm, or external frontend build steps required for compilation. Linux releases document `libwebkit2gtk-4.1` package dependency. |

---

## 7. Traceability

- **Use Cases:** [`docs/features/desktop-application/use-cases.md`](use-cases.md) (`UC-13`, `UC-14`, `UC-15`)
- **Technical Design:** [`docs/features/desktop-application/desktop-app-technical-design.md`](desktop-app-technical-design.md)
- **Architecture Decision:** [`docs/decisions/adr-026-dioxus-desktop-application.md`](../../decisions/adr-026-dioxus-desktop-application.md)
