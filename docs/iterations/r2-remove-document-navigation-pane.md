---
type: "Iteration Record"
title: "Iteration: R2 Remove the Document Navigation Pane"
description: "Removes catalog search and pane presentation while preserving authorized routes, authored links, direct targets, and refresh behavior."
id: "R2"
phase: "construction"
status: "completed"
tags: [iteration]
---

# Iteration: R2 Remove the Document Navigation Pane

Status: completed

Phase Intent:

- Implement ADR-020 as one end-to-end behavior slice, using executable checks
  to protect the fixed authorization and viewing-session behavior that remains.

Goal:

- Make every successful document response a focused single-column page with no
  catalog, search, pagination, pane control, or navigation browser state.

Risks Addressed:

- `R-03`: replacing `DocumentCatalog` could weaken known-route authorization
  or authored-link rewriting.
- Compatibility: removing query parsing or page composition could alter direct
  targets, repository and target scope, PlantUML routes, or automatic refresh.
- Layout: deleting sidebar rules could regress narrow or wide document
  readability.

Artifacts to Start:

- `KnownDocuments`, fixed route-authorization lookup:
  [`src/viewer/known_documents.rs`](../../src/viewer/known_documents.rs) - map only discovered identifiers to their immutable document indices.

Artifacts to Refine:

- Viewer session, routes, and page composition:
  [`src/viewer/`](../../src/viewer/) - remove catalog request and presentation responsibilities while retaining route lookup.
- Embedded browser assets:
  [`src/viewer/assets/`](../../src/viewer/assets/) - remove navigation-only
  script, storage, grid, and component rules.
- Browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - replace
  catalog-only scenarios with authored-link, focused-layout, inert-query,
  direct-PlantUML, history, guidance, and refresh evidence.
- Rust dependency manifest: [`Cargo.toml`](../../Cargo.toml) - remove the
  direct query-encoding dependency used only by the catalog.
- `ADR-020`:
  [`docs/decisions/adr-020-focused-document-review.md`](../decisions/adr-020-focused-document-review.md) - link construction evidence after verification.

Artifacts Consulted:

- Navigation-pane removal proposal:
  [`docs/proposals/remove-document-navigation-pane.md`](../proposals/remove-document-navigation-pane.md)
- `FEAT-01`, safe authored Markdown navigation:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md)
- `FEAT-03`, automatic refresh:
  [`docs/features/automatic-refresh/use-cases.md`](../features/automatic-refresh/use-cases.md)
- `ADR-003`, document-root authorization:
  [`docs/decisions/adr-003-document-root-discovery.md`](../decisions/adr-003-document-root-discovery.md)
- `ADR-019`, repository and target scope:
  [`docs/decisions/adr-019-repository-scoped-target-sessions.md`](../decisions/adr-019-repository-scoped-target-sessions.md)

Decisions to Record:

- Replace `viewer::catalog` with `viewer::known_documents`; keep the fixed
  identifier set used by Markdown rendering separate from the route map because
  each has one explicit consumer and neither provides search.
- Let Axum ignore query strings naturally by removing raw-query extraction from
  document handlers.
- Preserve the existing document typography and component styling; remove only
  navigation-specific layout and interaction rules.

Trace:

- ADR-020 -> focused page and known-document routes -> Rust page/route/state
  checks -> `BTE-01` browser behavior and compatibility checks

Exit Criteria:

- Successful pages expose no navigation landmark, catalog form/results,
  pagination, current marker, pane toggle, collapsed attribute, or navigation
  `sessionStorage` value.
- Known routes ignore `query` and `page`; unknown routes remain Lens-owned and
  do not expose source.
- Authored Markdown links, browser history, direct Markdown and PlantUML
  targets, repository and target scope, automatic refresh, and per-diagram
  behavior pass focused and regression checks.
- Narrow and wide browser checks show one document-focused column without page
  overflow.
- The direct `form_urlencoded` dependency and catalog-only module/tests are
  absent.

Results:

- The initial focused page test run discriminated the missing behavior:
  `cargo test --locked viewer::page::tests::` reported five passing historical
  checks and failed the two new expectations because pane controls were still
  present and the guidance title still named document navigation. The earlier
  attempt to pass two Cargo test filters was a command-usage error and was not
  counted as behavior evidence.
- `viewer::known_documents` now owns only the immutable
  identifier-to-document-index map used by document and revision routes.
  Markdown rendering retains the separately named fixed identifier set used to
  authorize authored links. Browser query strings are no longer parsed.
- Successful pages now contain only the document reading column. Catalog
  markup, search, pagination, active-result state, pane controls, collapsed
  attributes, navigation storage, sidebar CSS, navigation JavaScript, and the
  direct `form_urlencoded` manifest dependency were removed.
- The not-found page now says “Document unavailable” while preserving the
  Lens-owned response and return link. Unknown, hidden, traversal, symbolic
  target, and out-of-root behavior remains protected by route, target, and
  browser regression checks.
- `cargo fmt --check`, `cargo test --locked` (61 library tests and 5 CLI
  integration tests), and
  `cargo clippy --locked --all-targets --all-features -- -D warnings` passed.
  `npm run test:browser -- --reporter=line` passed all 21 scenarios, including
  narrow and wide single-column layout, absent navigation state, inert former
  catalog parameters, authored links and history, direct and known-route
  PlantUML, repository and target scope, automatic refresh, and diagram
  failure/retry behavior.
- No PlantUML block changed in R2, so diagram validation was not applicable.
  R3 still needs to reconcile current user, risk, release, and verification
  documentation and complete the proposal's transition walkthrough.

Artifact Outcomes:

- started: `KnownDocuments`, fixed route-authorization lookup:
  [`src/viewer/known_documents.rs`](../../src/viewer/known_documents.rs) - maps
  only discovered identifiers to immutable document indices and has focused
  known/unknown checks.
- refined: Viewer session, routes, and page composition:
  [`src/viewer/`](../../src/viewer/) - removes catalog request and presentation
  responsibilities while preserving known routes, fixed link authorization,
  refresh, and diagram dispatch.
- refined: Embedded browser assets:
  [`src/viewer/assets/`](../../src/viewer/assets/) - removes navigation-only
  script, storage, grid, and component rules while retaining document,
  metadata, table, diagram, and refresh behavior.
- refined: `BTE-01`, browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - replaces
  catalog scenarios with focused-review and compatibility evidence.
- refined: Rust dependency manifest and lock:
  [`Cargo.toml`](../../Cargo.toml) and [`Cargo.lock`](../../Cargo.lock) - remove
  `form_urlencoded` as a direct Lens dependency while leaving dependencies
  required transitively by the HTTP stack locked.
- refined: `ADR-020`, focused document review:
  [`docs/decisions/adr-020-focused-document-review.md`](../decisions/adr-020-focused-document-review.md) - links the completed R2 construction evidence.
- retired: `DocumentCatalog`:
  `src/viewer/catalog.rs` - removes search, query parsing, pagination, and their
  catalog-only unit tests.
