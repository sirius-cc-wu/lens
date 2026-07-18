# OC-03: Request a Document Catalog

Operation: `request_document(document_id)`

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
- The response's document catalog contains exactly the identifiers in that
  session's document set, each associated with its known-document route.
- The catalog identifies `document_id` as the current document.
- No document, session membership, filesystem state, or authorization rule was
  created, changed, or removed.

Postconditions on an unknown `document_id`:

- The response is the existing Lens guidance page.
- Lens does not interpret `document_id` as a filesystem path, scan the
  filesystem, or add an identifier to the document set.

Open Issues:

- The catalog is initially a complete lexical list. Hierarchical grouping and
  content search are separate proposals because they would need new user goals
  and scalability evidence.
