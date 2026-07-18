# ADR-001: Use the Public PlantUML Server for V1 Rendering

Status: accepted

Date: 2026-07-18

## Context

Lens must render PlantUML fenced blocks in Markdown. A local renderer would add
installation and packaging work, while the first version prioritizes a simple
path to a working viewer.

## Decision

Lens V1 sends PlantUML block source to the public PlantUML server at
`https://www.plantuml.com/plantuml` for rendering.

V1 accepts that diagram source leaves the machine. It does not provide a local
renderer, a privacy-preserving alternative, or a renderer-selection setting.

## Consequences

- PlantUML rendering does not require users to install Java, a local PlantUML
  server, or a container.
- Rendering depends on network access and the public service's availability,
  limits, and response behavior.
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
