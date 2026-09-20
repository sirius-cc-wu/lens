---
type: "Improvement Proposal"
title: "Render Source-File Links In-Browser and Prune VS Code Integration"
description: "Renders qualifying repository source code files directly within the Lens browser viewer and entirely eliminates VS Code redirection and external editor dependencies."
id: "PROP-IN-BROWSER-SOURCE-RENDERING"
status: "proposed"
tags: [proposal, navigation, source-code, viewer]
---

# Render Source-File Links In-Browser and Prune VS Code Integration

Status: proposed

Authoritative decision:
- [ADR-025: In-Browser Source Code Rendering and Removal of VS Code Integration](../decisions/adr-025-in-browser-source-code-rendering.md) (supersedes [ADR-021](../decisions/adr-021-validated-vscode-source-links.md))

Specification & Design:
- [Requirements & Example Mapping](../features/markdown-viewing/source-rendering-requirements.md)
- [System Sequence Diagram (SSD-07)](../features/markdown-viewing/ssd-07-view-source-file.md)
- [Operation Contract (OC-07)](../features/markdown-viewing/oc-07-request-source-document.md)
- [Technical Design & Pruning Audit](../features/markdown-viewing/source-rendering-technical-design.md)
- [Worker Task Board (Iteration C10)](../iterations/c10-source-code-rendering-tasks.md)

## Summary

When reading repository documentation in Lens, relative links frequently point to implementation files, manifests, fixtures, or configurations. Under ADR-021, Lens rewrote these links to `vscode://file/...`, immediately redirecting the user to external desktop VS Code.

This proposal transitions source link handling from external editor redirection to native in-browser rendering and **entirely removes all VS Code integration**. Selecting a relative source file link opens the file inside Lens with line numbers, fragment line targeting (`#L42`), syntax legibility, and live auto-refresh. All vestigial VS Code helpers, `vscode://` URL generation, indicator badges, and ambiguous suffix parsing are pruned, establishing Lens as a 100% self-contained repository viewer.
