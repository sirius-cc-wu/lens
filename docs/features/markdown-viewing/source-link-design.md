---
type: "Software Design"
title: "Validated VS Code Source-Link Handoff Design"
description: "Assigns Rust responsibilities for authorizing repository source targets, serializing VS Code URLs, and rendering accessible editor links."
id: "SOURCE-LINK-DESIGN"
status: "active"
language: "Rust"
tags: [design, use-case-realization, source-link]
---

# Validated VS Code Source-Link Handoff Design

This design realizes `UC-06` without adding a browser-facing filesystem
operation. A source-link resolver is a small Rust component that classifies an
authored relative path against the viewing session's fixed root and returns an
editor URL only after every filesystem rule succeeds.

## RZ-06: Render a Validated Source Link

System operation: `request_document(document_id)`

### Collaborators

- `viewer::state`: owns the canonical document root for the complete viewing
  session and initiates rendering at startup and after readable changes.
- `markdown::render`: transforms Markdown events, gives known documents
  precedence, and adds the visible accessible editor indication.
- `source_link::SourceLinkResolver`: owns the root-relative filesystem
  authorization rule and VS Code URL construction.
- Filesystem: supplies non-following metadata, canonical paths, file type, and
  readability evidence.

### Interaction Summary

1. `viewer::state` asks `markdown::render` to render one known Markdown
   document, supplying its canonical path, the known document identifiers, and
   the session-owned `SourceLinkResolver`.
2. `markdown::render` preserves an authored destination when it is external,
   absolute, a same-document fragment, or otherwise not a relative file
   candidate.
3. `markdown::render` rewrites a known Markdown or PlantUML identifier to its
   Lens document route before considering an editor handoff.
4. For another relative candidate, `markdown::render` asks
   `SourceLinkResolver::resolve(current_document, authored_path)`.
5. The resolver decodes the authored path, normalizes it without crossing the
   fixed root, rejects hidden and symbolic components using non-following
   metadata, requires a readable regular file, canonicalizes it, and verifies
   that the result remains inside the canonical root.
6. The resolver serializes the canonical native path as a `vscode://file/`
   URL, preserving path separators and a Windows drive colon while
   percent-encoding other non-URL path bytes.
7. `markdown::render` uses that URL as the link destination and appends the
   visible text `(opens in VS Code)` inside the link.

Disallowed candidates return absence from the resolver. The renderer then
retains the authored destination, which preserves Lens's current guidance
behavior for unresolved local links.

## Responsibility Decisions

### Who retains the authorization boundary?

`MarkdownTarget` remains the creator and information source for the canonical
document root. It transfers that owned `PathBuf` to `viewer::state` with the
discovered documents. This keeps the session boundary fixed rather than
reconstructing it from a document path.

### Who authorizes and serializes source targets?

`SourceLinkResolver` is an information expert for the fixed root and a pure
fabrication that protects renderer cohesion. Filesystem inspection and URL
encoding can change independently of Markdown event transformation, so these
responsibilities belong in one cohesive `source_link` module with its focused
tests.

### Who chooses document navigation over editor handoff?

`markdown::render` already owns link-event transformation and receives the
known document identifiers. It remains the expert for precedence and
presentation: known document route first, source resolver second, authored
destination otherwise.

The viewer controller only supplies stable session data. It does not perform
path rules, construct URLs, or add presentation markup.

## DCD-05: Rust Module and Type View

| Construct | Responsibility and operations |
|---|---|
| `target::MarkdownTarget` `<<struct>>` | Own `document_root`, discovered documents, and initial index; consume itself into those parts at viewer startup. |
| `source_link` `<<module>>` | Keep path decoding, component inspection, and platform URL encoding private and cohesive. |
| `source_link::SourceLinkResolver` `<<struct>>` | Own one canonical `PathBuf`; expose `new(document_root)` and `resolve(&self, current_document, destination) -> Option<String>`. |
| `markdown` `<<module>>` | Resolve known document routes, delegate source candidates, and render the accessible indication. |
| `viewer::ViewerState` `<<struct>>` | Own one resolver for startup rendering and every refresh; add no lock because the resolver is immutable. |

`MarkdownTarget` owns the root until `viewer::serve` consumes it.
`ViewerState` then owns the immutable resolver through its existing `Arc`.
Rendering borrows the resolver; no stored borrow, new trait, runtime
polymorphism, or additional synchronization is needed.

## Error and Race Behavior

- Resolution is intentionally fail-closed: decoding, metadata, readability, or
  canonicalization errors produce no generated editor URL.
- A canonical path ending in a colon and number is ambiguous with VS Code's
  line and column syntax, so serialization produces no generated editor URL.
- Every path component beneath the root is inspected with symbolic-link
  metadata before final canonicalization. Final containment protects against a
  changed or platform-normalized target.
- Lens does not open or retain a source-file handle after rendering. A file may
  change after validation; selecting the URL asks VS Code to resolve the
  already emitted canonical path. This does not expand Lens's HTTP authority.
- Refresh uses the same resolver and may add or remove an editor URL as the
  target's current properties change, but the root remains immutable.

## Trace

- Requirement: [`UC-06`](use-cases.md#uc-06-open-a-referenced-repository-file-in-vs-code)
- System sequence: [`SSD-06`](ssd-06-open-source-link.md)
- Contract: [`OC-06`](oc-06-request-document-source-links.md)
- Decision:
  [`ADR-021`](../../decisions/adr-021-validated-vscode-source-links.md)
- Risks: [`R-02`, `R-03`, and `R-04`](../../risk-list.md)
