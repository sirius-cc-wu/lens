# FEAT-02: Browse the Discovered Document Set

Status: implemented in C3; scalable search implemented in C5

## System Boundary

Lens is the system under discussion. An active viewing session has already
created an authorized document set as specified by `FEAT-01`, `SSD-02`, and
ADR-003. Browser rendering and ordinary browser navigation are outside the
system boundary except where Lens supplies their content.

## Actors

| Actor | Goal |
|---|---|
| Developer or technical writer | Locate and open a document that Lens has already authorized for the active viewing session. |
| Operating system browser | Displays the Lens page and delivers the user's navigation requests. |

## Use-Case List

| ID | Use case | Priority |
|---|---|---|
| `UC-07` | Browse the discovered document set | High |
| `UC-08` | Search the discovered document set | High |

`UC-07` extends the safe linked-document navigation of `UC-04`. It gives the
user a session-scoped way to reach documents without requiring a Markdown link
to reach each one. `UC-08` helps when that document set is long; it does not
create a content search capability.

## UC-07: Browse the Discovered Document Set

Primary actor: Developer or technical writer

Goal: Locate and open any document already authorized for the current viewing
session.

Preconditions:

- Lens has an active viewing session with a discovered document set.

Main success scenario:

1. The user opens a Lens document.
2. Lens presents the current document and one bounded page of discovered
   document identifiers.
3. Lens identifies the current document when it appears in that result page.
4. The user selects another document from the result page.
5. Lens displays the selected document in the same viewing session.

Extensions:

- 2a. If more matching identifiers exist than fit in the result page, Lens
  provides a way to request the adjacent result page.
- 2b. If the document set contains only the current document, Lens still
  identifies it and does not imply that more documents are available.
- 4a. If a request does not name a document in the discovered set, Lens returns
  the existing guidance response and does not interpret it as a filesystem
  path.
- 4b. If browser scripting is unavailable, the result page and its document
  links remain usable for selection.

Special requirements:

- The result page must contain only identifiers from the session's existing
  authorized document set. Rendering it must not scan the filesystem, resolve
  new paths, or create another authorization rule.
- Each response contains no more than 50 result links. A blank search starts
  with the first lexical page of authorized identifiers.
- The current-document indication must agree with the document returned for the
  request when its identifier is returned, and be exposed to assistive
  technology.
- Each result item must use the same known-document route as `UC-04`.

## UC-08: Search the Discovered Document Set

Primary actor: Developer or technical writer

Goal: Locate an already authorized document by searching its identifier.

Preconditions:

- The active session has a discovered document set.

Main success scenario:

1. The user enters identifier text and submits the search.
2. Lens returns the first bounded page of authorized identifiers that match the
   supplied text, along with the current document.
3. The user optionally requests another result page.
4. The user selects a matching document.
5. Lens displays the selected document as in `UC-07`.

Extensions:

- 2a. If no identifier matches, Lens communicates that no discovered document
  matches while retaining the current document and the search form.
- 2b. If a query exceeds 256 UTF-8 bytes, Lens does not search and communicates
  the limit while retaining the current document and the search form.
- 3a. If the requested page is missing, zero, malformed, or beyond the result
  set, Lens returns the first result page.
- 1a. If browser scripting is unavailable, the native form submits the search
  and the native page links remain available.

Special requirements:

- Search is limited to the active session's authorized document identifiers. It
  must not read document contents, query the filesystem, or issue a request for
  an arbitrary path.
- Lens builds the identifier index at session start. It does not alter that
  index when a browser submits a search.
- The search form has an accessible label. Lens communicates the returned page,
  total matching count, and an empty or over-limit result state to assistive
  technology.
- Lens sends no search request until the user submits the form or follows a
  result-page link. It sends no search request while the user types.

## Trace

- Proposals: [Document Navigation Pane](../../improvement-proposals.md#3-document-navigation-pane)
  and [Scalable Document Navigation Search](../../improvement-proposals.md#11-scalable-document-navigation-search)
- Implementation: [C3 document navigation pane](../../iterations/c3-document-navigation-pane.md)
- Existing safe-navigation basis: `UC-04`,
  [`FEAT-01`](../markdown-viewing/use-cases.md), and
  [ADR-003](../../decisions/adr-003-document-root-discovery.md)
- System sequence: [`SSD-03`](ssd-03-browse-document-catalog.md)
- Operation contract: [`OC-03`](oc-03-request-document-catalog.md)
- Design realization and Rust mapping: [`RZ-02` and `DCD-02`](design.md)
- Decision: [ADR-008](../../decisions/adr-008-paginated-session-catalog-search.md)
- Verification: `BTE-01`,
  [`tests/browser/lens.spec.mjs`](../../../tests/browser/lens.spec.mjs)
