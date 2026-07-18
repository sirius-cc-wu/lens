# OC-04: Refresh a Changed Markdown Document

Operation: `observe_document_change(document_id)`

Cross References: `UC-09`,
[SSD-04](ssd-04-refresh-changed-document.md), and
[ADR-007](../../decisions/adr-007-poll-known-document-paths.md)

Scope: Lens

Preconditions:

- A viewing session exists with a fixed document set.
- `document_id` identifies one document in that set; the session watcher, not
  a browser-provided path, supplies the identifier.

Postconditions when the current source is read successfully and differs from
the stored source:

- The known document's rendered representation became the result of rendering
  the newly read source under the existing session's Markdown, diagram, and
  known-link rules.
- The known document's revision became a value newer than its previous
  revision.
- The document set, document identifiers, document-root authorization rule,
  and initial-document selection remain unchanged.
- No repository file was created, changed, renamed, or removed by Lens.

Postconditions when the current source is unchanged:

- The document's rendered representation and revision remain unchanged.

Postconditions when the current source cannot be read:

- The document's last successfully rendered representation and revision remain
  unchanged.
- The document set and its authorization rule remain unchanged.

Related query: `request_document_revision(document_id, revision)` returns the
current revision of a known document without changing a document, revision,
document set, or filesystem state. For an unknown `document_id`, Lens returns
its existing guidance response and does not perform filesystem work.

Open Issues:

- The first slice uses a bounded polling interval rather than an operating
  system file-notification dependency. A later scalability measurement may
  justify replacing that implementation without changing this contract.
