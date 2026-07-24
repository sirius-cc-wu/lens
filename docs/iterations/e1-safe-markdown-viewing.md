---
type: "Iteration Record"
title: "Iteration: E1 Safe Markdown Viewing"
description: "Establishes a safe file-target vertical slice with controlled Markdown and PlantUML rendering."
id: "E1"
phase: "elaboration"
status: "completed"
tags: [iteration]
---

# Iteration: E1 Safe Markdown Viewing

Status: completed

Goal:

- Establish whether Lens can safely present a Markdown file in a local browser
  session using the public PlantUML server.

Risks Addressed:

- `R-01`: public PlantUML server availability and response behavior
- `R-02`: untrusted Markdown and SVG
- `R-03`: target scope escape
- `R-06`: browser launch and process lifetime

Artifacts to Start:

- `SSD-01`, system sequence for `UC-01`:
  [`docs/features/markdown-viewing/ssd-01-open-markdown-target.md`](../features/markdown-viewing/ssd-01-open-markdown-target.md) -
  identify system events before detailed design.
- `OC-01`, operation contract for resolving and opening a Markdown target:
  [`docs/features/markdown-viewing/oc-01-open-markdown-target.md`](../features/markdown-viewing/oc-01-open-markdown-target.md) -
  make target scope and error effects precise.

Artifacts to Refine:

- `FEAT-01`, primary feature and use cases:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md) -
  refine `UC-01` from executable-spike feedback.
- Supplementary specification:
  [`docs/supplementary-specification.md`](../supplementary-specification.md) -
  set public-renderer, sanitization, and performance constraints from evidence.
- Risk list: [`docs/risk-list.md`](../risk-list.md) - update probabilities and
  mitigations from the spike.

Artifacts Consulted:

- Vision and business case: [`docs/vision.md`](../vision.md)
- Glossary: [`docs/glossary.md`](../glossary.md)
- Development case: [`docs/development-case.md`](../development-case.md)
- `ADR-001`, public PlantUML rendering:
  [`docs/decisions/adr-001-public-plantuml-rendering.md`](../decisions/adr-001-public-plantuml-rendering.md)
- `ADR-002`, loopback viewer scope:
  [`docs/decisions/adr-002-loopback-viewer-scope.md`](../decisions/adr-002-loopback-viewer-scope.md)

Decisions to Record:

- Public PlantUML server integration: record a decision only if request,
  timeout, or failure behavior has durable product consequences.
- Browser-session scope: listener binding, target-root authorization, and
  browser-launch fallback - create a decision record if the choice has durable
  security or deployment consequences.

Trace:

- `UC-01` -> `SSD-01` -> `OC-01` -> target-resolution and Markdown-viewing
  spike -> automated boundary tests

Exit Criteria:

- A runnable vertical slice accepts one readable Markdown file, serves it only
  over a loopback session, and displays it in a browser or reports a manual URL.
- Tests demonstrate that traversal and symlink inputs cannot read beyond the
  authorized target scope.
- One normal PlantUML block and one failed block have observable, documented
  behavior.
- Successful, invalid, unavailable, and delayed public-renderer responses have
  observable, documented behavior.
- The resulting evidence updates `R-01` through `R-03` and supports a credible
  `E2` plan.

Results:

- Implemented a Rust vertical slice for direct `.md` and `.markdown` file
  targets. It starts a `127.0.0.1` session on an ephemeral port and reports a
  manual URL when the browser launcher is unavailable.
- The viewer renders Markdown, replaces PlantUML blocks with local diagram
  routes, and proxies them to the public PlantUML SVG endpoint. It preserves
  source and presents an error when rendering fails.
- A live request to the documented PlantUML SVG endpoint returned valid SVG.
  Automated tests cover valid, invalid, unavailable, and delayed renderer
  responses; missing and unsupported targets; raw Markdown HTML; traversal
  paths; and canonical symlink targets.
- Renderer requests use a 10-second timeout and a 2 MiB response limit.
- Residual risk: production availability and rate limiting of the public
  PlantUML service still need observation, and directory authorization is not
  yet designed.

Artifact Outcomes:

- started: `SSD-01`, system sequence:
  [`docs/features/markdown-viewing/ssd-01-open-markdown-target.md`](../features/markdown-viewing/ssd-01-open-markdown-target.md) -
  identified the target, document, and diagram system events.
- started: `OC-01`, resolve and open Markdown target contract:
  [`docs/features/markdown-viewing/oc-01-open-markdown-target.md`](../features/markdown-viewing/oc-01-open-markdown-target.md) -
  defined single-file authorization and validation effects.
- refined: `FEAT-01`, primary feature and use cases:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md) -
  recorded the direct-file boundary of the executable slice.
