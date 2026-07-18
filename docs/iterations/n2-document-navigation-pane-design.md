# Iteration: N2 Document Navigation Pane Design

Status: completed

Phase Intent:

- Reduce the selected feature's architecture and authorization risk enough to
  start a narrow construction slice with executable acceptance checks.

Goal:

- Realize the document catalog and identifier filter using only the existing
  viewing-session document set and document route.

Risks Addressed:

- `R-03`: a sidebar or search could expose an excluded document or turn a
  browser request into a filesystem lookup.

Artifacts to Start:

- `SSD-03`, browse an authorized document catalog:
  [`docs/features/document-navigation-pane/ssd-03-browse-document-catalog.md`](../features/document-navigation-pane/ssd-03-browse-document-catalog.md) - identify system events and reuse the existing request operation.
- `OC-03`, request a document catalog:
  [`docs/features/document-navigation-pane/oc-03-request-document-catalog.md`](../features/document-navigation-pane/oc-03-request-document-catalog.md) - specify response and authorization postconditions.
- `RZ-02` and `DCD-02`, navigation-pane design:
  [`docs/features/document-navigation-pane/design.md`](../features/document-navigation-pane/design.md) - assign responsibilities and map them to Rust.
- `ADR-006`, session-derived navigation pane:
  [`docs/decisions/adr-006-document-navigation-pane.md`](../decisions/adr-006-document-navigation-pane.md) - preserve the no-new-route decision.

Artifacts to Refine:

- `FEAT-02`, document navigation-pane use cases:
  [`docs/features/document-navigation-pane/use-cases.md`](../features/document-navigation-pane/use-cases.md) - link the realized artifacts.

Artifacts Consulted:

- `FEAT-01`, Markdown-viewing use cases:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md)
- `SSD-02`, open and navigate a document root:
  [`docs/features/markdown-viewing/ssd-02-open-document-root.md`](../features/markdown-viewing/ssd-02-open-document-root.md)
- `OC-02`, open a document root:
  [`docs/features/markdown-viewing/oc-02-open-document-root.md`](../features/markdown-viewing/oc-02-open-document-root.md)
- `ADR-003`, document-root discovery:
  [`docs/decisions/adr-003-document-root-discovery.md`](../decisions/adr-003-document-root-discovery.md)

Decisions to Record:

- `ADR-006`: compose navigation from immutable session state and filter it in
  the browser, without a new endpoint.

Trace:

- Proposal 3 -> `FEAT-02` (`UC-07`, `UC-08`) -> `SSD-03` -> `OC-03` ->
  `RZ-02` / `DCD-02` -> construction unit and browser checks

Exit Criteria:

- The SSD shows only Lens boundary events and distinguishes browser-local
  filtering from a system operation.
- The contract says the catalog is exactly the authorized set and unknown paths
  do not cause filesystem work.
- The design names responsibility owners, preserves Rust ownership and Axum's
  immutable shared state, and gives construction-level test targets.
- All added or changed PlantUML diagrams render successfully through the
  configured renderer.

Results:

- `SSD-03` reuses `request_document(document_id)`; filtering stays browser
  local and does not create an API or lookup operation.
- `OC-03` makes the catalog's exact membership, active marker, and no-mutation
  guarantee explicit for both known and unknown identifiers.
- `RZ-02` assigns catalog creation to `ViewerState` as information expert and
  preserves the Axum handler as a thin controller. The Rust design needs one
  inherent method plus existing stateless functions; no trait or new module is
  warranted.
- ADR-006 records the accepted no-new-route decision. With
  `LENS_PLANTUML_SERVER` unset, the configured public renderer returned HTTP
  200 `image/svg+xml` for all three added diagrams (`SSD-03`, `RZ-02`, and
  `DCD-02`).

Artifact Outcomes:

- started: `SSD-03`, browse an authorized document catalog:
  [`docs/features/document-navigation-pane/ssd-03-browse-document-catalog.md`](../features/document-navigation-pane/ssd-03-browse-document-catalog.md) - defines the request and browser-local filtering boundary.
- started: `OC-03`, request a document catalog:
  [`docs/features/document-navigation-pane/oc-03-request-document-catalog.md`](../features/document-navigation-pane/oc-03-request-document-catalog.md) - specifies known and unknown response conditions.
- started: `RZ-02` and `DCD-02`, navigation-pane design:
  [`docs/features/document-navigation-pane/design.md`](../features/document-navigation-pane/design.md) - records GRASP decisions and Rust mapping.
- started: `ADR-006`, session-derived navigation pane:
  [`docs/decisions/adr-006-document-navigation-pane.md`](../decisions/adr-006-document-navigation-pane.md) - accepts the existing-session, browser-filter approach.
- refined: `FEAT-02`, browse the discovered document set:
  [`docs/features/document-navigation-pane/use-cases.md`](../features/document-navigation-pane/use-cases.md) - links `SSD-03`, `OC-03`, the design, and the decision.
- refined: Documentation index: [`docs/index.md`](../index.md) - links
  `ADR-006`.
