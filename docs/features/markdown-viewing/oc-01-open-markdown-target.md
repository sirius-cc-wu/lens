---
type: "Operation Contract"
title: "OC-01: Open a Markdown Target"
description: "Specifies the viewing-session state and authorized resources established when Lens opens one Markdown target."
id: "OC-01"
operation: "open_markdown_target(target_path)"
traces: [UC-01, SSD-01]
status: "active"
tags: [analysis, operation-contract]
---

# OC-01: Open a Markdown Target

Operation: `open_markdown_target(target_path)`

Cross References: `UC-01`, [SSD-01](ssd-01-open-markdown-target.md)

Scope: Lens

Preconditions:

- None. The operation validates the actor-provided target path.

Postconditions on success:

- A Markdown document was identified from the canonical form of `target_path`.
- A viewing session was created for exactly that Markdown document.
- The viewing session has a local loopback URL for the Markdown document.
- The viewing session exposes only the selected document, its fixed viewer
  assets, and diagrams recognized in that document.
- Lens attempted to open the viewing session's URL in the operating system
  browser and made the URL available to the developer.
- The source Markdown document was not modified.

Postconditions on validation failure:

- No viewing session was created.
- Lens reports whether the target is missing, unreadable, a directory, or an
  unsupported file type.

Open Issues:

- `UC-02` and `UC-03` will define the target-root and navigation rules for
  directories. This contract intentionally authorizes only one Markdown file.
- `request_diagram(diagram_id)` needs a separate contract if caching or retry
  behavior becomes a durable product decision.
