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
### Debate 2: Single-Instance Inter-Process Communication (IPC)

When a user executes `lens docs/spec.md` from a terminal while Lens Desktop is open, how should the CLI forward the target to the existing window?

* **Option A: Loopback HTTP Control Server (Axum on a local port):**
  * The desktop application binds a random loopback port (e.g., `127.0.0.1:PORT`) and writes port + token to a metadata file.
  * *Trade-off:* Requires TCP port allocation, security tokens to prevent cross-user probing, and handling firewall alerts on some platforms.
* **Option B: Bounded Local Sockets with OS Peer-Credential Verification (Qualified Choice):**
  * Unix Domain Socket on POSIX located in a verified user-owned private runtime directory (`0700`) under `$XDG_RUNTIME_DIR/lens/lens.sock` (or `/tmp/lens-$UID/lens.sock`).
  * Named Pipe on Windows (`\\.\pipe\lens-$USERNAME`) with explicit current-user security descriptor (ACL).
  * **Peer Authentication:** On Unix, both client and server query operating-system peer credentials (`SO_PEERCRED` on Linux, `getpeereid` on macOS) immediately after connection to verify the peer's UID matches the invoking user's UID. Connections from differing UIDs are rejected before reading any bytes.
  * **Bounded Framing:** Communication reuses the 4-byte big-endian length-prefixed framing established in ADR-022, with a hard `MAX_FRAME_BYTES = 64 * 1024` (64 KB). Any client attempting to send an oversized or unterminated frame is disconnected immediately without unbounded memory allocation.
  * *Verdict:* **Adopt Option B.** Sub-millisecond handshake, zero network stack overhead, strict OS peer isolation, and immunity to memory exhaustion attacks.

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

* **PlantUML (Passive Image Boundary):**
  * Fetched asynchronously by `lens-core` via `reqwest` from the configured PlantUML server.
  * **Security Boundary:** To prevent malicious or compromised PlantUML servers from injecting active scripts, event handlers (`onload`, `onclick`), or foreign DOM nodes into the desktop webview, returned SVGs are **never injected directly as raw markup into the live DOM**.
  * SVGs are encoded as passive data URIs (`<img class="plantuml-diagram" src="data:image/svg+xml;base64,..." alt="...">`) or served via an isolated custom asset protocol (`lens://plantuml/{hash}`). Browsers and webviews enforce that `<img>` SVG representations execute no scripts and cannot access the parent DOM.
* **Mermaid.js:**
  * Bundled directly into the `lens-app` binary via `include_str!("../assets/mermaid.min.js")`.
  * Injected into the `wry` webview at startup using `with_initialization_script`.
  * On every tab switch or document re-render, a Dioxus `use_effect` fires `document::eval("mermaid.run({ querySelector: '.mermaid' });")`.
  * *Verdict:* Instant, offline rendering of Mermaid diagrams without external network calls; clean separation between async PlantUML network fetches and local Mermaid rendering, with complete neutralization of active SVG payloads.

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
│ │ │ TabBar Component     │ │ Active Workspace Status               │ │ │
│ │ │ [spec.md ×] [arch.puml]│ │ Root: /home/user/project-A            │ │ │
│ │ └──────────────────────┘ └───────────────────────────────────────┘ │ │
│ │ ┌──────────────────┬─────────────────────────────────────────────┐ │ │
│ │ │ Drawer / Catalog │ DocumentViewer (Scoped to Tab Context)      │ │ │
│ │ │ (Active Tab Root)│ ┌─────────────────────────────────────────┐ │ │ │
│ │ │ - docs/          │ │ # Architecture Specification            │ │ │ │
│ │ │   - spec.md      │ │                                         │ │ │ │
│ │ │   - api.md       │ │ <img src="data:image/svg+xml;base64,..">│ │ │ │
│ │ │ - src/           │ │ ```mermaid                              │ │ │ │
│ │ │                  │ │ graph TD; ...                           │ │ │ │
│ │ │                  │ └─────────────────────────────────────────┘ │ │ │
│ │ └──────────────────┴─────────────────────────────────────────────┘ │ │
│ └────────────────────────────────────────────────────────────────────┘ │
│                                    │                                   │
│                                    ▼                                   │
│ ┌────────────────────────────────────────────────────────────────────┐ │
│ │              Local IPC Listener (Length-Prefixed Framing)          │ │
│ │           $XDG_RUNTIME_DIR/lens/lens.sock (Peer UID Verified)      │ │
│ └──────────────────────────────────▲─────────────────────────────────┘ │
└────────────────────────────────────┼───────────────────────────────────┘
                                     │ (Framed OpenRequest: 64KB Bound)
                        ┌────────────┴───────────┐
                        │    CLI: lens <path>    │
                        └────────────────────────┘
```

---

## 3. IPC Message Protocol Specification

The CLI and Desktop Application communicate over the local socket using 4-byte big-endian length-prefixed frames (max 64 KB) containing typed JSON payloads with lossless platform-native paths:

### Request: `OpenRequest`
```json
{
  "protocol_version": 1,
  "request_id": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
  "invocation_directory": {
    "platform": "unix",
    "units": [47, 104, 111, 109, 101, 47, 117, 115, 101, 114]
  },
  "target": {
    "platform": "unix",
    "units": [100, 111, 99, 115, 47, 115, 112, 101, 99, 46, 109, 100]
  },
  "scope": "repository",
  "plantuml_server": null
}
```

### Response: `OpenResponse`
```json
{
  "protocol_version": 1,
  "request_id": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
  "status": "ok",
  "tab_id": "tab-3",
  "message": "Target opened in active window."
}
```

### Request: `StopRequest`
```json
{
  "protocol_version": 1,
  "request_id": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
}
```

### Response: `StopResponse`
```json
{
  "protocol_version": 1,
  "request_id": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
  "status": "ok",
  "message": "Lens desktop application shutting down."
}
```

---

## 4. Dioxus Reactive State & Per-Tab Workspace Isolation

To prevent cross-repository contamination when a developer works across multiple repositories in separate tabs, all document discovery, source links, and scope state are encapsulated in an immutable `WorkspaceContext` attached to each tab:

```rust
pub struct AppState {
    pub tabs: Vec<TabState>,
    pub active_tab_index: usize,
    pub drawer_open: bool,
}

pub struct TabState {
    pub id: String,
    pub file_path: PathBuf,
    pub title: String,
    pub rendered_html: String,
    pub scroll_offset: f64,
    pub is_modified_on_disk: bool,
    pub workspace: Arc<WorkspaceContext>,
}

pub struct WorkspaceContext {
    pub document_root: PathBuf,
    pub discovered_documents: HashSet<PathBuf>,
    pub source_link_resolver: SourceLinkResolver,
    pub scope: TargetScope,
    pub plantuml_server: Option<String>,
}
```

* Switching between Tab 1 (Repository A) and Tab 2 (Repository B) seamlessly switches the active `WorkspaceContext`.
* Relative links inside Tab 1 resolve strictly against Repository A's root; links inside Tab 2 resolve strictly against Repository B's root.
* Modifying a document on disk in Repository A only triggers a refresh on tabs associated with that repository's context.

---

## 5. Platform Dependencies, Display Detection & Concurrent Startup

### Concurrent Cold-Start Election
When two or more `lens` commands start concurrently when no application is running:
1. Both commands observe that no server is responding.
2. Both attempt atomic creation of a lockfile or atomic binding of the domain socket in the private runtime directory.
3. The winner of the election assumes server responsibility and spawns `lens-app`.
4. The loser(s) observe the socket lock, enter a client poll loop with exponential backoff (e.g., polling every 50ms up to 5 seconds) waiting for the winner's endpoint to acknowledge a ping.
5. Once the endpoint is verified ready and authenticated, the losing instance sends its `OpenRequest` over the IPC socket and exits code 0.
6. **Result:** Exactly one desktop window is launched, and both targets are opened as distinct tabs.

### Platform Display Detection
* **Linux:** Checks whether `DISPLAY` or `WAYLAND_DISPLAY` is non-empty. If both are unset (e.g., SSH session, headless container), GUI launch is aborted with an actionable error directing the user to `--server`.
* **macOS:** Does not inspect `DISPLAY` or `WAYLAND_DISPLAY`. Availability is determined by the macOS WindowServer/AppKit environment.
* **Windows:** Uses Win32 GUI session detection (`GetSystemMetrics(SM_REMOTESESSION)` / desktop availability).

---

## 6. Traceability

- **Requirements:** [`docs/features/desktop-application/desktop-app-requirements.md`](desktop-app-requirements.md)
- **Use Cases:** [`docs/features/desktop-application/use-cases.md`](use-cases.md)
- **Architecture Decision:** [`docs/decisions/adr-026-dioxus-desktop-application.md`](../../decisions/adr-026-dioxus-desktop-application.md)
