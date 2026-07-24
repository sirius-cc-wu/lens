---
type: "Iteration Record"
title: "Iteration: A1 Automatic Refresh Design"
description: "Designs automatic refresh for the displayed authorized document while preserving readable content and session scope."
id: "A1"
phase: "elaboration"
status: "completed"
tags: [iteration]
---

# Iteration: A1 Automatic Refresh Design

Status: completed

Phase Intent:

- Reduce the selected post-V1 feature's refresh, partial-save, and
  authorization risks enough to begin a narrow construction slice with
  browser-observable acceptance checks.

Goal:

- Let an author see a saved change to the currently displayed known document
  without manually reloading the browser, while preserving the fixed document
  set established at session start.

Risks Addressed:

- `R-08`: automatic refresh could rescan or expose files outside the session's
  authorized document set, replace readable content during a partial save, or
  reload the browser indefinitely after a failed read.

Artifacts to Start:

- `FEAT-03`, automatic-refresh use cases:
  [`docs/features/automatic-refresh/use-cases.md`](../features/automatic-refresh/use-cases.md) - define the author goal, fixed-set boundary, and failed-save behavior.
- `SSD-04`, refresh a changed Markdown document:
  [`docs/features/automatic-refresh/ssd-04-refresh-changed-document.md`](../features/automatic-refresh/ssd-04-refresh-changed-document.md) - identify the file-observation and browser revision events.
- `OC-04`, refresh a changed Markdown document:
  [`docs/features/automatic-refresh/oc-04-refresh-changed-document.md`](../features/automatic-refresh/oc-04-refresh-changed-document.md) - specify replacement, revision, and failed-read postconditions.
- `RZ-03` and `DCD-03`, automatic-refresh design:
  [`docs/features/automatic-refresh/design.md`](../features/automatic-refresh/design.md) - assign refresh, revision-query, and browser-reload responsibilities.
- `ADR-007`, poll known document paths:
  [`docs/decisions/adr-007-poll-known-document-paths.md`](../decisions/adr-007-poll-known-document-paths.md) - preserve the fixed-set polling decision.

Artifacts to Refine:

- Improvement proposals:
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - mark proposal 4 selected for A1 and C4.
- Risk list: [`docs/risk-list.md`](../risk-list.md) - record the selected
  refresh risk and mitigation boundary.
- Documentation index: [`docs/index.md`](../index.md) - link `FEAT-03` and
  ADR-007.
- Glossary: [`docs/glossary.md`](../glossary.md) - define a document revision.

Artifacts Consulted:

- `FEAT-01`, Markdown-viewing use cases:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md)
- `SSD-02`, open and navigate a document root:
  [`docs/features/markdown-viewing/ssd-02-open-document-root.md`](../features/markdown-viewing/ssd-02-open-document-root.md)
- `OC-02`, open a document root:
  [`docs/features/markdown-viewing/oc-02-open-document-root.md`](../features/markdown-viewing/oc-02-open-document-root.md)
- `ADR-003`, document-root discovery:
  [`docs/decisions/adr-003-document-root-discovery.md`](../decisions/adr-003-document-root-discovery.md)
- `BTE-01`, browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs)

Decisions to Record:

- `ADR-007`: poll only canonical paths that the active session has already
  authorized, retain the last successful rendering on read failure, and expose
  only a revision signal to the current browser document.

Trace:

- Proposal 4 -> `FEAT-03` (`UC-09`) -> `SSD-04` -> `OC-04` ->
  `RZ-03` / `DCD-03` -> viewer unit tests and `BTE-01`

Exit Criteria:

- The use case distinguishes a refreshed known document from new document
  discovery and specifies readable behavior during a partial save.
- The SSD identifies only black-box file-observation, revision-query, and
  document-request events.
- The contract states exact replacement and no-change postconditions.
- The design identifies Rust ownership, locking, and browser-reload boundaries
  without introducing speculative trait or module abstractions.
- Every added PlantUML block validates through the configured renderer.
- Construction has focused unit and browser acceptance targets.

Results:

- `FEAT-03` defines `UC-09` for the currently displayed known document. It
  explicitly keeps session membership fixed and retains the last readable
  representation when a save is temporarily unreadable.
- `SSD-04` and `OC-04` distinguish the watcher-owned
  `observe_document_change` operation from the browser's revision query and
  existing document request.
- `RZ-03` assigns immutable authorization to `ViewerState`, last-good content
  to `ViewerDocument`, periodic coordination to a free async function, and
  reload choice to browser script. Its Rust mapping uses `Arc` and a narrow
  `RwLock` without a watcher trait or a new source module.
- ADR-007 accepts bounded polling of existing canonical paths and a
  revision-only browser route. The public PlantUML renderer returned HTTP 200
  `image/svg+xml` for the SSD, realization, and class diagram.
- C4 will first add a browser acceptance check that changes the current fixture
  without calling `page.reload()`, demonstrate its absence against the current
  viewer, then add the watcher, revision route, and focused unit coverage.

Artifact Outcomes:

- started: `FEAT-03`, refresh changed Markdown documents:
  [`docs/features/automatic-refresh/use-cases.md`](../features/automatic-refresh/use-cases.md) - defines `UC-09` and its safety boundaries.
- started: `SSD-04`, refresh a changed Markdown document:
  [`docs/features/automatic-refresh/ssd-04-refresh-changed-document.md`](../features/automatic-refresh/ssd-04-refresh-changed-document.md) - establishes system events.
- started: `OC-04`, refresh a changed Markdown document:
  [`docs/features/automatic-refresh/oc-04-refresh-changed-document.md`](../features/automatic-refresh/oc-04-refresh-changed-document.md) - specifies successful, unchanged, and failed read outcomes.
- started: `RZ-03` and `DCD-03`, automatic-refresh design:
  [`docs/features/automatic-refresh/design.md`](../features/automatic-refresh/design.md) - records responsibilities, Rust mapping, and test targets.
- started: `ADR-007`, poll known document paths:
  [`docs/decisions/adr-007-poll-known-document-paths.md`](../decisions/adr-007-poll-known-document-paths.md) - accepts the polling and revision decision.
- refined: Improvement proposals:
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - proposal 4 is selected for A1 and C4.
- refined: Risk list: [`docs/risk-list.md`](../risk-list.md) - adds `R-08`.
- refined: Documentation index: [`docs/index.md`](../index.md) - links
  `FEAT-03` and ADR-007.
- refined: Glossary: [`docs/glossary.md`](../glossary.md) - defines document
  revision.
