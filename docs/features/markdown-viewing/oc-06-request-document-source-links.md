---
type: "Operation Contract"
title: "OC-06: Request a Document with Source Links"
description: "Specifies the authorization and response guarantees when Lens renders relative file links in a known Markdown document."
id: "OC-06"
operation: "request_document(document_id)"
traces: [UC-06, SSD-06]
status: "active"
tags: [analysis, operation-contract, source-link]
---

# OC-06: Request a Document with Source Links

Operation: `request_document(document_id)`

Cross References: `UC-06`,
[SSD-06](ssd-06-open-source-link.md), and
[ADR-021](../../decisions/adr-021-validated-vscode-source-links.md)

Scope: Lens

## Preconditions

- A viewing session exists with one canonical document root and a fixed known
  document set.

## Postconditions on a Known Markdown Document

- The response represents the current readable source of the requested known
  document.
- A link to a known Markdown or PlantUML document retained its Lens document
  route, including its authored query or fragment suffix.
- A relative link to an existing, readable, visible, non-symbolic regular file
  inside the canonical root received a `vscode://file/` destination containing
  the target's percent-encoded canonical absolute path.
- A supported positive line-number fragment received a VS Code line and column
  suffix that selects that line at column 1.
- Every Lens-generated `vscode:` link visibly states that it opens in VS Code,
  and that statement is part of the link's accessible name.
- A missing, unreadable, hidden, symbolic, directory, absolute, invalidly
  encoded, or out-of-root target, or a target whose canonical path ended in a
  colon and number, received no Lens-generated `vscode:` URL.
- A zero, malformed, or unsupported source-location fragment received no
  Lens-generated `vscode:` URL.
- External destinations, authored `vscode:` destinations, and same-document
  fragments retained their authored destinations.
- The response contained no source-file bytes, no source-file route, and no
  browser-supplied filesystem identifier.
- No editor process was started and no viewing-session authorization state
  changed.

## Postconditions on an Unknown Document

- Lens returned its existing unavailable-document response.
- The supplied identifier was not interpreted as a filesystem path.

## Refresh Guarantee

- Re-rendering a changed known Markdown document repeated these postconditions
  against the same fixed canonical root. It did not expand the known document
  set or launch an editor.
