---
type: "Iteration Record"
title: "Iteration: P5 Diagram Failure Controls"
description: "Implements renderer status, retry, and session-disable controls for diagram failures."
id: "P5"
phase: "construction"
status: "completed"
tags: [iteration]
---

# Iteration: P5 Diagram Failure Controls

Status: completed

Phase Intent:

- Construct a narrow, browser-observable recovery path for renderer failures
  without widening renderer or filesystem authority.

Goal:

- Let a user see renderer status, retry a failed diagram, or stop diagram
  rendering for the active viewing session.

Risks Addressed:

- `R-01`: renderer availability failures can leave diagram users without a
  clear recovery or fallback action.
- `R-03`: a browser-facing control could accidentally accept arbitrary renderer
  configuration or paths.

Artifacts to Start:

- `ADR-011`, diagram failure controls:
  [`docs/decisions/adr-011-diagram-failure-controls.md`](../decisions/adr-011-diagram-failure-controls.md) - define the fixed retry and disable boundaries.

Artifacts to Refine:

- `FEAT-01`, Markdown viewing use cases:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md) - specify status, retry, and session disable outcomes.
- Supplementary specification, proposal list, risk list, glossary, and README:
  [`docs/supplementary-specification.md`](../supplementary-specification.md), [`docs/improvement-proposals.md`](../improvement-proposals.md), [`docs/risk-list.md`](../risk-list.md), [`docs/glossary.md`](../glossary.md), and [`README.md`](../../README.md) - state the quality boundary and user controls.
- `BTE-01`, browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - prove retry and session disable behavior.

Artifacts Consulted:

- `ADR-009`, selectable renderer:
  [`docs/decisions/adr-009-selectable-plantuml-rendering.md`](../decisions/adr-009-selectable-plantuml-rendering.md)
- Existing renderer failure behavior:
  [`docs/iterations/c2-browser-guidance-and-renderer-failure.md`](c2-browser-guidance-and-renderer-failure.md)

Decisions to Record:

- `ADR-011`: retain fixed diagram identifiers for retries and permit only one
  in-memory session-disable POST mutation.

Trace:

- Proposal 5 -> `FEAT-01` (`UC-01`) -> `ADR-011` -> viewer-route tests and
  `BTE-01` retry and disable scenarios

Exit Criteria:

- Every rendered document identifies the active renderer.
- A failed diagram exposes a retry action that succeeds against a recovered
  controlled renderer.
- Session disable stops renderer calls, leaves source readable, and does not
  accept renderer configuration from the browser.
- Formatter, tests, browser evidence, and Clippy pass.

Results:

- `ViewerState` now owns an atomic session-disable flag alongside its fixed
  renderer. The fixed `/renderer/disable` POST route sets only that flag; the
  diagram route returns a disabled result before it can invoke a renderer.
- The client retry button reloads the existing image route with a cache buster.
  The disable control removes image sources, opens each source fallback, and
  updates the status without a page reload.
- Viewer tests verify the public status and disabled diagram route. Browser
  tests verify a 503-to-200 retry sequence and that session disable prevents a
  further controlled-renderer request.
- No PlantUML design block changed in this iteration, so PlantUML-server
  validation was not required.

Artifact Outcomes:

- started: `ADR-011`, diagram failure controls:
  [`docs/decisions/adr-011-diagram-failure-controls.md`](../decisions/adr-011-diagram-failure-controls.md) - records the retry and session-state decision.
- refined: `FEAT-01`, user documentation, quality constraints, risk list, and
  `BTE-01` - record the implemented recovery controls and executable evidence.
