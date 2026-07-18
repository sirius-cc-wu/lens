# OC-02: Open a Document Root

Operation: `open_document_root(target_path?)`

Cross References: `UC-02`, `UC-03`, [SSD-02](ssd-02-open-document-root.md)

Scope: Lens

Preconditions:

- None. The operation validates the actor-provided target path when present.

Postconditions on success:

- A document root was identified from the canonical current directory, a
  canonical directory target, or the canonical parent of a Markdown file target.
- A document set was created from supported Markdown files discovered within the
  document root.
- Every document in the document set is associated with a stable identifier
  relative to the document root.
- The explicitly named file, a root `README` document, a `docs/index` document,
  or the first discovered document became the viewing session's initial
  document.
- A viewing session was created for the document root, document set, and initial
  document.
- The source documents were not modified.

Postconditions on validation failure:

- No viewing session was created.
- Lens reports whether the target is missing, unreadable, hidden, a symbolic
  link, unsupported, or has no discoverable Markdown documents.

Open Issues:

- `UC-06` remains unresolved; the document set does not authorize code-file
  viewing.
- Large-repository document discovery limits need measurement before
  construction expands scope.
