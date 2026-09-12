---
type: "Architecture Decision"
title: "ADR-023: Open Mermaid Diagrams in Standalone Scalable SVG Views"
description: "Defines client-side standalone SVG generation via Blob URLs with viewport adaptation for Mermaid diagrams."
id: "ADR-023"
status: "accepted"
date: "2026-09-13"
tags: [architecture, decision, mermaid, svg, viewer]
---

# ADR-023: Open Mermaid Diagrams in Standalone Scalable SVG Views

Status: accepted

Date: 2026-09-13

## Context

Lens formats Markdown documents in a centered prose column capped at 920 pixels (`min(920px, calc(100% - 2rem))`). While optimal for text readability, complex Mermaid diagrams (such as large architectural flowcharts, extensive sequence diagrams, and class hierarchies) are scaled down to fit this column width, rendering text labels illegibly small. Additionally, Mermaid's layout engine automatically injects inline `max-width: <calculated>px;` styles onto the root `<svg>` element, which artificially caps vector scaling even when ample viewport space is available. Lens requires a capability to view rendered Mermaid diagrams in a standalone, freely resizable browser tab while maintaining strict offline operation, zero server-side JavaScript execution, and existing Content Security Policy protections.

## Decision

Lens generates standalone Mermaid SVG views entirely within the client browser using in-memory Blob URLs (`image/svg+xml;charset=utf-8`).

Each rendered Mermaid figure includes an accessible link (`<a class="diagram-open-link" data-mermaid-open target="_blank" rel="noopener noreferrer" hidden>Open SVG</a>`). This control remains hidden until client-side Mermaid rendering successfully completes.

Upon successful rendering, Lens post-processes the generated SVG DOM before creating the Blob:
1. Removes the inline `maxWidth` style cap injected by Mermaid so the diagram is free to expand across large viewports.
2. Sets root `<svg>` attributes `width="100%"` and `height="100%"` while preserving the calculated `viewBox` and aspect ratio.
3. Ensures an explicit solid background (`background-color: #ffffff;`) is present on the root `<svg>` so diagrams retain clear contrast when viewed in browsers with dark image-viewer canvases.

The serialized, adapted SVG is wrapped into an in-memory Blob and registered with `URL.createObjectURL()`. The resulting URL is assigned to the `Open SVG` link's `href` attribute, and the control is revealed. Any previously generated Blob URL on that element is revoked via `URL.revokeObjectURL()` to prevent memory leaks during client-side re-renders.

To prevent URL corruption, Lens's client-side link handler in `app.js` explicitly exempts URLs with the `blob:` scheme from session token rewriting (`?token=...`), ensuring that clicking the link navigates directly to the exact registered in-memory resource.

## Consequences

- Users can open any rendered Mermaid diagram in a separate browser tab as a pure vector SVG document (`image/svg+xml`) that scales dynamically with window resizing and supports native browser zoom and "Save image as..." actions.
- The feature operates 100% offline with zero external network requests and zero server roundtrips.
- The server remains completely stateless with respect to Mermaid rendering; no server-side JavaScript runtime (Node.js/V8) or SVG cache endpoints are required.
- Content Security Policy headers (`default-src 'self'`, `style-src 'self' 'unsafe-inline'`) remain unchanged and fully enforced; standalone SVG documents inherit origin styling rules without weakening script execution restrictions.
- Standard user-initiated navigation via `<a target="_blank">` is immune to browser popup blockers.
- Exempting `blob:` links from token query appending prevents `net::ERR_FILE_NOT_FOUND` navigation errors caused by corrupting browser Blob registry keys.
- The "Open SVG" link is strictly gated on successful rendering and remains hidden on syntax or layout errors.

## Trace

- Use case: [`UC-11`](../features/markdown-viewing/use-cases.md)
- Requirements: [`FEAT-01-REQ-MERMAID-SVG`](../features/markdown-viewing/mermaid-svg-requirements.md)
- Technical design: [`mermaid-svg-technical-design.md`](../features/markdown-viewing/mermaid-svg-technical-design.md)
