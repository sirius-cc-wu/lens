---
type: "Iteration Record"
title: "Iteration: E4 Server-Only PlantUML Rendering"
description: "Makes the removal of renderer modes construction-ready by fixing server configuration, failure, compatibility, and Rust ownership boundaries."
id: "E4"
phase: "elaboration"
status: "completed"
tags: [iteration]
---

# Iteration: E4 Server-Only PlantUML Rendering

Status: completed

Phase Intent:

- Resolve the architectural and compatibility risks in removing renderer
  selection before construction changes the public CLI, Rust API, and browser
  behavior.

Goal:

- Define one traceable server-rendering path whose destination is fixed at
  session startup, whose failure never falls back to another server, and whose
  implementation handoff removes rather than renames renderer variation.

Risks Addressed:

- `R-01`: a server-rendering path can disclose source or become unavailable;
  the design must preserve a controlled destination, bounded failure, and
  visible source.
- `R-10`: removing renderer modes breaks CLI and Rust API consumers and removes
  local-command and no-rendering workflows.

Artifacts to Start:

- `ADR-017`, session-fixed PlantUML server:
  [`docs/decisions/adr-017-session-plantuml-server.md`](../decisions/adr-017-session-plantuml-server.md) -
  record the selected path and supersession.
- `OC-05`, request a diagram:
  [`docs/features/markdown-viewing/oc-05-request-diagram.md`](../features/markdown-viewing/oc-05-request-diagram.md) -
  make fixed-destination, response-bound, and no-fallback outcomes testable.
- `RZ-05` and `DCD-04`, server rendering design:
  [`docs/features/markdown-viewing/server-rendering-design.md`](../features/markdown-viewing/server-rendering-design.md) -
  map the requirements to idiomatic Rust collaboration and ownership.

Artifacts to Refine:

- `PROP-REMOVE-RENDERER`:
  [`docs/proposals/remove-renderer.md`](../proposals/remove-renderer.md) -
  resolve normalization and library-release acceptance details and add trace.
- `FEAT-01`, `SSD-01`, and `OC-01`:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md),
  [`docs/features/markdown-viewing/ssd-01-open-markdown-target.md`](../features/markdown-viewing/ssd-01-open-markdown-target.md),
  and
  [`docs/features/markdown-viewing/oc-01-open-markdown-target.md`](../features/markdown-viewing/oc-01-open-markdown-target.md) -
  replace renderer modes with black-box server behavior and link the new
  diagram-request contract.
- Supplementary specification, risk list, glossary, proposal history, and
  documentation index:
  [`docs/supplementary-specification.md`](../supplementary-specification.md),
  [`docs/risk-list.md`](../risk-list.md),
  [`docs/glossary.md`](../glossary.md),
  [`docs/improvement-proposals.md`](../improvement-proposals.md), and
  [`docs/index.md`](../index.md) - align current requirements and navigation.
- `ADR-005`, `ADR-009`, and `ADR-011`:
  [`docs/decisions/`](../decisions/) - preserve their historical bodies while
  recording full or partial supersession.

Artifacts Consulted:

- `P1`, local PlantUML rendering:
  [`docs/iterations/p1-local-plantuml-rendering.md`](p1-local-plantuml-rendering.md)
- `P5`, diagram failure controls:
  [`docs/iterations/p5-diagram-failure-controls.md`](p5-diagram-failure-controls.md)
- Current renderer implementation:
  [`src/plantuml.rs`](../../src/plantuml.rs),
  [`src/viewer.rs`](../../src/viewer.rs),
  [`src/markdown.rs`](../../src/markdown.rs), and
  [`src/main.rs`](../../src/main.rs)

Decisions to Record:

- `ADR-017`: read and normalize `LENS_PLANTUML_SERVER` once per session, use
  the public server only when that normalized value is empty, and never accept
  or expose a replacement through browser routes.
- Remove `RendererMode` and `DiagramRenderer`; represent the single server path
  as a server string owned by `ViewerState` and cohesive `plantuml` functions.
- Retain per-diagram retry and failure behavior, but remove the session-disable
  state and browser mutation because it cannot prevent initial requests.

Trace:

- `PROP-REMOVE-RENDERER` -> `FEAT-01` (`UC-01`, `UC-10`) -> `SSD-01` ->
  `OC-05` -> `ADR-017` -> `RZ-05` -> `DCD-04` -> construction tests

Exit Criteria:

- Default, configured, invalid, unavailable, and removed-argument behavior is
  unambiguous at the system boundary.
- The configured-server failure contract explicitly forbids default-server
  fallback.
- The Rust design identifies the owner and lifecycle of server configuration
  without introducing a replacement variation abstraction.
- CLI, public API, browser-route, documentation, and release migration work is
  enumerated for construction.
- Every changed PlantUML block validates through the configured PlantUML
  server.

Results:

- Accepted ADR-017 and linked the proposal through use cases, SSD, operation
  contract, realization, Rust type target, risks, and construction checks.
- Chose a session-owned server string and module functions instead of an enum,
  trait, or one-variant wrapper. The browser still requests only authorized
  diagram identifiers.
- Preserved historical iteration bodies and narrowed supersession to the
  affected parts of ADR-005 and ADR-011.
- Construction remains pending. User-facing README, release-readiness evidence,
  and source-code behavior must continue to describe the shipped renderer modes
  until the construction iteration changes and verifies them together.
- The changed `SSD-01` block and the new `RZ-05` and `DCD-04` PlantUML blocks
  were validated through the configured PlantUML server.

Artifact Outcomes:

- started: `ADR-017`, `OC-05`, `RZ-05`, and `DCD-04` - establish the accepted
  decision, operation guarantees, collaboration, and Rust target at their
  canonical paths.
- refined: `PROP-REMOVE-RENDERER`, `FEAT-01`, and `SSD-01` - make the
  server-only behavior and compatibility boundary construction-ready.
- refined: affected durable decisions and cross-cutting requirements - record
  current supersession and target constraints without rewriting iteration
  history.
- deferred: source, executable tests, README, release notes, and
  release-readiness evidence - update together during construction.
