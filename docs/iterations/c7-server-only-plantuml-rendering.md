---
type: "Iteration Record"
title: "Iteration: C7 Server-Only PlantUML Rendering"
description: "Constructs the single session-fixed PlantUML server path and removes renderer modes."
id: "C7"
phase: "construction"
status: "completed"
tags: [iteration]
---

# Iteration: C7 Server-Only PlantUML Rendering

Status: completed

Phase Intent:

- Implement the server-only rendering decision from E4 while keeping failures
  bounded, visible, and fixed to the server selected at session startup.

Goal:

- Remove renderer-mode variation from the command line, exported Rust API,
  viewer state, browser controls, and runtime dependencies as one verified
  compatibility change.

Risks Addressed:

- `R-01`: keep PlantUML requests on one explicit destination, with no fallback
  after a configured-server failure.
- `R-10`: make the command-line and Rust API break explicit, executable, and
  documented for users of the removed local-command and disabled modes.

Artifacts to Start:

- Pending release notes:
  [`docs/release-notes.md`](../release-notes.md) - publish the breaking CLI,
  Rust API, and former-local-user migration before the next release.

Artifacts to Refine:

- `PROP-REMOVE-RENDERER`, `FEAT-01`, `RZ-05`, and `DCD-04`:
  [`docs/proposals/remove-renderer.md`](../proposals/remove-renderer.md),
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md),
  and
  [`docs/features/markdown-viewing/server-rendering-design.md`](../features/markdown-viewing/server-rendering-design.md)
- Runtime implementation and verification:
  [`src/main.rs`](../../src/main.rs),
  [`src/lib.rs`](../../src/lib.rs),
  [`src/plantuml.rs`](../../src/plantuml.rs),
  [`src/markdown.rs`](../../src/markdown.rs),
  [`src/viewer.rs`](../../src/viewer.rs),
  [`tests/cli.rs`](../../tests/cli.rs), and
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs)
- User and release guidance:
  [`README.md`](../../README.md),
  [`docs/release-notes.md`](../release-notes.md), and
  [`docs/release-readiness.md`](../release-readiness.md)

Artifacts Consulted:

- `E4`, server-only PlantUML rendering:
  [`docs/iterations/e4-server-only-plantuml-rendering.md`](e4-server-only-plantuml-rendering.md)
- `ADR-017`, session-fixed PlantUML server:
  [`docs/decisions/adr-017-session-plantuml-server.md`](../decisions/adr-017-session-plantuml-server.md)
- `OC-05`, request a diagram:
  [`docs/features/markdown-viewing/oc-05-request-diagram.md`](../features/markdown-viewing/oc-05-request-diagram.md)

Decisions to Record:

- Read and normalize `LENS_PLANTUML_SERVER` once at startup, using the public
  server only when the normalized value is empty.
- Store the selected base URL directly in viewing-session state. Do not replace
  the removed modes with a one-variant enum, trait, or wrapper.
- Preserve the bounded server request, visible source, and per-diagram retry.
  Remove local process launching, session-disable state, and the browser route
  that mutated that state.
- Remove Tokio's process and asynchronous I/O feature flags because runtime
  command execution is no longer part of Lens.

Trace:

- `PROP-REMOVE-RENDERER` -> `FEAT-01` (`UC-01`, `UC-10`) -> `SSD-01` ->
  `OC-05` -> `ADR-017` -> `RZ-05` -> `DCD-04` -> CLI, Rust, and browser tests

Test-Driven Evidence:

- Oracle: the proposal acceptance criteria and `OC-05` postconditions.
- CLI red: help still advertised renderer selection and `--renderer` was still
  accepted before the option and public API were removed.
- Runtime red: rendered pages still exposed disable state, the page still
  described renderer selection, and `POST /renderer/disable` returned success.
- Browser red: the obsolete disable control was visible and its mutation route
  returned success.
- Green: focused CLI, Rust, and browser checks passed after the smallest
  production change; complete Rust and browser suites then passed.

Exit Criteria:

- Help and the exported Rust API expose no renderer-mode selection.
- Default and configured servers are normalized once per session, and a
  configured-server failure contacts no fallback.
- Markdown and standalone PlantUML documents always use the diagram image
  route, while failed requests retain source and retry controls.
- The local renderer process, disable control, and disable route are absent.
- User migration, release readiness, risk, and decision documentation describe
  the shipped behavior.
- Formatting, lint, Rust tests, browser tests, and changed PlantUML blocks
  validate.

Results:

- Replaced renderer-mode branching with a session-owned PlantUML server string
  and cohesive server request functions.
- Removed local command execution and its Tokio feature dependencies.
- Added executable checks for CLI rejection, the one-argument public entry
  point, server normalization, no fallback, page status, and the removed
  browser route.
- Published migration guidance for former public, local, and disabled-mode
  users.
- `cargo fmt --check`, all 55 library tests, all 5 CLI tests, Clippy with
  warnings denied, and all 16 browser scenarios passed. Both `RZ-05` and
  `DCD-04`, plus the refined `DCD-03`, validated through the configured/default
  PlantUML server with HTTP 200 and no diagram-error response.

Artifact Outcomes:

- started: pending release notes - record the breaking migration at their
  canonical release path.
- implemented: `PROP-REMOVE-RENDERER`, `FEAT-01`, `RZ-05`, `DCD-04`, and
  `ADR-017` at their canonical paths.
- refined: README, pending release notes, release readiness, risks, durable
  decision scope, and documentation navigation.
- verified: focused red-to-green checks followed by complete Rust and browser
  regression suites, formatting, lint, and configured-server PlantUML
  validation.
