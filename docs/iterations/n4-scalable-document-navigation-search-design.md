---
type: "Iteration Record"
title: "Iteration: N4 Scalable Document Navigation Search Design"
description: "Turns bounded navigation requirements into traceable system behavior, Rust responsibilities, and construction targets."
id: "N4"
phase: "elaboration"
status: "completed"
tags: [iteration]
---

# Iteration: N4 Scalable Document Navigation Search Design

Status: completed

Phase Intent:

- Turn ADR-008's bounded navigation decision into traceable system behavior,
  implementation responsibilities, and testable C5 construction targets.

Goal:

- Design a native-form, server-rendered identifier search that returns bounded
  pages from the immutable authorized document catalog.

Risks Addressed:

- `R-03`: query values or result links could bypass the known-document
  authorization boundary.
- `R-09`: result and request behavior could still be unbounded or unusable
  without browser scripting.

Artifacts to Start:

- None. The existing `FEAT-02` package already has canonical use-case,
  system-sequence, contract, and realization artifacts to refine.

Artifacts to Refine:

- `FEAT-02`, browse the discovered document set:
  [`docs/features/document-navigation-pane/use-cases.md`](../features/document-navigation-pane/use-cases.md) - replace browser-local filtering with submitted server-side identifier search.
- `SSD-03`, browse an authorized document catalog:
  [`docs/features/document-navigation-pane/ssd-03-browse-document-catalog.md`](../features/document-navigation-pane/ssd-03-browse-document-catalog.md) - add query and page system events.
- `OC-03`, request a document catalog:
  [`docs/features/document-navigation-pane/oc-03-request-document-catalog.md`](../features/document-navigation-pane/oc-03-request-document-catalog.md) - define bounded result, over-limit, and invalid-page postconditions.
- `RZ-02` and `DCD-02`, navigation-pane design:
  [`docs/features/document-navigation-pane/design.md`](../features/document-navigation-pane/design.md) - assign the index, controller, presentation, and Rust module responsibilities.
- `ADR-008`, paginated session-catalog search:
  [`docs/decisions/adr-008-paginated-session-catalog-search.md`](../decisions/adr-008-paginated-session-catalog-search.md) - clarify the visible result for an over-limit query.

Artifacts Consulted:

- `ADR-003`, document-root discovery:
  [`docs/decisions/adr-003-document-root-discovery.md`](../decisions/adr-003-document-root-discovery.md)
- `ADR-007`, poll known document paths:
  [`docs/decisions/adr-007-poll-known-document-paths.md`](../decisions/adr-007-poll-known-document-paths.md)
- `R-03` and `R-09`, authorization and scalable-navigation risks:
  [`docs/risk-list.md`](../risk-list.md)
- `BTE-01`, browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs)

Decisions to Record:

- Keep a document response as the sole search response. A native GET form and
  page links carry `query` and `page`; a too-long query preserves the document
  and displays a limit message rather than running an index search.
- Extract `DocumentCatalog` into `src/viewer/catalog.rs` because the new index,
  query normalization, page selection, and focused tests have an independent
  reason to change from Axum routes and document rendering.

Trace:

- Proposal 11 -> ADR-008 -> `FEAT-02` (`UC-07`, `UC-08`) -> `SSD-03` ->
  `OC-03` -> `RZ-02` / `DCD-02` -> planned C5 unit and `BTE-01` checks

Exit Criteria:

- The use cases describe explicit submission, pagination, no-JavaScript
  navigation, result limits, and rate behavior without implementation detail.
- The SSD and contract distinguish document identification from search values
  and state known, unknown, over-limit, and invalid-page outcomes.
- The realization assigns each non-trivial responsibility, maps it to Rust,
  and evaluates the module boundary required by repository guidance.
- All modified PlantUML diagrams render successfully through the configured
  PlantUML server, or an unavailable validation service is recorded.

Results:

- `UC-08` is now submitted identifier search, not a browser-local filter. It
  preserves ordinary browser operation and sends no request while the user
  types.
- `SSD-03` and `OC-03` define `request_document(document_id, query?, page?)`.
  The known document identifier is resolved separately from query and page;
  an over-limit query returns a visible limit result, while invalid pages start
  at the first page.
- `RZ-02` assigns index lookup and bounded page selection to `DocumentCatalog`,
  keeps the Axum handler thin, and treats markup as a stateless function.
  C5 will create the cohesive nested `viewer::catalog` module rather than add
  this separate responsibility to the already-large `viewer.rs`.
- PlantUML validation was skipped. The sandbox could not resolve the configured
  public PlantUML host, and the external request was not approved because it
  would transmit newly written diagram source. The diagrams remain unverified
  until that service is explicitly authorized and available.

Artifact Outcomes:

- refined: `FEAT-02`, browse the discovered document set:
  [`docs/features/document-navigation-pane/use-cases.md`](../features/document-navigation-pane/use-cases.md) - defines submitted identifier search and native pagination.
- refined: `SSD-03`, search an authorized document catalog:
  [`docs/features/document-navigation-pane/ssd-03-browse-document-catalog.md`](../features/document-navigation-pane/ssd-03-browse-document-catalog.md) - defines the query/page system operation.
- refined: `OC-03`, request a bounded document catalog page:
  [`docs/features/document-navigation-pane/oc-03-request-document-catalog.md`](../features/document-navigation-pane/oc-03-request-document-catalog.md) - specifies bounded, invalid-page, and over-limit outcomes.
- refined: `RZ-02` and `DCD-02`, navigation-pane design:
  [`docs/features/document-navigation-pane/design.md`](../features/document-navigation-pane/design.md) - assigns concrete Rust responsibilities and module boundary.
- refined: `ADR-008`, paginated session-catalog search:
  [`docs/decisions/adr-008-paginated-session-catalog-search.md`](../decisions/adr-008-paginated-session-catalog-search.md) - makes the over-limit response explicit.
