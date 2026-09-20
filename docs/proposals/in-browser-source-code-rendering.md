---
type: "Improvement Proposal"
title: "Render Source-File Links In-Browser"
description: "Renders qualifying repository source code files directly within the Lens browser viewer, replacing disruptive default VS Code redirection."
id: "PROP-IN-BROWSER-SOURCE-RENDERING"
status: "proposed"
tags: [proposal, navigation, source-code, viewer]
---

# Render Source-File Links In-Browser

Status: proposed

Authoritative decision:
- [ADR-025: In-Browser Source Code Rendering](../decisions/adr-025-in-browser-source-code-rendering.md)

Specification & Design:
- [Requirements & Example Mapping](../features/markdown-viewing/source-rendering-requirements.md)
- [System Sequence Diagram (SSD-07)](../features/markdown-viewing/ssd-07-view-source-file.md)
- [Operation Contract (OC-07)](../features/markdown-viewing/oc-07-request-source-document.md)
- [Technical Design](../features/markdown-viewing/source-rendering-technical-design.md)

## Summary

When reading repository documentation in Lens, relative links frequently point to implementation files, manifests, fixtures, or configurations. Under ADR-021, Lens rewrites these links to `vscode://file/...`, immediately redirecting the user to external desktop VS Code.

This proposal transitions source link handling from external editor redirection to native in-browser rendering. Selecting a relative source file link opens the file inside Lens with line numbers, fragment line targeting (`#L42`), syntax legibility, and live auto-refresh. The rendered source page also provides an optional "Open in VS Code" button for users who explicitly decide to edit the file.
