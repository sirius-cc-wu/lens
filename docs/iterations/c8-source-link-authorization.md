---
type: "Iteration Record"
title: "Iteration: C8 Source-Link Authorization"
description: "Implements and verifies fail-closed source-target validation and cross-platform VS Code URL serialization."
id: "C8"
phase: "construction"
status: "completed"
tags: [iteration]
---

# Iteration: C8 Source-Link Authorization

Status: completed

Phase Intent:

- Implement the security-sensitive filesystem and URL core behind ADR-020
  through focused executable examples.

Goal:

- Resolve a relative link to a canonical VS Code file URL only when its current
  target is a readable, visible, non-symbolic regular file inside the fixed
  viewing-session root.

Risks Addressed:

- `R-03`: traversal, absolute paths, hidden components, symbolic links, and
  non-files must fail closed.
- Cross-platform compatibility: native separators, Windows drive letters,
  spaces, and non-ASCII text must retain their filesystem meaning in the URL.

Artifacts to Start:

- Source-link resolver and focused unit checks:
  [`src/source_link.rs`](../../src/source_link.rs) - own shared relative-path
  normalization, filesystem authorization, and URL serialization.

Artifacts to Refine:

- Target and viewer session root ownership:
  [`src/target.rs`](../../src/target.rs) and
  [`src/viewer/state.rs`](../../src/viewer/state.rs) - retain and borrow the
  canonical root during initial and refreshed rendering.
- Markdown link resolution:
  [`src/markdown.rs`](../../src/markdown.rs) - use shared normalization and
  give known documents precedence before source-file resolution.
- Dependency manifest:
  [`Cargo.toml`](../../Cargo.toml) and [`Cargo.lock`](../../Cargo.lock) - make
  the already locked percent-encoding implementation a direct dependency.

Artifacts Consulted:

- `OC-06`, request a document with source links:
  [`docs/features/markdown-viewing/oc-06-request-document-source-links.md`](../features/markdown-viewing/oc-06-request-document-source-links.md)
- `ADR-020`, validated VS Code source links:
  [`docs/decisions/adr-020-validated-vscode-source-links.md`](../decisions/adr-020-validated-vscode-source-links.md)
- `DCD-05`, Rust responsibility design:
  [`docs/features/markdown-viewing/source-link-design.md`](../features/markdown-viewing/source-link-design.md)

Decisions to Record:

- None. C8 implements ADR-020's authorization and serialization rules without
  changing its external-editor scope.

Trace:

- `UC-06` -> `SSD-06` -> `OC-06` -> `ADR-020` -> `DCD-05` -> source resolver,
  Markdown destination checks, and focused Rust examples

Test-Driven Evidence:

- Oracle: OC-06 and ADR-020 define qualifying target properties, fail-closed
  cases, and platform URL syntax independently from production code.
- Slice size: authorization and serialization share one stable resolver
  boundary and are the highest-risk prerequisite for renderer presentation.
- Discrimination: after the direct dependency's lockfile entry was refreshed
  offline, `cargo test --locked source_link` ran nine semantic checks against a
  resolver that deliberately returned no result. All nine failed on their
  expected positive or comparison assertions; no compilation or setup failure
  was counted as behavior evidence.
- Green: the same focused command passed after implementation. The final
  focused resolver and Markdown selection set passed 11 checks, followed by all
  80 library tests, all 5 CLI tests, Clippy with warnings denied, formatting,
  and diff checks.

Exit Criteria:

- A qualifying in-root file resolves to its canonical `vscode://file/` URL.
- Spaces, non-ASCII text, native separators, and Windows drive syntax serialize
  without changing path meaning.
- Hidden, symbolic, missing, directory, absolute, invalidly encoded, and
  out-of-root targets return no generated URL.
- Known Markdown and PlantUML identifiers retain first precedence in link
  resolution.
- Initial and refreshed rendering use the same immutable canonical root.
- Focused and complete Rust formatting, test, and Clippy checks pass.

Results:

- Added a cohesive `source_link` module that decodes and normalizes relative
  paths, rejects malformed escapes and URI schemes, inspects every root-relative
  component with non-following metadata, requires a readable regular file,
  canonicalizes it, and proves final containment.
- Added platform URL serialization that uses forward separators, removes the
  Windows verbatim-path prefix, preserves drive-letter colons, and
  percent-encodes spaces, non-ASCII text, and URL delimiters.
- Retained the canonical document root in `MarkdownTarget`, transferred it to
  viewer state, and reused one immutable resolver for initial and refreshed
  Markdown rendering.
- Replaced Markdown-only normalization with the shared resolver vocabulary.
  Known Markdown and PlantUML targets now take precedence, while qualifying
  non-document files receive editor URLs without authored query or fragment
  suffixes.
- Focused tests cover valid files, spaces and non-ASCII text, Windows path
  syntax, malformed encoding, absolute paths, traversal, hidden files and
  directories, file and directory symbolic links, missing files, directories,
  document precedence, suffix removal, and refresh reuse.
- `cargo test --locked` passed all 80 library and 5 CLI tests.
  `cargo clippy --locked --all-targets --all-features -- -D warnings`,
  `cargo fmt --check`, and `git diff --check` also passed.

Artifact Outcomes:

- started and verified: source-link resolver and focused unit checks at
  [`src/source_link.rs`](../../src/source_link.rs).
- refined: target and viewer state - retain and reuse the canonical session
  root without adding synchronization.
- refined: Markdown rendering - shares normalized relative paths and preserves
  known-document precedence before source resolution.
- refined: Cargo dependency metadata - records `percent-encoding` as a direct,
  locked dependency.
- refined: `R-03` - links the completed fail-closed construction evidence.
