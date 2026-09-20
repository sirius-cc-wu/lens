---
type: "Architecture Decision"
title: "ADR-026: Dioxus Desktop Application Shell and In-Process Presentation"
description: "Establishes a single-instance Dioxus desktop application using wry and tao, replacing loopback browser tabs with an integrated multi-tab window and local socket IPC handoff."
id: "ADR-026"
status: "accepted"
date: "2026-09-21"
tags: [architecture, decision, desktop, dioxus, ipc, webview]
owner_loop: spec-validate
phase_profile: design
---

# ADR-026: Dioxus Desktop Application Shell and In-Process Presentation

Status: accepted

Date: 2026-09-21

Supersedes [ADR-002: Loopback Viewer Scope](adr-002-loopback-viewer-scope.md) and [ADR-022: Per-User Background Service](adr-022-per-user-background-service.md) for desktop viewing workflows.

---

## Context

Lens was originally designed to serve rendered Markdown and diagrams through ephemeral loopback HTTP ports to external browser tabs. While this avoided GUI dependencies, it caused severe browser tab sprawl, lacked OS-level window and tab management, required complex loopback token authentication to safeguard local files, and created friction when coordinating interactive tools like coding agents.

Lens requires an integrated, high-performance desktop application interface while preserving its fast, non-blocking CLI terminal workflow and 100% pure Rust codebase.

---

## Decision

Lens adopts **Dioxus (`dioxus-desktop`)** to provide a dedicated, single-instance native desktop application window with in-process webview presentation and local IPC command forwarding.

### 1. Multi-Crate Workspace Architecture

Lens is refactored into three modular crates:
1. `lens-core`: Pure Rust library owning document discovery, root detection, Markdown parsing ([`pulldown-cmark`](../../Cargo.toml#L24)), PlantUML encoding, and cache state. Completely free of GUI or WebKit dependencies.
2. `lens-app`: The Dioxus desktop application binary, containing the `tao` window shell, `wry` webview, multi-tab workspace state, and reactive UI components.
3. `lens` (CLI): The command-line dispatcher that resolves target paths, queries the single-instance IPC socket, forwards commands to the running desktop application, or starts the application when inactive.

### 2. Single-Instance Local IPC Channel & Endpoint Security

- Exactly one Lens desktop window executes per operating-system user.
- Endpoint security retains the proven guarantees of ADR-022:
  - On POSIX systems, the endpoint resides in a verified user-owned private runtime directory (`0700` permissions) under `$XDG_RUNTIME_DIR/lens` (or `/tmp/lens-$UID/`).
  - The client and server verify operating-system peer credentials (`SO_PEERCRED` on Linux, `getpeereid` on macOS) before exchanging payloads, rejecting any connection from a differing UID.
  - On Windows, the named pipe (`\\.\pipe\lens-$USERNAME`) enforces an explicit current-user security descriptor (ACL).
- Atomic Cold-Start Election: When multiple `lens` commands start concurrently, endpoint creation is atomic. The winner binds the listener and initializes the desktop application; losing instances await listener readiness with a bounded timeout and exponential backoff, then forward their requests as clients without spawning duplicate windows.
- Bounded Framed Protocol & Native Paths:
  - Communication uses the bounded 4-byte length-prefixed framing established in ADR-022 with `MAX_FRAME_BYTES = 64 * 1024` (64 KB). Frames exceeding this limit or failing framing validation are rejected immediately without unbounded buffering.
  - Filesystem paths (`target` and `invocation_directory`) use lossless platform-native representation (`WirePath::Unix(Vec<u8>)` and `WirePath::Windows(Vec<u16>)`).

### 3. In-Process Presentation, Tab Isolation & Passive Diagram Boundaries

- The desktop application renders UI layout and document bodies in-process using Dioxus components and `wry`.
- No loopback HTTP server or query-parameter authentication tokens are used for desktop viewing.
- **Per-Tab Workspace Context:** Each open tab owns its own independent `WorkspaceContext` encapsulating its canonical document root, discovered document set, target scope, source-link resolver, and PlantUML server configuration. Opening documents across distinct repositories retains strict project-level isolation without cross-repository catalog pollution.
- **Passive Diagram Security Boundary:** PlantUML diagrams are fetched asynchronously from the configured server and rendered strictly behind a passive `<img>` boundary (e.g. via `data:image/svg+xml;base64,...` or an isolated custom asset protocol). SVGs are never injected directly as active elements into the live DOM tree, preventing script execution and DOM event-handler injection from untrusted diagram responses.
- Mermaid.js is bundled into the application binary, injected into the webview at startup, and executed client-side via Dioxus DOM evaluation hooks (`document::eval`) upon document mounting.

### 4. Non-Display and Headless Environments

- On Linux, if neither `DISPLAY` nor `WAYLAND_DISPLAY` is set, the `lens` CLI command will not attempt to spawn graphical windows.
- On macOS and Windows, platform-native GUI subsystem availability is used; macOS desktop launches do not require or inspect `DISPLAY` or `WAYLAND_DISPLAY`.
- Users in headless or remote SSH environments can explicitly run `lens --server [TARGET]` to start the headless loopback HTTP server, preserving remote terminal workflows.

---

## Consequences

### Capabilities & Guarantees
- Single-window workspace: Multiple documents and diagrams open in tabs within a single application window rather than littering the user's web browser tabs.
- Cross-project safety: Opening multiple repositories across tabs preserves separate document roots and link resolution rules without mutual interference.
- Instant CLI handoff: Running `lens <path>` in a terminal switches to or opens a tab in the desktop application in under 200ms.
- Active SVG containment: PlantUML diagram responses cannot execute arbitrary JavaScript or manipulate the host DOM.
- Authenticated IPC: Peer-credential checks and private directories prevent endpoint impersonation and cross-user snooping.
- Bounded framing: 64 KB maximum frame size prevents memory exhaustion from oversized or malformed client payloads.
- Lossless paths: Arbitrary byte sequences on Unix and unpaired UTF-16 code units on Windows are preserved without lossy Unicode conversion.
- Offline Mermaid rendering: Mermaid diagrams render locally without network access.

### Accepted Trade-Offs & Constraints
- Platform dependencies: Linux desktop installations require `libwebkit2gtk-4.1-0` and `libgtk-3-0`.
- Headless environments require explicit `--server` flag to run headless HTTP viewing.

### Invariants Preserved
- Canonical containment: Documents and source links remain strictly contained within each tab's individual discovered document root.
- User isolation: The local IPC socket strictly requires authenticated matching operating-system user identity.

---

## Trace

- Use cases: [`docs/features/desktop-application/use-cases.md`](../features/desktop-application/use-cases.md) (`UC-13`, `UC-14`, `UC-15`)
- Requirements: [`docs/features/desktop-application/desktop-app-requirements.md`](../features/desktop-application/desktop-app-requirements.md) (`FEAT-05-REQ`)
- Technical design: [`docs/features/desktop-application/desktop-app-technical-design.md`](../features/desktop-application/desktop-app-technical-design.md)
