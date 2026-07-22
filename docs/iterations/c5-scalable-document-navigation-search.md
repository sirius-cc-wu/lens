---
type: "Iteration Record"
title: "Iteration: C5 Scalable Document Navigation Search"
description: "Implements bounded server-rendered identifier search over the authorized document catalog."
id: "C5"
phase: "construction"
status: "completed"
tags: [iteration]
---

# Iteration: C5 Scalable Document Navigation Search

Status: completed

Phase Intent:

- Implement the N4 design as a narrow, executable vertical slice and preserve
  the authorized-document boundary under paginated identifier search.

Goal:

- Replace the complete client-side navigation catalog with a capped,
  server-rendered identifier search that remains usable without JavaScript.

Risks Addressed:

- `R-03`: a search request or result link could select a filesystem path rather
  than a document already authorized for the viewing session.
- `R-09`: a complete catalog, per-keystroke request pattern, or unbounded page
  could make large documentation trees impractical.

Artifacts to Start:

- `DocumentCatalog`, session identifier index:
  [`src/viewer/catalog.rs`](../../src/viewer/catalog.rs) - own index lookup,
  query parsing, page selection, and focused unit tests separately from the
  loopback viewer module.

Artifacts to Refine:

- `FEAT-02`, browse the discovered document set:
  [`docs/features/document-navigation-pane/use-cases.md`](../features/document-navigation-pane/use-cases.md) - record C5 implementation status.
- `RZ-02` and `DCD-02`, navigation-pane design:
  [`docs/features/document-navigation-pane/design.md`](../features/document-navigation-pane/design.md) - record the concrete Rust construction result.
- `BTE-01`, browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - submit a search and page through results without JavaScript.
- Improvement proposals:
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - mark proposal 11 implemented.
- Risk list: [`docs/risk-list.md`](../risk-list.md) - record C5's bounded-search evidence.
- V1 release readiness:
  [`docs/release-readiness.md`](../release-readiness.md) - include the browser-search outcome.

Artifacts Consulted:

- `ADR-008`, paginated session-catalog search:
  [`docs/decisions/adr-008-paginated-session-catalog-search.md`](../decisions/adr-008-paginated-session-catalog-search.md)
- `SSD-03`, search an authorized document catalog:
  [`docs/features/document-navigation-pane/ssd-03-browse-document-catalog.md`](../features/document-navigation-pane/ssd-03-browse-document-catalog.md)
- `OC-03`, request a bounded document catalog page:
  [`docs/features/document-navigation-pane/oc-03-request-document-catalog.md`](../features/document-navigation-pane/oc-03-request-document-catalog.md)
- `R-03` and `R-09`, authorization and scalable-navigation risks:
  [`docs/risk-list.md`](../risk-list.md)

Decisions to Record:

- None. C5 implements ADR-008 and the N4 realization without adding a new
  authorization route, search service, trait, or mutable session state.

Trace:

- Proposal 11 -> ADR-008 -> `FEAT-02` (`UC-07`, `UC-08`) -> `SSD-03` ->
  `OC-03` -> `RZ-02` / `DCD-02` -> catalog unit tests and `BTE-01`

Exit Criteria:

- A response contains at most 50 authorized document identifiers, and a
  matching next page is reachable through an ordinary link.
- A submitted, case-insensitive identifier search works without JavaScript;
  typing alone sends no request.
- A 257-byte query reports the limit without searching, and malformed or
  out-of-range pages return the first matching page.
- Unknown document identifiers retain the existing guidance response and no
  search operation scans the filesystem or reads document contents.
- Formatter, Rust tests, Clippy, and the complete browser suite pass.

Results:

- `DocumentCatalog` owns one immutable `BTreeMap` created from the session's
  existing authorized identifiers. It resolves known document IDs and scans
  only that in-memory index to create an owned 50-item page.
- Document routes parse `query` and `page` separately from their known document
  identifier. The rendered navigation uses a native GET form, accessible result
  status, and previous/next links; the obsolete JavaScript input filter was
  removed while diagram handling and refresh polling stayed unchanged.
- Discrimination evidence: before implementation,
  `cargo test --locked more_than_result_limit_then_shows_first_page_and_next_link`
  failed with 51 navigation items where the oracle required 50. The same test
  passed after the bounded page implementation.
- Browser evidence: `BTE-01` creates 51 matching documents, disables
  JavaScript, submits `reference`, verifies exactly 50 first-page links, then
  follows the native next link to `guides/reference-050.md`.
- Final validation passed: `cargo fmt --check`, 40 library tests plus three CLI
  tests under `cargo test --locked`,
  `cargo clippy --locked --all-targets --all-features -- -D warnings`, and all
  seven `npm run test:browser` scenarios. The residual risk is measurement, not
  authorization: each submitted search scans the immutable identifier index, so
  an unusually large supported document-count limit still needs performance
  evidence.
- The C5 implementation corrected the canonical realization's method name to
  `navigation_pane`. Its PlantUML blocks remain unverified because N4's required
  public-service validation could not be approved after the sandbox could not
  resolve that host; the unavailability is recorded in the N4 iteration record.

Artifact Outcomes:

- started: `DocumentCatalog`, session identifier index:
  [`src/viewer/catalog.rs`](../../src/viewer/catalog.rs) - contains search parsing, bounded page selection, and five focused tests.
- refined: `FEAT-02`, browse the discovered document set:
  [`docs/features/document-navigation-pane/use-cases.md`](../features/document-navigation-pane/use-cases.md) - marked scalable search implemented in C5.
- refined: `RZ-02` and `DCD-02`, navigation-pane design:
  [`docs/features/document-navigation-pane/design.md`](../features/document-navigation-pane/design.md) - records the exact catalog, route, and browser-script construction.
- refined: `BTE-01`, browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - verifies submitted search and no-JavaScript pagination.
- refined: Improvement proposals:
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - proposal 11 is implemented.
- refined: Risk list: [`docs/risk-list.md`](../risk-list.md) - records result and browser evidence for `R-09`.
- refined: V1 release readiness:
  [`docs/release-readiness.md`](../release-readiness.md) - names submitted and no-JavaScript paginated search in browser evidence.
