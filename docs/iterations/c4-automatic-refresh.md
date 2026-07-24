---
type: "Iteration Record"
title: "Iteration: C4 Automatic Refresh"
description: "Implements and verifies safe refresh of the displayed known Markdown document after a successful save."
id: "C4"
phase: "construction"
status: "completed"
tags: [iteration]
---

# Iteration: C4 Automatic Refresh

Status: completed

Phase Intent:

- Construct and verify the automatic-refresh slice whose requirements and
  design stabilized in A1.

Goal:

- Refresh the browser's currently displayed known Markdown document after a
  successful save without changing the session's document set or replacing
  readable content during a failed read.

Risks Addressed:

- `R-08`: automatic refresh could broaden document access, expose an
  undiscovered path, or make a partial save erase a readable page.

Artifacts to Start:

- None. A1 established the canonical feature, interaction, contract,
  realization, and decision artifacts.

Artifacts to Refine:

- `FEAT-03`, automatic-refresh use cases:
  [`docs/features/automatic-refresh/use-cases.md`](../features/automatic-refresh/use-cases.md) - mark `UC-09` implemented and record executable evidence.
- `RZ-03` and `DCD-03`, automatic-refresh design:
  [`docs/features/automatic-refresh/design.md`](../features/automatic-refresh/design.md) - record the concrete Rust implementation and lock boundary.
- `BTE-01`, browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - save the displayed fixture and verify automatic browser refresh.
- Improvement proposals:
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - mark proposal 4 implemented.
- Risk list: [`docs/risk-list.md`](../risk-list.md) - record refresh evidence
  and residual scale risk.
- V1 release readiness:
  [`docs/release-readiness.md`](../release-readiness.md) - add automatic
  refresh to the browser-suite outcomes.

Artifacts Consulted:

- `SSD-04`, refresh a changed Markdown document:
  [`docs/features/automatic-refresh/ssd-04-refresh-changed-document.md`](../features/automatic-refresh/ssd-04-refresh-changed-document.md)
- `OC-04`, refresh a changed Markdown document:
  [`docs/features/automatic-refresh/oc-04-refresh-changed-document.md`](../features/automatic-refresh/oc-04-refresh-changed-document.md)
- `ADR-007`, poll known document paths:
  [`docs/decisions/adr-007-poll-known-document-paths.md`](../decisions/adr-007-poll-known-document-paths.md)
- `BTE-01`, existing browser harness:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs)

Decisions to Record:

- None. C4 implements ADR-007's bounded, fixed-path polling and revision
  decision without adding another abstraction or discovery route.

Trace:

- Proposal 4 -> `FEAT-03` (`UC-09`) -> `SSD-04` -> `OC-04` ->
  `RZ-03` / `DCD-03` -> viewer unit tests and `BTE-01`

Exit Criteria:

- Saving the displayed known document changes its rendered browser content
  without an explicit browser reload.
- A failed document read leaves the last rendered content and revision intact.
- The watcher reads only session-owned canonical paths; an unknown revision
  request returns the existing 404 guidance response.
- Formatter, Rust tests, Clippy, and the complete browser suite pass.

Results:

- `ViewerState` now keeps immutable authorization state separate from an
  `RwLock` of last-successful document representations. The session starts one
  500-millisecond polling task that reads only each stored canonical path.
- A changed readable source is rendered under the existing link and PlantUML
  rules, then atomically replaces that document's representation and advances
  its revision. A missing or unreadable source is ignored until a later poll,
  preserving the browser's readable content.
- Successful pages expose their identifier and revision to local script. The
  script polls the `no-store` revision route every 500 milliseconds and reloads
  only when that current document's revision changes. The handler resolves the
  identifier through the existing map; an unknown revision request returns 404.
- The new browser test first failed against the former viewer after waiting
  five seconds for the saved heading. After implementation, it passed without
  `page.reload()`. The complete browser suite passed all six tests; 34 library
  tests, three CLI tests, and Clippy with warnings denied also passed.
- Residual risk: each active session reads every known document twice per
  second. The bounded polling interval is appropriate for ordinary
  documentation roots, but a scalability measurement should precede support
  for unusually large document sets.

Artifact Outcomes:

- refined: `FEAT-03`, refresh changed Markdown documents:
  [`docs/features/automatic-refresh/use-cases.md`](../features/automatic-refresh/use-cases.md) - marked implemented with browser and unit evidence.
- refined: `RZ-03` and `DCD-03`, automatic-refresh design:
  [`docs/features/automatic-refresh/design.md`](../features/automatic-refresh/design.md) - records the actual `RwLock`, revision route, and interval behavior.
- refined: `BTE-01`, browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - verifies revision-only output and automatic refresh after a saved change.
- refined: Improvement proposals:
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - proposal 4 is implemented in C4.
- refined: Risk list: [`docs/risk-list.md`](../risk-list.md) - records
  fixed-path, failed-read, and browser-refresh evidence.
- refined: V1 release readiness:
  [`docs/release-readiness.md`](../release-readiness.md) - includes automatic
  refresh in the browser-suite outcome.
