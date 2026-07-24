---
type: "Operation Contract"
title: "OC-02: Open a Document Root"
description: "Specifies document-root discovery, selection, and viewing-session state after opening a supported target."
id: "OC-02"
operation: "open_document_root(target_path?, scope?)"
traces: [UC-02, UC-03, SSD-02]
status: "active"
tags: [analysis, operation-contract]
---

# OC-02: Open a Document Root

Operation: `open_document_root(target_path?, scope?)`

Cross References: `UC-02`, `UC-03`, [SSD-02](ssd-02-open-document-root.md)

Scope: Lens

Preconditions:

- None. The operation validates the actor-provided target path when present.

Postconditions on success:

- In repository scope, the document root was the nearest canonical ancestor
  containing a supported `.git` marker for a current-directory, directory, or
  supported file target.
- When no supported repository marker existed, the document root was the
  canonical current directory, canonical directory target, or canonical parent
  of a supported file target.
- In target scope, repository recognition was skipped and the document root was
  the canonical current directory, canonical directory target, or canonical
  parent of a supported file target.
- A supported `.git` marker was either a non-symbolic-link directory or a
  regular file. A `.git` symbolic link did not establish a repository root.
- Repository recognition did not invoke Git or read `.git` contents.
- A document set was created from supported Markdown and `.puml` files
  discovered within the document root.
- Every document in the document set is associated with a stable identifier
  relative to the document root.
- An explicitly named file remained the initial document when repository
  recognition made its document root broader than its parent.
- A selected directory or current directory remained the initial-selection
  anchor: its root `README`, its `docs/index`, or its first supported document
  in lexical path order became the initial document when available.
- When a repository-scoped directory anchor contained no supported document,
  the repository root's `README`, `docs/index`, or first supported document
  became the initial document.
- A viewing session was created for the document root, document set, and initial
  document.
- The source documents were not modified.

Postconditions on validation failure:

- No viewing session was created.
- Lens reports whether the target is missing, unreadable, hidden, a symbolic
  link, unsupported, or has no discoverable Markdown or PlantUML documents.
- A repository-scoped target below a hidden repository entry was reported as
  hidden rather than creating a session whose discovery excludes its
  initial-selection anchor.

Open Issues:

- `UC-06` remains unresolved; the document set does not authorize code-file
  viewing.
- Large-repository discovery limits remain a residual risk because the default
  scope can authorize more supported documents than a target contains.
