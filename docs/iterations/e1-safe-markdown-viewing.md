# Iteration: E1 Safe Markdown Viewing

Status: planned

Goal:

- Establish whether Lens can safely present a Markdown file in a local browser
  session using the public PlantUML server.

Risks Addressed:

- `R-01`: public PlantUML server availability and response behavior
- `R-02`: untrusted Markdown and SVG
- `R-03`: target scope escape
- `R-06`: browser launch and process lifetime

Artifacts to Start:

- `SSD-01`, system sequence for `UC-01`: pending canonical path - identify
  system events before detailed design.
- `OC-01`, operation contract for resolving and opening a Markdown target:
  pending canonical path - make target scope and error effects precise.

Artifacts to Refine:

- `FEAT-01`, primary feature and use cases:
  [`docs/features/markdown-viewing.md`](../features/markdown-viewing.md) -
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

- Pending execution.

Artifact Outcomes:

- planned: `SSD-01`, system sequence - not started.
- planned: `OC-01`, resolve and open Markdown target contract - not started.
- planned: `FEAT-01`, primary feature and use cases - not yet refined.
