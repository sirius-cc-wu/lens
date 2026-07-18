# FEAT-03: Refresh Changed Markdown Documents

Status: implemented in C4

## System Boundary

Lens is the system under discussion. A viewing session has already created an
authorized document set as specified by `FEAT-01`, `SSD-02`, and ADR-003. A
developer's editor writes files and the browser renders responses outside the
Lens boundary; Lens observes only the paths in that fixed document set and
supplies the refreshed response.

## Actors

| Actor | Goal |
|---|---|
| Developer or technical writer | See an open Markdown document update after saving it, without manually reloading the browser page. |
| Operating system browser | Displays the current Lens document and asks Lens whether that document's revision changed. |
| Filesystem | Makes the current contents of an already authorized document available to Lens. |

## Use-Case List

| ID | Use case | Priority |
|---|---|---|
| `UC-09` | Refresh a changed Markdown document | High |

`UC-09` extends the viewing session established by `UC-02` or `UC-03`. It does
not discover newly created files or broaden the session's document set.

## UC-09: Refresh a Changed Markdown Document

Primary actor: Developer or technical writer

Goal: Read the current saved contents of an open, discovered Markdown document
without manually refreshing the Lens page.

Preconditions:

- Lens has an active viewing session with a fixed discovered document set.
- The browser is displaying one document from that set.

Main success scenario:

1. The developer saves changed Markdown in the displayed document.
2. Lens observes that the saved contents of that known document differ from its
   current session representation.
3. Lens replaces the document's rendered representation and advances its
   document revision.
4. The browser learns that the displayed document has a newer revision.
5. The browser requests the current document again.
6. Lens returns the new rendered representation.
7. The developer reads the changed document without an explicit browser reload.

Extensions:

- 2a. If the document is momentarily missing, unreadable, or contains an
  incomplete save, Lens keeps the last successfully rendered representation and
  retries later. It does not publish a new revision for the failed read.
- 2b. If a file is created, deleted, renamed, hidden, or made a symbolic link
  after the session starts, Lens does not add it to or remove it from the
  document set. A fresh Lens session is required to discover a different set.
- 4a. If the changed document is not the browser's current document, Lens
  refreshes its stored representation but the browser does not reload the
  unrelated current page. The new representation appears when that known
  document is selected.
- 4b. If the browser's revision check fails transiently, the browser retains
  the currently displayed document and tries again later.
- 4c. If browser scripting is unavailable, ordinary document requests still
  return the latest successfully refreshed representation, but automatic page
  reload is unavailable.

Special requirements:

- Lens observes only canonical paths belonging to the session's existing
  document set. It must not rescan the document root or accept a browser path
  as a filesystem path while refreshing.
- A successful refresh renders Markdown and PlantUML placeholders with the
  same safety and document-link rules as the initial session load.
- A failed refresh must not replace readable browser content with an error or
  create a reload loop.
- The browser's revision query must identify only the current known document
  and must not return source text or add an authorization rule.

## Trace

- Proposal: [Automatic Refresh](../../improvement-proposals.md#4-automatic-refresh)
- Session and authorization basis: `UC-02` through `UC-04`,
  [`FEAT-01`](../markdown-viewing/use-cases.md), and
  [ADR-003](../../decisions/adr-003-document-root-discovery.md)
- System sequence: [`SSD-04`](ssd-04-refresh-changed-document.md)
- Operation contract: [`OC-04`](oc-04-refresh-changed-document.md)
- Design realization and Rust mapping: [`RZ-03` and `DCD-03`](design.md)
- Decision: [ADR-007](../../decisions/adr-007-poll-known-document-paths.md)
- Verification: `BTE-01`, [`tests/browser/lens.spec.mjs`](../../../tests/browser/lens.spec.mjs)

## Construction Result

`BTE-01` saves the displayed fixture document, observes a revision-only `0`
response before the change, and then verifies that the browser displays the new
heading and body without calling `page.reload()`. Viewer unit tests cover a
successful refresh, retention after a missing-file read, and the 404 response
for an unknown revision path.
