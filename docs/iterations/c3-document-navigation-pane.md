# Iteration: C3 Document Navigation Pane

Status: completed

Phase Intent:

- Construct and verify the thin navigation-pane slice whose requirements and
  design stabilized in N1 and N2.

Goal:

- Let a user browse and filter the current session's authorized document set
  from a responsive sidebar without adding a filesystem or server-query path.

Risks Addressed:

- `R-03`: navigation and search could reveal an excluded document or treat
  user input as a filesystem lookup.
- `R-02`: a document identifier containing HTML-significant characters could
  become unsafe page markup.

Artifacts to Start:

- None. N1 and N2 established the canonical feature, interaction, contract,
  realization, and decision artifacts.

Artifacts to Refine:

- `FEAT-02`, document navigation-pane use cases:
  [`docs/features/document-navigation-pane/use-cases.md`](../features/document-navigation-pane/use-cases.md) - mark the selected user goals as implemented and link executable evidence.
- `RZ-02` and `DCD-02`, navigation-pane design:
  [`docs/features/document-navigation-pane/design.md`](../features/document-navigation-pane/design.md) - record the implementation result.
- `BTE-01`, browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - add catalog, active-marker, filter, and sidebar-selection evidence.
- Improvement proposals:
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - mark proposal 3 implemented.
- Risk list: [`docs/risk-list.md`](../risk-list.md) - record the browser
  evidence for the document-set boundary.

Artifacts Consulted:

- `SSD-03`, browse an authorized document catalog:
  [`docs/features/document-navigation-pane/ssd-03-browse-document-catalog.md`](../features/document-navigation-pane/ssd-03-browse-document-catalog.md)
- `OC-03`, request a document catalog:
  [`docs/features/document-navigation-pane/oc-03-request-document-catalog.md`](../features/document-navigation-pane/oc-03-request-document-catalog.md)
- `ADR-006`, session-derived navigation pane:
  [`docs/decisions/adr-006-document-navigation-pane.md`](../decisions/adr-006-document-navigation-pane.md)

Decisions to Record:

- None. Construction implements the N2 `ADR-006` decision without a new
  variation point or route.

Trace:

- Proposal 3 -> `FEAT-02` (`UC-07`, `UC-08`) -> `SSD-03` -> `OC-03` ->
  `RZ-02` / `DCD-02` -> `BTE-01` and viewer unit tests

Exit Criteria:

- Every successful document response presents only the existing session catalog
  and marks its current document.
- A user can select a catalog item, filter it without changing the route, and
  receive a clear result when no identifier matches.
- A hidden document is neither listed nor searchable; no new route or
  filesystem access is introduced.
- Unit, browser, formatter, test, and linter checks pass.

Results:

- `ViewerState::navigation_pane` builds the catalog from the immutable
  `document_ids` authorization map. Existing Axum document routes and unknown
  path behavior remain unchanged.
- Each successful page has an accessible sidebar, labelled filter, current
  marker, no-match status, and no-script fallback. The client script filters
  only the entries already in the page; it makes no request.
- The focused browser checks first failed against the absent pane: the sidebar
  link timed out and the catalog test found no matching navigation link. After
  implementation, all five browser checks passed. The unit check also verifies
  one active marker and escaped HTML-significant identifiers.
- No durable design feedback was needed: the implementation matches the N2
  `ViewerState`-expert and stateless-layout decisions.

Artifact Outcomes:

- refined: `FEAT-02`, browse the discovered document set:
  [`docs/features/document-navigation-pane/use-cases.md`](../features/document-navigation-pane/use-cases.md) - marked implemented with browser evidence.
- refined: `RZ-02` and `DCD-02`, navigation-pane design:
  [`docs/features/document-navigation-pane/design.md`](../features/document-navigation-pane/design.md) - records the Rust implementation and responsive behavior.
- refined: `BTE-01`, browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - verifies selection from the pane, authorized membership, active marker, filtering, and hidden-document absence.
- refined: Improvement proposals:
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - proposal 3 is implemented.
- refined: Risk list: [`docs/risk-list.md`](../risk-list.md) - records C3
  evidence for the authorization boundary.
