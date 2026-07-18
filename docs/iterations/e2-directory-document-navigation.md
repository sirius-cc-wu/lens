# Iteration: E2 Directory Document Navigation

Status: completed

Goal:

- Validate document-root selection and safe navigation between discovered
  Markdown documents.

Risks Addressed:

- `R-03`: file and directory resolution can expose files outside the requested
  repository.
- `R-04`: the intended boundary between documentation browsing and code
  browsing is unclear.

Artifacts to Start:

- `SSD-02`, open and navigate a document root:
  [`docs/features/markdown-viewing/ssd-02-open-document-root.md`](../features/markdown-viewing/ssd-02-open-document-root.md)
- `OC-02`, open a document root:
  [`docs/features/markdown-viewing/oc-02-open-document-root.md`](../features/markdown-viewing/oc-02-open-document-root.md)
- `ADR-003`, document-root discovery:
  [`docs/decisions/adr-003-document-root-discovery.md`](../decisions/adr-003-document-root-discovery.md)

Artifacts to Refine:

- `FEAT-01`, primary feature and use cases:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md)
- Risk list: [`docs/risk-list.md`](../risk-list.md)

Trace:

- `UC-02`, `UC-03`, `UC-04` -> `SSD-02` -> `OC-02` -> document-root
  discovery and navigation tests

Exit Criteria:

- `lens`, `lens <directory>`, and `lens <markdown-file>` select an initial
  document from a defined document root.
- A relative link to a discovered document renders that document.
- Unknown, out-of-root, and symlinked targets cannot read additional files.
- `UC-06` remains explicitly deferred or receives product-owner clarification.

Results:

- Implemented one typed target model for no argument, directory, and Markdown
  file invocation. Directory and current-directory targets select a root README,
  then `docs/index`, otherwise the first document in lexical path order; a file
  target remains the initial document.
- The viewer discovers in-root `.md` and `.markdown` files, excludes symbolic
  links and hidden entries, and maps each document to an identifier relative to
  the document root. It selects a root README, then `docs/index`, before using
  lexical path order.
- Markdown links are rewritten only when they resolve to a known document
  identifier. Unknown, out-of-root, and non-document paths remain on the
  Lens-owned 404 guidance path without a filesystem lookup.
- Automated tests cover root README selection, direct-file sibling discovery,
  `.git` exclusion, symbolic-link exclusion, known and unknown document routes,
  and relative-link rewriting.
- `UC-06` remains deferred; the document set does not authorize code-file
  viewing.

Artifact Outcomes:

- started: `SSD-02`, open and navigate a document root:
  [`docs/features/markdown-viewing/ssd-02-open-document-root.md`](../features/markdown-viewing/ssd-02-open-document-root.md) -
  identified document-root and linked-document system events.
- started: `OC-02`, open a document root:
  [`docs/features/markdown-viewing/oc-02-open-document-root.md`](../features/markdown-viewing/oc-02-open-document-root.md) -
  defined document-set, selection, and failure postconditions.
- started: `ADR-003`, document-root discovery:
  [`docs/decisions/adr-003-document-root-discovery.md`](../decisions/adr-003-document-root-discovery.md) -
  recorded root authorization and navigation constraints.
- refined: `FEAT-01`, primary feature and use cases:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md) -
  detailed target-root opening and discovered-document navigation.
