---
type: "Architecture Decision"
title: "ADR-012: Admit Standalone PlantUML Files to the Document Set"
description: "Admits visible standalone PlantUML files without broadening Lens into a general source-code viewer."
id: "ADR-012"
status: "partially superseded"
superseded_in_part_by: "ADR-017"
date: "2026-07-19"
tags: [architecture, decision, plantuml]
---

# ADR-012: Admit Standalone PlantUML Files to the Document Set

Status: partially superseded by ADR-017

Date: 2026-07-19

## Context

PlantUML source is often stored in a `.puml` file rather than embedded in a
Markdown fence. Supporting those diagrams must not expand Lens into a general
source-code browser or weaken document-root authorization.

## Decision

Lens accepts a visible regular `.puml` file as a direct target and discovers
visible regular `.puml` files beneath the existing document root. They receive
the same stable relative identifiers, navigation routes, hidden-entry
exclusion, symbolic-link exclusion, and canonical-root check as Markdown
documents.

The viewer renders a standalone `.puml` source as exactly one diagram using the
session's existing renderer choice and failure controls. Markdown link rewriting
remains Markdown-only; a `.puml` file is reachable through the delivered
navigation pane, not through a new path-resolution rule.

## Consequences

- Diagram authors can open a `.puml` file directly or select it from the
  authorized document set.
- The document set remains a finite, start-time authorization boundary.
- Lens still does not serve arbitrary source-code extensions or resolve browser
  paths as filesystem paths.
- A standalone diagram uses the same public, local, disabled, retry, and
  session-disable behavior as an embedded PlantUML block.

## Current Scope

ADR-017 supersedes the renderer-choice portion of this decision. A standalone
diagram now uses the viewing session's fixed PlantUML server and retains the
same visible-source failure and per-diagram retry behavior as an embedded
PlantUML block. Its document-set and authorization rules remain accepted.

## Trace

- Proposal: [Standalone PlantUML Files](../improvement-proposals.md#6-standalone-plantuml-files)
- Requirements: `UC-02`, `UC-03`, `UC-04`, and `UC-10` in
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md)
- Existing authorization: [ADR-003](adr-003-document-root-discovery.md)
- Iteration: [`P6`](../iterations/p6-standalone-plantuml-files.md)
- Partial successor: [ADR-017](adr-017-session-plantuml-server.md)
