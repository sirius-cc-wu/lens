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

**Objective:** Implement the local domain socket / named pipe IPC channel for single-instance application coordination.

- [ ] **Task 2.1:** Define serde-compatible protocol types:
  - `OpenTarget { version, target, invocation_directory, scope, plantuml_server }`
  - `OpenTargetResponse { version, status, tab_id, message }`
  - `StopApp { version }`
- [ ] **Task 2.2:** Implement `IpcClient`:
  - Connects to `$XDG_RUNTIME_DIR/lens.sock` (or `/tmp/lens-$UID.sock` / Windows Named Pipe).
  - Sends line-delimited JSON payload with timeout.
  - Receives response and surfaces typed result.
- [ ] **Task 2.3:** Implement `IpcServer`:
  - Binds socket with `0600` permissions.
  - Detects stale socket files (connect attempt fails with `ECONNREFUSED` -> unlinks and rebinds).
  - Asynchronously emits incoming requests onto a channel for the UI event loop.
- [ ] **Task 2.4:** Write 3A unit and integration tests:
  - `unreachable_stale_socket_then_recovers_and_binds_new_listener`
  - `active_server_then_receives_and_acknowledges_open_target_message`
  - `unauthorized_cross_user_access_is_prevented_by_filesystem_permissions`

---

### Slice 3: Dioxus Desktop Application Shell (`crates/lens-app`)

**Objective:** Construct the native desktop window using Dioxus (`dioxus-desktop`), managing multi-tab workspace state and in-process document viewing.

- [ ] **Task 3.1:** Initialize `crates/lens-app` with `dioxus` (0.5+), `dioxus-desktop`, and `lens-core`.
- [ ] **Task 3.2:** Define reactive state models:
  - `AppState { active_root, tabs, active_tab_index, drawer_open }`
  - `TabState { id, file_path, title, rendered_html, scroll_offset }`
- [ ] **Task 3.3:** Build UI component hierarchy:
  - `AppShell`: Top-level window layout.
  - `TabBar`: Render tab pills with close buttons, active highlighting, and dirty indicators.
  - `DocumentViewer`: Embeds sanitized Markdown HTML via `dangerous_inner_html`.
  - `NavigationDrawer`: Collapsible sidebar listing discovered repository documents.
- [ ] **Task 3.4:** Mount IPC listener coroutine:
  - Subscribes to `IpcServer` events.
  - When `OpenTarget` is received: resolves document, appends or focuses tab, calls `window.set_focus()`.
- [ ] **Task 3.5:** Mount file-watcher coroutine:
  - Registers `notify` watchers for open tab paths.
  - On file modification: re-parses document via `lens-core` and updates tab HTML without resetting scroll position.

---

### Slice 4: In-Process Diagram Rendering & Mermaid.js (`crates/lens-app`)

**Objective:** Implement offline Mermaid.js diagram generation and asynchronous PlantUML SVG embedding.

- [ ] **Task 4.1:** Bundle `mermaid.min.js` in `crates/lens-app/assets/` and register in `wry` webview via `with_initialization_script`.
- [ ] **Task 4.2:** Implement Dioxus `use_effect` hook inside `DocumentViewer`:
  - Evaluates `document::eval("if (window.mermaid) { mermaid.run({ querySelector: '.mermaid' }); }")` upon mount or tab change.
- [ ] **Task 4.3:** Implement asynchronous PlantUML SVG fetcher:
  - Scans document for PlantUML blocks.
  - Asynchronously requests SVGs from configured server via `reqwest`.
  - Inserts rendered SVG into the DOM block.

---

### Slice 5: CLI Dispatcher & Fallback Handling (`crates/lens-cli`)

**Objective:** Connect `lens` CLI command to the desktop application, with graceful headless detection.

- [ ] **Task 5.1:** Update CLI argument parsing to support `lens [TARGET]`, `lens stop`, and `--server`.
- [ ] **Task 5.2:** Implement execution dispatch logic:
  - If `--server` is specified: launch legacy loopback HTTP service.
  - If graphical display is missing (`DISPLAY` and `WAYLAND_DISPLAY` unset on Linux): report error with hint to use `--server`.
  - Otherwise: attempt IPC forward via `IpcClient`. If no server active, launch `lens-app` process.
- [ ] **Task 5.3:** End-to-end integration tests:
  - `cli_invoked_without_active_app_then_starts_desktop_window`
  - `cli_invoked_with_active_app_then_forwards_target_and_exits_zero`

---

## Acceptance & Qualification Gate

Downstream Worker deliverables must satisfy:
1. `cargo fmt --check` and `cargo clippy --locked --all-targets --all-features -- -D warnings` clean.
2. 100% test pass across `lens-core`, `lens-ipc`, and `lens-cli`.
3. ADR-026 invariants verified (single instance, 0600 socket permissions, in-process DOM rendering).
4. No network requests emitted for Mermaid rendering (offline guarantee).
