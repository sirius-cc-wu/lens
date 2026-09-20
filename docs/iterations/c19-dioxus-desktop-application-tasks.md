---
type: "Iteration Plan & Task Board"
title: "Iteration C19: Dioxus Desktop Application Implementation"
description: "Structured work packages, test specifications, and verification checklists for downstream Workers implementing the Dioxus desktop application."
id: "C19"
status: "ready"
tags: [iteration, tasks, planning, worker-handoff, desktop, dioxus]
---

# Iteration C19: Dioxus Desktop Application Implementation

## Overview

This task board breaks the architectural specification of [ADR-026](../decisions/adr-026-dioxus-desktop-application.md), [FEAT-05-REQ](../features/desktop-application/desktop-app-requirements.md), [SSD-08](../features/desktop-application/ssd-08-open-target-in-application.md), [OC-08](../features/desktop-application/oc-08-open-target-in-application.md), [SSD-09](../features/desktop-application/ssd-09-command-line-handoff.md), [OC-09](../features/desktop-application/oc-09-command-line-handoff.md), and [Technical Design](../features/desktop-application/desktop-app-technical-design.md) into 5 discrete, test-driven vertical slices for execution by downstream Workers.

---

## Work Packages & Execution Slices

### Slice 1: Modular Crate Extraction (`crates/lens-core`)

**Objective:** Decouple document discovery, Markdown parsing, and PlantUML rendering from GUI/network transport into a pure Rust library `lens-core`.

- [ ] **Task 1.1:** Restructure repository as a Cargo workspace with root `Cargo.toml`.
- [ ] **Task 1.2:** Create `crates/lens-core` and migrate:
  - `src/discovery.rs`
  - `src/markdown.rs`
  - `src/plantuml.rs`
  - `src/source_link.rs`
- [ ] **Task 1.3:** Ensure `crates/lens-core` has no dependencies on `wry`, `tao`, or `gtk`.
- [ ] **Task 1.4:** Verify all existing unit tests in `lens-core` pass (`cargo test -p lens-core --locked`).

---

### Slice 2: Local IPC Protocol & Single-Instance Listener (`crates/lens-ipc`)

**Objective:** Implement the local domain socket / named pipe IPC channel with 4-byte length-prefixed framing (64 KB bound), peer credential verification, and atomic election for single-instance coordination.

- [ ] **Task 2.1:** Define protocol types with lossless platform-native paths:
  - `WirePath` (`Unix(Vec<u8>)`, `Windows(Vec<u16>)`), `RequestId`, `ProtocolVersion`.
  - `OpenRequest`, `OpenResponse`, `StopRequest`, `StopResponse`.
  - Enforce `MAX_FRAME_BYTES = 64 * 1024` (64 KB) with 4-byte big-endian framing.
- [ ] **Task 2.2:** Implement secure endpoint management:
  - Ensure private runtime directory exists with `0700` permissions (`$XDG_RUNTIME_DIR/lens` or `/tmp/lens-$UID/`).
  - Implement peer operating-system UID check (`SO_PEERCRED` on Linux, `getpeereid` on macOS, current-user ACL on Windows).
  - Reject connections from foreign UIDs before reading frames.
- [ ] **Task 2.3:** Implement atomic cold-start election & client retry:
  - File lock or atomic socket binding to elect a single primary application process.
  - Losers wait with exponential backoff (up to 5 seconds) for the primary endpoint to become ready, then forward their `OpenRequest`.
  - Stale socket detection: unlinks and rebinds if connection refused.
- [ ] **Task 2.4:** Write 3A unit and integration tests:
  - `unreachable_stale_socket_then_recovers_and_binds_new_listener`
  - `active_server_then_receives_and_acknowledges_open_target_message`
  - `peer_uid_mismatch_then_rejects_connection_before_payload`
  - `oversized_frame_exceeding_64kb_then_rejects_immediately_without_buffering`
  - `native_path_with_non_utf8_bytes_or_unpaired_surrogates_then_roundtrips_losslessly`
  - `simultaneous_cold_starts_then_elects_single_server_and_forwards_second_target`

---

### Slice 3: Dioxus Desktop Application Shell & Tab Isolation (`crates/lens-app`)

**Objective:** Construct the native desktop window using Dioxus (`dioxus-desktop`), managing multi-tab workspace state with strict per-tab repository isolation.

- [ ] **Task 3.1:** Initialize `crates/lens-app` with `dioxus` (0.5+), `dioxus-desktop`, and `lens-core`.
- [ ] **Task 3.2:** Define reactive state models with per-tab `WorkspaceContext`:
  - `AppState { tabs, active_tab_index, drawer_open }`
  - `TabState { id, file_path, title, rendered_html, scroll_offset, workspace: Arc<WorkspaceContext> }`
  - `WorkspaceContext { document_root, discovered_documents, source_link_resolver, scope, plantuml_server }`
- [ ] **Task 3.3:** Build UI component hierarchy:
  - `AppShell`: Top-level window layout.
  - `TabBar`: Render tab pills with close buttons, active highlighting, and dirty indicators.
  - `DocumentViewer`: Embeds sanitized Markdown HTML via `dangerous_inner_html`.
  - `NavigationDrawer`: Collapsible sidebar listing documents discovered for the active tab's workspace.
- [ ] **Task 3.4:** Mount IPC listener coroutine:
  - Subscribes to `IpcServer` events.
  - When `OpenRequest` is received: resolves target, constructs `WorkspaceContext`, appends or focuses tab, calls `window.set_focus()`.
- [ ] **Task 3.5:** Mount file-watcher coroutine:
  - Registers `notify` watchers for open tab paths.
  - On file modification: re-parses document via `lens-core` and updates tab HTML without resetting scroll position.
- [ ] **Task 3.6:** Write tab-isolation tests:
  - `tabs_from_distinct_repositories_then_retain_independent_document_roots_and_link_resolution`

---

### Slice 4: In-Process Diagram Rendering & Passive PlantUML Boundary (`crates/lens-app`)

**Objective:** Implement offline Mermaid.js diagram generation and passive PlantUML SVG embedding.

- [ ] **Task 4.1:** Bundle `mermaid.min.js` in `crates/lens-app/assets/` and register in `wry` webview via `with_initialization_script`.
- [ ] **Task 4.2:** Implement Dioxus `use_effect` hook inside `DocumentViewer`:
  - Evaluates `document::eval("if (window.mermaid) { mermaid.run({ querySelector: '.mermaid' }); }")` upon mount or tab change.
- [ ] **Task 4.3:** Implement asynchronous PlantUML SVG fetcher & passive boundary:
  - Scans document for PlantUML blocks.
  - Asynchronously requests SVGs from configured server via `reqwest`.
  - Encodes returned SVG into a passive `<img>` element (`data:image/svg+xml;base64,...`) to block active script execution.
- [ ] **Task 4.4:** Write passive diagram security test:
  - `plantuml_svg_with_active_scripts_then_rendered_behind_passive_img_without_script_execution`

---

### Slice 5: CLI Dispatcher & Platform Display Detection (`crates/lens-cli`)

**Objective:** Connect `lens` CLI command to the desktop application, with platform-specific display detection.

- [ ] **Task 5.1:** Update CLI argument parsing to support `lens [TARGET]`, `lens stop`, and `--server`.
- [ ] **Task 5.2:** Implement execution dispatch logic:
  - If `--server` is specified: launch legacy loopback HTTP service.
  - On Linux: if `DISPLAY` and `WAYLAND_DISPLAY` are both unset, report error directing user to `--server`.
  - On macOS and Windows: use native GUI session detection without checking `DISPLAY`/`WAYLAND_DISPLAY`.
  - Otherwise: send `OpenRequest` via `IpcClient`; if no server active, coordinate atomic election and launch `lens-app`.
- [ ] **Task 5.3:** End-to-end integration tests:
  - `cli_invoked_without_active_app_then_starts_desktop_window`
  - `cli_invoked_with_active_app_then_forwards_target_and_exits_zero`
  - `macos_desktop_launch_with_unset_display_variables_then_succeeds`

---

## Acceptance & Qualification Gate

Downstream Worker deliverables must satisfy:
1. `cargo fmt --check` and `cargo clippy --locked --all-targets --all-features -- -D warnings` clean.
2. 100% test pass across `lens-core`, `lens-ipc`, and `lens-cli`.
3. ADR-026 invariants verified:
   - Peer UID validation (`SO_PEERCRED`/`getpeereid`) prevents cross-user access.
   - PlantUML SVGs render exclusively behind passive `<img>` boundaries.
   - Bounded 64 KB framing rejects oversized inputs.
   - Per-tab `WorkspaceContext` guarantees multi-repo isolation.
   - macOS launches without requiring `DISPLAY` or `WAYLAND_DISPLAY`.
4. No network requests emitted for Mermaid rendering (offline guarantee).
