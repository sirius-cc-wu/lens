# FEAT-02: Browse the Discovered Document Set

Status: started in N1

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
| `UC-08` | Filter the discovered document set | Medium |

`UC-07` extends the safe linked-document navigation of `UC-04`. It gives the
user a complete, session-scoped document catalog rather than requiring a
Markdown link to reach each document. `UC-08` helps when that catalog is long;
it does not create a content search capability.

## UC-07: Browse the Discovered Document Set

Primary actor: Developer or technical writer

Goal: Locate and open any document already authorized for the current viewing
session.

Preconditions:

- Lens has an active viewing session with a discovered document set.

Main success scenario:

1. The user opens a Lens document.
2. Lens presents the current document and the discovered document set.
3. Lens identifies the current document in that set.
4. The user selects another document in the set.
5. Lens displays the selected document in the same viewing session.

Extensions:

- 2a. If the document set contains only the current document, Lens still
  identifies it and does not imply that more documents are available.
- 4a. If a request does not name a document in the discovered set, Lens returns
  the existing guidance response and does not interpret it as a filesystem
  path.
- 4b. If browser scripting is unavailable, the discovered-document catalog
  remains usable for selection.

Special requirements:

- The catalog must contain only identifiers from the session's existing
  authorized document set. Rendering the catalog must not scan the filesystem,
  resolve new paths, or create another authorization rule.
- The current-document indication must agree with the document returned for the
  request and be exposed to assistive technology.
- Each catalog item must use the same known-document route as `UC-04`.

## UC-08: Filter the Discovered Document Set

Primary actor: Developer or technical writer

Goal: Narrow the displayed document catalog to locate an already authorized
document by its identifier.

Preconditions:

- The `UC-07` catalog is available.

Main success scenario:

1. The user supplies text to filter the catalog.
2. Lens limits the visible catalog to identifiers that match that text.
3. The user selects a matching document.
4. Lens displays the selected document as in `UC-07`.

Extensions:

- 2a. If no identifier matches, Lens communicates that no discovered document
  matches while retaining the current document and the filter control.
- 1a. If browser scripting is unavailable, Lens leaves the complete catalog
  available and does not claim to perform filtering.

Special requirements:

- Filtering is limited to already displayed authorized document identifiers. It
  must not read document contents, query the filesystem, or issue a request for
  an arbitrary path.
- The filter control has an accessible label and its result state is available
  to assistive technology.

## Trace

- Proposal: [Document Navigation Pane](../../improvement-proposals.md#3-document-navigation-pane)
- Existing safe-navigation basis: `UC-04`,
  [`FEAT-01`](../markdown-viewing/use-cases.md), and
  [ADR-003](../../decisions/adr-003-document-root-discovery.md)
- Planned system sequence and contract: `SSD-03` and `OC-03`
