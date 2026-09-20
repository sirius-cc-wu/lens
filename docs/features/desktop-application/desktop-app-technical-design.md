---
type: "Technical Design"
title: "FEAT-05: Dioxus Desktop Application Technical Design"
description: "Detailed architecture, trade-off debate, component structure, IPC protocol, and lifecycle design for the Lens Dioxus desktop application."
id: "FEAT-05-DESIGN"
status: "active"
scope: "Lens"
tags: [design, architecture, desktop, dioxus, ipc, webview]
---

# FEAT-05: Dioxus Desktop Application Technical Design

## 1. Architectural Debate & Dilemmas

Transitioning Lens from a loopback HTTP daemon into a native desktop application introduces critical design choices. This section evaluates the key architectural debates to converge on a qualified, robust design.

---

### Debate 1: Workspace Structure — Monolithic Binary vs. Multi-Crate Workspace

```
Option A: Monolithic Binary                   Option B: Multi-Crate Workspace
┌───────────────────────────────┐              ┌───────────────────────────────┐
│           lens binary         │              │          lens (CLI)           │
│  - core discovery & parser    │              └───────────────┬───────────────┘
│  - dioxus desktop UI (wry)    │                              │ (IPC or direct)
│  - HTTP daemon (axum)         │              ┌───────────────▼───────────────┐
└───────────────────────────────┘              │     lens-app (Dioxus GUI)     │
                                               └───────────────┬───────────────┘
                                                               │ (imports)
                                               ┌───────────────▼───────────────┐
                                               │   lens-core (Domain Logic)    │
                                               └───────────────────────────────┘
```

* **Option A (Monolithic Crate):** Keep a single crate with Cargo feature flags (`cargo build --features desktop`).
  * *Pros:* Single `Cargo.toml`; simplest code movement.
  * *Cons:* Coupling WebKitGTK and `tao`/`wry` dependencies to the entire codebase. Makes pure headless testing or CLI builds slow and dependent on graphics headers.
* **Option B (Multi-Crate Workspace — Qualified Choice):** Split into:
  1. `lens-core`: Pure Rust library for document discovery, root detection, Markdown parsing ([`pulldown-cmark`](../../Cargo.toml#L24)), PlantUML encoding, and cache management. Zero GUI or WebKit dependencies.
  2. `lens-app`: The Dioxus desktop UI application binary, containing the `tao` window shell, `wry` webview, tab management, and reactive components.
  3. `lens` (or `lens-cli`): The command-line dispatcher that handles target resolution, single-instance detection, IPC forwarding, and fallback.
  * *Verdict:* **Adopt Option B.** Clean separation of concerns ensures that `lens-core` can be thoroughly unit-tested without graphical dependencies, and builds remain fast and modular.

---

### Debate 2: Single-Instance Inter-Process Communication (IPC)

When a user executes `lens docs/spec.md` from a terminal while Lens Desktop is open, how should the CLI forward the target to the existing window?

* **Option A: Loopback HTTP Control Server (Axum on a local port):**
  * The desktop application binds a random loopback port (e.g., `127.0.0.1:PORT`) and writes port + token to a metadata file.
  * *Trade-off:* Requires TCP port allocation, security tokens to prevent cross-user probing, and handling firewall alerts on some platforms.
* **Option B: Local Domain Sockets / Named Pipes (Qualified Choice):**
  * Unix Domain Socket on POSIX (`$XDG_RUNTIME_DIR/lens.sock` or `/tmp/lens-$UID.sock`).
  * Named Pipe on Windows (`\\.\pipe\lens-$USERNAME`).
  * Communication uses line-delimited JSON messages (`{"action":"open_target","path":"...","scope":"..."}`).
  * *Trade-off:* Filesystem-based access control (`0600` permissions) strictly limits communication to the invoking OS user without cryptographic tokens. Connection refused cleanly identifies stale crashed sockets.
  * *Verdict:* **Adopt Option B.** Maximum speed (< 10ms handshake), zero network stack overhead, and native OS user isolation.

---

### Debate 3: Presentation Model inside Dioxus — Loopback Server vs. In-Process Asset Serving

* **Option A: Dioxus Webview points to an embedded Axum HTTP server (`http://127.0.0.1:port`):**
  * *Trade-off:* Retains current Axum routing, but preserves HTTP roundtrips, port management, and latency inside the local desktop app.
* **Option B: Pure In-Process RSX + Custom Protocol Handler (Qualified Choice):**
  * Dioxus components render the application shell (tabs, sidebar, controls).
  * Markdown body is rendered directly into the webview DOM using `dangerous_inner_html`.
  * Local image and diagram assets are loaded either via `wry`'s custom protocol handler (`lens://assets/...`) or inlined as data/SVG strings.
  * *Verdict:* **Adopt Option B.** Eliminates all network overhead and loopback port dependencies from the desktop viewing experience.

---

### Debate 4: Client-Side Mermaid.js & PlantUML Execution Lifecycle

* **PlantUML:**
  * Fetched asynchronously by `lens-core` via `reqwest` from the configured PlantUML server.
  * Returned SVG strings are cached in memory and injected directly into the DOM tree inside a `<div class="plantuml-diagram">`.
* **Mermaid.js:**
  * Bundled directly into the `lens-app` binary via `include_str!("../assets/mermaid.min.js")`.
  * Injected into the `wry` webview at startup using `with_initialization_script`.
  * On every tab switch or document re-render, a Dioxus `use_effect` fires `document::eval("mermaid.run({ querySelector: '.mermaid' });")`.
  * *Verdict:* Instant, offline rendering of Mermaid diagrams without external network calls; clean separation between async PlantUML network fetches and local Mermaid rendering.

---

## 2. System Architecture & Component Model

```
┌────────────────────────────────────────────────────────────────────────┐
│                         Lens Desktop Application                       │
│                                                                        │
│ ┌────────────────────────────────────────────────────────────────────┐ │
│ │                         AppShell (Dioxus)                          │ │
│ │                                                                    │ │
│ │ ┌──────────────────────┐ ┌───────────────────────────────────────┐ │ │
│ │ │ TabBar Component     │ │ Quick Actions & Status               │ │ │
│ │ │ [spec.md ×] [arch.puml]│ │ Root: /home/user/project             │ │ │
│ │ └──────────────────────┘ └───────────────────────────────────────┘ │ │
│ │ ┌──────────────────┬─────────────────────────────────────────────┐ │ │
│ │ │ Drawer / Catalog │ DocumentViewer                              │ │ │
│ │ │ - docs/          │ ┌─────────────────────────────────────────┐ │ │ │
│ │ │   - spec.md      │ │ # Architecture Specification            │ │ │ │
│ │ │   - api.md       │ │                                         │ │ │ │
│ │ │ - src/           │ │ ```mermaid                              │ │ │ │
│ │ │   - main.rs      │ │ graph TD; ...                           │ │ │ │
│ │ │                  │ │ ```                                     │ │ │ │
│ │ │                  │ └─────────────────────────────────────────┘ │ │ │
│ │ └──────────────────┴─────────────────────────────────────────────┘ │ │
│ └────────────────────────────────────────────────────────────────────┘ │
│                                    │                                   │
│                                    ▼                                   │
│ ┌────────────────────────────────────────────────────────────────────┐ │
│ │                   Local IPC Listener (Unix Socket)                 │ │
│ │                  $XDG_RUNTIME_DIR/lens.sock                        │ │
│ └──────────────────────────────────▲─────────────────────────────────┘ │
└────────────────────────────────────┼───────────────────────────────────┘
                                     │ (OpenTarget JSON)
                        ┌────────────┴───────────┐
                        │    CLI: lens <path>    │
                        └────────────────────────┘
```

---

## 3. IPC Message Protocol Specification

The CLI and Desktop Application communicate over the local socket using line-delimited JSON:

### Request: `OpenTarget`
```json
{
  "version": 1,
  "action": "open_target",
  "target": "docs/architecture.md",
  "invocation_directory": "/home/user/project",
  "scope": "repository",
  "plantuml_server": null
}
```

### Response: `OpenTargetResponse`
```json
{
  "version": 1,
  "status": "ok",
  "tab_id": "tab-3",
  "message": "Target opened in active window."
}
```

### Request: `StopApp`
```json
{
  "version": 1,
  "action": "stop"
}
```

### Response: `StopAppResponse`
```json
{
  "version": 1,
  "status": "ok",
  "message": "Lens desktop application shutting down."
}
```

---

## 4. Dioxus Reactive State Hierarchy

```rust
pub struct AppState {
    pub active_root: PathBuf,
    pub tabs: Vec<TabState>,
    pub active_tab_index: usize,
    pub drawer_open: bool,
    pub discovered_catalog: Vec<DocumentEntry>,
}

pub struct TabState {
    pub id: String,
    pub file_path: PathBuf,
    pub title: String,
    pub rendered_html: String,
    pub scroll_offset: f64,
    pub is_modified_on_disk: bool,
}
```

1. **`use_signal(|| AppState)`**: Holds global window state.
2. **`use_coroutine` for IPC Reception**: Spawns an async loop listening on the Unix Domain Socket; when an `OpenTarget` arrives, it invokes `AppState::open_or_switch_tab(target)` and calls `window.set_focus()`.
3. **`use_coroutine` for File Watching**: Watches paths referenced by active tabs using `notify`; on change, re-parses through `lens-core` and updates the active `TabState.rendered_html`.

---

## 5. Platform Dependencies & Linux Packaging

* **Linux Requirements:**
  * Build-time: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`.
  * Runtime: `libwebkit2gtk-4.1-0`, `libgtk-3-0`.
* **Debian Packaging (`debs/`):**
  * Update package control file to specify `Depends: libwebkit2gtk-4.1-0 (>= 2.38), libgtk-3-0 (>= 3.24)`.
* **Headless Detection:**
  * If neither `DISPLAY` nor `WAYLAND_DISPLAY` is present on POSIX systems, `lens` CLI aborts before attempting to initialize `tao`, reporting clear guidance.

---

## 6. Traceability

- **Requirements:** [`docs/features/desktop-application/desktop-app-requirements.md`](desktop-app-requirements.md)
- **Use Cases:** [`docs/features/desktop-application/use-cases.md`](use-cases.md)
- **Architecture Decision:** [`docs/decisions/adr-026-dioxus-desktop-application.md`](../../decisions/adr-026-dioxus-desktop-application.md)
