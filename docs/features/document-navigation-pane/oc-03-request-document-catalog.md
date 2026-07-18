# OC-03: Request a Bounded Document Catalog Page

Operation: `request_document(document_id, query?, page?)`

Cross References: `UC-07`, `UC-08`,
[SSD-03](ssd-03-browse-document-catalog.md), and
[ADR-003](../../decisions/adr-003-document-root-discovery.md)

Scope: Lens

Preconditions:

- A viewing session exists with a discovered document set and one initial
  document.

Postconditions on a known `document_id`:

- The response represents the document associated with `document_id` in the
  viewing session's document set.
- When `query` is absent or no more than 256 UTF-8 bytes, the response contains
  at most 50 identifiers from that session's document set. Each result matches
  the blank lexical catalog or the case-insensitive identifier query and is
  associated with its known-document route.
- The response identifies `document_id` as current when its identifier is in
  the returned page.
- The response reports whether more matching identifiers precede or follow the
  returned page, and preserves the accepted query in its page links.
- An absent, zero, malformed, or out-of-range `page` returns the first matching
  page.
- A query longer than 256 UTF-8 bytes produces no index search or result links
  and communicates the limit while retaining the requested document.
- No document, session membership, filesystem state, or authorization rule was
  created, changed, or removed.

Postconditions on an unknown `document_id`:

- The response is the existing Lens guidance page.
- Lens does not interpret `document_id` as a filesystem path, scan the
  filesystem, or add an identifier to the document set.

Open Issues:

- Hierarchical grouping and content search remain separate proposals because
  they need distinct user goals, privacy decisions, and scalability evidence.
