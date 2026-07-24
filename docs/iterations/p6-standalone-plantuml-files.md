---
type: "Iteration Record"
title: "Iteration: P6 Standalone PlantUML Files"
description: "Extends the authorized document model to visible standalone PlantUML files."
id: "P6"
phase: "construction"
status: "completed"
tags: [iteration]
---

# Iteration: P6 Standalone PlantUML Files

Status: completed

Phase Intent:

- Extend the authorized document model to a focused diagram-source format
  without crossing into general source-code viewing.

Goal:

- Let a user open and navigate a visible `.puml` file as one Lens diagram.

Risks Addressed:

- `R-03`: adding another file type could broaden filesystem access beyond the
  start-time document set.

Artifacts to Start:

- `ADR-012`, standalone PlantUML files:
  [`docs/decisions/adr-012-standalone-plantuml-files.md`](../decisions/adr-012-standalone-plantuml-files.md) - define the accepted extension and rendering boundary.

Artifacts to Refine:

- `FEAT-01`, Markdown and PlantUML viewing use cases:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md) - add `UC-10` and broaden target rules precisely.
- `ADR-003` and `OC-02`:
  [`docs/decisions/adr-003-document-root-discovery.md`](../decisions/adr-003-document-root-discovery.md) and [`docs/features/markdown-viewing/oc-02-open-document-root.md`](../features/markdown-viewing/oc-02-open-document-root.md) - preserve the document-set contract.
- Proposal list, README, supplementary specification, glossary, and risk list:
  [`docs/improvement-proposals.md`](../improvement-proposals.md), [`README.md`](../../README.md), [`docs/supplementary-specification.md`](../supplementary-specification.md), [`docs/glossary.md`](../glossary.md), and [`docs/risk-list.md`](../risk-list.md) - record scope and evidence.
- `BTE-01`, browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - navigate and render a standalone diagram.

Artifacts Consulted:

- `ADR-009`, renderer selection:
  [`docs/decisions/adr-009-selectable-plantuml-rendering.md`](../decisions/adr-009-selectable-plantuml-rendering.md)
- `ADR-011`, diagram controls:
  [`docs/decisions/adr-011-diagram-failure-controls.md`](../decisions/adr-011-diagram-failure-controls.md)

Decisions to Record:

- `ADR-012`: include only `.puml` in the existing discovery set and render it
  as one diagram through the existing route and renderer boundary.

Trace:

- Proposal 6 -> `UC-10` -> `ADR-012` -> target, Markdown-rendering, and
  `BTE-01` standalone-diagram checks

Exit Criteria:

- A direct visible `.puml` target selects itself and discovers supported
  siblings only beneath its canonical parent.
- A discovered `.puml` identifier appears in the existing navigation pane and
  renders through the existing diagram route.
- Hidden, symbolic-link, unsupported, and browser-provided paths do not widen
  document membership or filesystem access.
- Formatter, unit tests, browser tests, and Clippy pass.

Results:

- The target module classifies supported documents as Markdown or PlantUML,
  preserving all prior root, hidden-entry, symbolic-link, and canonical-path
  rules. The viewer uses that classification to send standalone source to a
  one-diagram renderer representation.
- Unit tests cover the direct target, document kind, and document-scoped
  standalone renderer representation. Browser evidence follows a navigation
  pane link to `architecture.puml` and verifies the controlled renderer's
  second SVG request.
- No PlantUML design block changed in this iteration, so PlantUML-server
  validation was not required.

Artifact Outcomes:

- started: `ADR-012`, standalone PlantUML files:
  [`docs/decisions/adr-012-standalone-plantuml-files.md`](../decisions/adr-012-standalone-plantuml-files.md) - establishes the supported source-format boundary.
- refined: `FEAT-01`, `ADR-003`, `OC-02`, user documentation, risk list, and
  `BTE-01` - record the constrained `.puml` target and navigation behavior.
