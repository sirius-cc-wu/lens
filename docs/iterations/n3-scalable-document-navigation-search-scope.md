# Iteration: N3 Scalable Document Navigation Search Scope

Status: completed

Phase Intent:

- Select proposal 11 and reduce its response-size, request-behavior, and
  authorization risks before revising the feature's scenarios or code.

Goal:

- Replace the complete, browser-filtered navigation catalog with bounded,
  server-rendered identifier search while keeping the current viewing session's
  authorization boundary fixed.

Risks Addressed:

- `R-03`: a navigation search could make an excluded document reachable or
  turn a browser value into a filesystem lookup.
- `R-09`: a complete catalog or unconstrained request pattern could make large
  document trees impractical.

Artifacts to Start:

- `ADR-008`, paginated session-catalog search:
  [`docs/decisions/adr-008-paginated-session-catalog-search.md`](../decisions/adr-008-paginated-session-catalog-search.md) - select the bounded interface and supersede ADR-006.

Artifacts to Refine:

- Improvement proposals:
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - record that proposal 11 is selected.
- `ADR-006`, session-derived navigation pane:
  [`docs/decisions/adr-006-document-navigation-pane.md`](../decisions/adr-006-document-navigation-pane.md) - retain its history while marking its complete-catalog decision superseded.
- Risk list: [`docs/risk-list.md`](../risk-list.md) - record the response-size
  and request-behavior risk as `R-09`.
- Supplementary specification:
  [`docs/supplementary-specification.md`](../supplementary-specification.md) - make the query and response limits durable quality constraints.
- Documentation index: [`docs/index.md`](../index.md) - link the new decision.

Artifacts Consulted:

- `FEAT-02`, browse the discovered document set:
  [`docs/features/document-navigation-pane/use-cases.md`](../features/document-navigation-pane/use-cases.md)
- `SSD-03`, browse an authorized document catalog:
  [`docs/features/document-navigation-pane/ssd-03-browse-document-catalog.md`](../features/document-navigation-pane/ssd-03-browse-document-catalog.md)
- `OC-03`, request a document catalog:
  [`docs/features/document-navigation-pane/oc-03-request-document-catalog.md`](../features/document-navigation-pane/oc-03-request-document-catalog.md)
- `RZ-02` and `DCD-02`, navigation-pane design:
  [`docs/features/document-navigation-pane/design.md`](../features/document-navigation-pane/design.md)
- `ADR-003`, document-root discovery:
  [`docs/decisions/adr-003-document-root-discovery.md`](../decisions/adr-003-document-root-discovery.md)

Decisions to Record:

- `ADR-008`: search the immutable session document catalog through an explicit
  GET form, with 256-byte queries and 50-item pages, rather than delivering the
  complete catalog or adding a JavaScript-only endpoint.

Trace:

- Proposal 11 -> `ADR-008` -> planned `FEAT-02` refinement (`UC-07`, `UC-08`)
  -> planned `SSD-03` / `OC-03` / `RZ-02` refinement -> construction checks

Exit Criteria:

- The accepted interaction defines pagination, no-JavaScript navigation,
  response and query limits, and when a browser sends a search request.
- The decision preserves the existing known-document route and explicitly
  rejects filesystem re-discovery and content search.
- The next iteration has concrete requirement, system-event, and executable
  verification questions.

Results:

- ADR-008 supersedes ADR-006's complete-catalog and browser-local-filter
  decision. Lens will build one immutable index from the active session's
  authorized identifiers and search only that index.
- A native GET form on the current document route accepts a 256-byte query and
  returns at most 50 case-insensitive identifier matches. One-based previous
  and next links provide pagination without JavaScript.
- Searches occur only on an explicit form submission or page-link request;
  there is no per-keystroke request, background search polling, separate search
  route, filesystem scan, content read, or independent rate limiter.
- N4 will refine `FEAT-02`, `SSD-03`, `OC-03`, and `RZ-02` around the accepted
  operation and define C5's unit and browser acceptance checks.

Artifact Outcomes:

- started: `ADR-008`, paginated session-catalog search:
  [`docs/decisions/adr-008-paginated-session-catalog-search.md`](../decisions/adr-008-paginated-session-catalog-search.md) - accepts the bounded, native-form interface.
- refined: Improvement proposals:
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - proposal 11 is selected in N3.
- refined: `ADR-006`, session-derived navigation pane:
  [`docs/decisions/adr-006-document-navigation-pane.md`](../decisions/adr-006-document-navigation-pane.md) - superseded by ADR-008 without losing the original decision context.
- refined: Risk list: [`docs/risk-list.md`](../risk-list.md) - adds `R-09`.
- refined: Supplementary specification:
  [`docs/supplementary-specification.md`](../supplementary-specification.md) - defines query and response limits.
- refined: Documentation index: [`docs/index.md`](../index.md) - links ADR-008.
