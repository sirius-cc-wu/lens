---
type: "Iteration Record"
title: "Iteration: C11 Retained Viewer Session"
description: "Extracts browser-ready viewer startup into an owned session while preserving the public foreground server."
id: "C11"
phase: "construction"
status: "completed"
tags: [iteration, viewer, lifecycle]
---

# Iteration: C11 Retained Viewer Session

Status: completed

Phase Intent:

- Establish the independently retained browser-facing unit that the background
  service can own before introducing any command protocol or process behavior.

Goal:

- Introduce `ViewerSession` and `viewer::start_session` so a caller can obtain
  a ready loopback URL and retain the session tasks, while public
  `lens::serve(MarkdownTarget)` keeps its foreground launch and Ctrl-C contract.

Risks Addressed:

- Starting the HTTP server in a task could acknowledge a URL before it is
  bound, detach watcher lifetime from the session, or change the existing
  foreground shutdown and manual-URL behavior.

Artifact Budget:

- create: `docs/iterations/c11-retained-viewer-session.md` - preserve the
  lifecycle objective, test-first evidence, and iteration-to-commit mapping.
- keep with implementation: session ownership and lifecycle oracle - the
  `ViewerSession` code and focused HTTP test are the authoritative owners.
- omit: separate lifecycle design - `DCD-04` already owns this responsibility
  and no new cross-cutting decision is expected.

Artifacts to Start:

- This C11 iteration record - capture the session extraction and evidence.

Artifacts to Refine:

- Viewer composition and lifecycle:
  [`src/viewer/mod.rs`](../../src/viewer/mod.rs)
- `BACKGROUND-VIEWER-DESIGN` only if implementation changes the represented
  task ownership or foreground compatibility:
  [`docs/features/background-viewer-service/design.md`](../features/background-viewer-service/design.md)

Artifacts Consulted:

- `OC-07`, ready URL and isolated-session postconditions:
  [`docs/features/background-viewer-service/oc-07-request-target-view.md`](../features/background-viewer-service/oc-07-request-target-view.md)
- `ADR-002`, loopback viewer scope:
  [`docs/decisions/adr-002-loopback-viewer-scope.md`](../decisions/adr-002-loopback-viewer-scope.md)

Decisions to Record:

- Keep server and watcher task handles plus graceful server shutdown inside
  `ViewerSession`; dropping the handle ends its tasks.

Trace:

- `UC-11` -> `OC-07` ready URL -> `DCD-04` `ViewerSession` -> C11 lifecycle test

Test-Driven Evidence:

- Oracle: OC-07 requires a bound loopback view before acknowledgment, while
  DCD-04 assigns the listener URL, watcher, and HTTP task lifetime to one owned
  `ViewerSession`.
- Slice size: one lifecycle boundary and its foreground compatibility adapter;
  no service protocol, endpoint, or process behavior enters this iteration.
- Discrimination: with the stable `start_session` seam returning the explicit
  placeholder error, `cargo test --locked
  started_session_then_serves_selected_document_while_handle_is_retained
  -- --nocapture` failed at session startup with `viewer session startup is not
  implemented`. The test therefore detected absent lifecycle behavior before
  attempting HTTP I/O.

Exit Criteria:

- A started session serves its selected document while its handle is retained.
- Dropping a session ends both watcher and HTTP tasks; partial startup does not
  publish a URL.
- Public `lens::serve` opens the same browser URL, prints the same guidance, and
  waits for Ctrl-C with graceful HTTP shutdown.
- Formatting, locked Rust tests, Clippy, compiled-browser tests, and diff
  checks pass.

Results:

- Added `ViewerSession` with an already-bound view URL, selected document path,
  HTTP server task, watcher task, and graceful-shutdown sender.
- Added `start_session`, which completes every fallible bind and server setup
  step before returning the URL, then ties both spawned tasks to the session
  handle. Dropping the handle aborts both tasks.
- Reimplemented public `lens::serve` through the new session without moving
  browser launch, messages, manual URL guidance, or the Ctrl-C wait out of the
  foreground path.
- Focused checks passed for a retained session serving its selected document
  and a dropped session releasing its loopback listener.
- Full verification passed: `cargo fmt --check`, `cargo test --locked` (79
  library and five CLI tests), `cargo clippy --locked --all-targets
  --all-features -- -D warnings`, `npm run test:browser` (26 scenarios), and
  `git diff --check`.
- No PlantUML block changed, so diagram validation was not applicable.

Artifact Outcomes:

- started and completed: C11 retained viewer session - records the lifecycle
  slice, discrimination evidence, and passing verification.
- refined: viewer composition and lifecycle - `ViewerSession` owns task
  lifetime and `serve` is the foreground compatibility adapter.
- consulted without change: `BACKGROUND-VIEWER-DESIGN` - the implemented
  ownership and collaboration match `DCD-04` and require no design correction.
