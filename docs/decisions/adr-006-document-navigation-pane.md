# ADR-006: Derive the Navigation Pane from the Session Document Set

Status: superseded by [ADR-008](adr-008-paginated-session-catalog-search.md)

Date: 2026-07-18

## Context

The document navigation pane adds a sidebar and identifier search to help users
locate repository documentation. A new browser route or a search endpoint would
create another request surface near the authorization boundary of ADR-003. The
feature only needs the identifiers Lens has already discovered and authorized.

## Decision

Each successful document response includes a navigation pane derived from the
current `ViewerState` document set. The pane lists the complete lexical set of
known document identifiers, gives each the existing known-document route, and
marks the response's identifier as current.

The filter runs only in the browser against the rendered identifier list. It
does not issue a Lens request, read document content, scan the filesystem, or
alter session membership. Without browser scripting, the complete catalog and
its links remain available.

## Consequences

- The pane cannot list or make reachable a hidden, symbolic-link, or otherwise
  undiscovered document.
- The existing document route remains the only navigation route; unknown paths
  keep the ADR-003 guidance behavior.
- Search means identifier filtering, not document-content search. A future
  content-search proposal needs its own authorization, performance, and privacy
  design.
- Each document response carries the complete catalog. This is acceptable for
  the currently unbounded but ordinary repository scope; performance limits need
  evidence before adding pagination or an index.

## Trace

- Implementation: [C3 document navigation pane](../iterations/c3-document-navigation-pane.md)
- Requirements: [`FEAT-02`](../features/document-navigation-pane/use-cases.md)
- System sequence: [`SSD-03`](../features/document-navigation-pane/ssd-03-browse-document-catalog.md)
- Contract: [`OC-03`](../features/document-navigation-pane/oc-03-request-document-catalog.md)
- Existing authorization decision: [ADR-003](adr-003-document-root-discovery.md)
- Superseding scalability decision:
  [ADR-008](adr-008-paginated-session-catalog-search.md)
