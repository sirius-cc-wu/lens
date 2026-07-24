---
type: "Architecture Decision"
title: "ADR-003: Authorize Navigation Through a Discovered Document Root"
description: "Defines document-root discovery as the authorization boundary for repository document navigation."
id: "ADR-003"
status: "accepted"
date: "2026-07-18"
tags: [architecture, decision, security]
---

# ADR-003: Authorize Navigation Through a Discovered Document Root

Status: accepted

Date: 2026-07-18

## Context

Lens E2 adds directory and current-directory targets plus navigation between
Markdown documents. Resolving browser URLs directly as filesystem paths would
break the fixed-session security boundary established by ADR-002.

## Decision

Lens creates a canonical document root from the current directory, a visible
non-symbolic-link directory argument, or the canonical parent of a visible
non-symbolic-link supported file argument. It discovers `.md`, `.markdown`, and
`.puml` files recursively inside that root, excluding symbolic links and hidden
files and directories.

The session assigns each discovered document an identifier relative to the
document root. It serves only known identifiers. Markdown links are rewritten
only when they resolve to a known in-root document; other local paths retain the
Lens-owned 404 guidance behavior.

For a directory or current-directory target, Lens initially opens a root
`README.md` or `README.markdown` when present, then `docs/index.md` or
`docs/index.markdown`, and otherwise the first document in lexical path order.
A file target remains the initial document.

## Consequences

- `lens`, `lens <directory>`, `lens <markdown-file>`, and `lens <puml-file>`
  have one typed target
  model and no environment-variable or process-global override path.
- A direct Markdown or PlantUML file can navigate to discovered siblings under
  its canonical parent.
- URL traversal and symlinked files cannot expand the document set.
- Directory scanning and code-file browsing remain separate concerns.

## Trace

- Use cases: [`FEAT-01`](../features/markdown-viewing/use-cases.md)
- System sequence: [`SSD-02`](../features/markdown-viewing/ssd-02-open-document-root.md)
- Contract: [`OC-02`](../features/markdown-viewing/oc-02-open-document-root.md)
- Security decision: [ADR-002](adr-002-loopback-viewer-scope.md)
