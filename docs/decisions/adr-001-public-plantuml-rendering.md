---
type: "Architecture Decision"
title: "ADR-001: Use the Public PlantUML Server for V1 Rendering"
description: "Records the original V1 decision to use the public PlantUML server for diagram rendering."
id: "ADR-001"
status: "superseded"
superseded_by: "ADR-009"
date: "2026-07-18"
tags: [architecture, decision]
---

# ADR-001: Use the Public PlantUML Server for V1 Rendering

Status: superseded by ADR-009

Date: 2026-07-18

## Context

Lens must render PlantUML fenced blocks in Markdown. A local renderer would add
installation and packaging work, while the first version prioritizes a simple
path to a working viewer.

## Decision

Lens V1 sent PlantUML block source to the public PlantUML server at
`https://www.plantuml.com/plantuml` for rendering. It did not provide a local
renderer, a privacy-preserving alternative, or a renderer-selection setting.

## Consequences

- The original public-renderer decision is retained as historical context for
  the public default now defined by ADR-009.
- Lens proxies renderer requests through the local viewer with a 10-second
  timeout and 2 MiB response limit.
- Lens must keep the original source visible and show an actionable error if a
  renderer request fails.
- The renderer output remains untrusted and must satisfy the browser-content
  sanitization boundary in the supplementary specification.

## Trace

- Requirement: `UC-01` in
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md)
- Risk: `R-01` in [`docs/risk-list.md`](../risk-list.md)
- Verification plan: [`E1`](../iterations/e1-safe-markdown-viewing.md)
- Successor: [ADR-009](adr-009-selectable-plantuml-rendering.md)
