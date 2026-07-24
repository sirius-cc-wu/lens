---
title: Lens
---

# Lens

Lens is a Linux CLI that opens repository Markdown and PlantUML diagrams in a
browser without requiring Obsidian.

## V1

```bash
cargo install --path . --locked
lens [TARGET]
```

V1 is a documentation-only viewer for Linux. It discovers Markdown documents
under the selected root, supports safe navigation between those documents, and
uses one session-fixed PlantUML server for diagrams. See the
[release readiness checklist](release-readiness.md) for verification and scope.

## Product Documents

These documents record the current inception baseline for Lens, a standalone
command-line Markdown viewer with PlantUML support. They are intentionally
small and will be refined as elaboration resolves the listed risks.

## Current Artifacts

- [Vision and business case](vision.md)
- [Improvement proposals](improvement-proposals.md)
- [Implemented proposal: remove renderer selection](proposals/remove-renderer.md)
- [Primary feature and use cases](features/markdown-viewing/use-cases.md) (`FEAT-01`)
- [Document navigation pane use cases](features/document-navigation-pane/use-cases.md) (`FEAT-02`)
- [Automatic refresh use cases](features/automatic-refresh/use-cases.md) (`FEAT-03`)
- [V1 UML design views](features/markdown-viewing/uml-design.md)
- [Server-only PlantUML rendering design](features/markdown-viewing/server-rendering-design.md)
- [Supplementary specification](supplementary-specification.md)
- [Glossary](glossary.md)
- [Risk list](risk-list.md)
- [ADR-001: Use the public PlantUML server for V1 rendering](decisions/adr-001-public-plantuml-rendering.md)
- [ADR-002: Restrict viewing-session scope](decisions/adr-002-loopback-viewer-scope.md)
- [ADR-003: Authorize document-root discovery](decisions/adr-003-document-root-discovery.md)
- [ADR-004: V1 Linux documentation-viewer scope](decisions/adr-004-v1-release-scope.md)
- [ADR-005: Use a controlled PlantUML renderer in browser tests](decisions/adr-005-controlled-renderer-browser-tests.md)
- [ADR-006: Derive the navigation pane from the session document set](decisions/adr-006-document-navigation-pane.md)
- [ADR-007: Poll only known document paths for automatic refresh](decisions/adr-007-poll-known-document-paths.md)
- [ADR-008: Search the session document catalog in bounded pages](decisions/adr-008-paginated-session-catalog-search.md)
- [ADR-009: Select a PlantUML renderer per viewing session (superseded)](decisions/adr-009-selectable-plantuml-rendering.md)
- [ADR-010: Package Linux binary release artifacts](decisions/adr-010-linux-binary-release-artifacts.md)
- [ADR-011: Keep diagram controls inside a viewing session (partially superseded)](decisions/adr-011-diagram-failure-controls.md)
- [ADR-012: Admit standalone PlantUML files to the document set (partially superseded)](decisions/adr-012-standalone-plantuml-files.md)
- [ADR-013: Support native browser launch and archives](decisions/adr-013-cross-platform-support.md)
- [ADR-014: Verify and publish tagged native releases](decisions/adr-014-release-automation.md)
- [ADR-015: Render leading YAML metadata safely](decisions/adr-015-yaml-frontmatter-rendering.md)
- [ADR-016: Persist navigation pane visibility in the browser tab](decisions/adr-016-collapsible-document-navigation-pane.md)
- [ADR-017: Use one session-fixed PlantUML server](decisions/adr-017-session-plantuml-server.md)
- [V1 release readiness](release-readiness.md)
- [Pending release notes](release-notes.md)
- [Development case](development-case.md)
- [Elaboration phase plan](elaboration-phase-plan.md)
- [Iteration records](iterations/)

## Lifecycle Status

Inception completed on 2026-07-18. Subsequent elaboration and construction
iterations resolved architectural risks and implemented the documented V1
behavior. The release-readiness checklist records the remaining release work.
