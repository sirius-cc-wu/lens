---
type: "Iteration Record"
title: "Iteration: P1 Local PlantUML Rendering"
description: "Implements selectable public, local, or disabled PlantUML rendering for one viewing session."
id: "P1"
phase: "construction"
status: "completed"
tags: [iteration]
---

# Iteration: P1 Local PlantUML Rendering

Status: completed

Phase Intent:

- Construct the highest-value post-V1 privacy and availability slice without
  broadening browser-controlled access.

Goal:

- Let a user select public, local, or disabled PlantUML rendering for one
  viewing session while retaining the visible-source fallback.

Risks Addressed:

- `R-01`: reliance on the public renderer can expose diagram source and make
  rendering unavailable when the service cannot be reached.

Artifacts to Start:

- `ADR-009`, selectable PlantUML rendering:
  [`docs/decisions/adr-009-selectable-plantuml-rendering.md`](../decisions/adr-009-selectable-plantuml-rendering.md) - define the session-fixed renderer boundary.

Artifacts to Refine:

- `FEAT-01`, Markdown viewing use cases:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md) - specify the three renderer outcomes.
- Improvement proposals:
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - mark proposal 1 implemented.
- Risk list and glossary:
  [`docs/risk-list.md`](../risk-list.md) and [`docs/glossary.md`](../glossary.md) - record the updated mitigation and vocabulary.

Artifacts Consulted:

- `ADR-001`, public PlantUML rendering:
  [`docs/decisions/adr-001-public-plantuml-rendering.md`](../decisions/adr-001-public-plantuml-rendering.md)
- `BTE-01`, browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs)

Decisions to Record:

- `ADR-009`: make renderer selection an explicit CLI input captured once in
  `ViewerState`; do not accept renderer configuration from browser routes.

Trace:

- Proposal 1 -> `FEAT-01` (`UC-01`) -> `ADR-009` -> renderer-mode tests and
  `BTE-01`

Exit Criteria:

- The CLI exposes public, local, and disabled modes with public as the default.
- The local path runs only the installed PlantUML command, receives source over
  standard input, and returns only bounded SVG output.
- Disabled mode makes no diagram request and preserves source for the user.
- Existing public rendering, formatter, unit tests, browser tests, and Clippy
  checks pass.

Results:

- `RendererMode` is parsed once by the CLI and `DiagramRenderer` is owned by
  `ViewerState`. Public requests keep the existing controlled-renderer test
  seam; local rendering invokes `plantuml -tsvg -pipe`; disabled rendering emits
  no diagram image element.
- Unit tests cover disabled markup, the local command's standard-input SVG
  path, public failures, and CLI help. The browser suite verifies that disabled
  mode makes zero controlled-renderer requests while retaining PlantUML source.
- No PlantUML design block changed in this iteration, so PlantUML-server
  validation was not required.

Artifact Outcomes:

- started: `ADR-009`, selectable PlantUML rendering:
  [`docs/decisions/adr-009-selectable-plantuml-rendering.md`](../decisions/adr-009-selectable-plantuml-rendering.md) - records the session-boundary decision.
- refined: `FEAT-01`, Markdown viewing use cases:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md) - specifies public, local, and disabled behavior.
- refined: proposal 1, risk list, glossary, README, and `BTE-01` - record the
  delivered behavior and executable evidence.
