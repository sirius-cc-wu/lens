# Iteration: M3 Diagram Rendering Module

Status: completed

Phase Intent:

- Isolate the viewer's external diagram-rendering transports behind one narrow,
  crate-private module boundary.

Goal:

- Move public-server requests, local-command execution, renderer client
  construction, limits, and transport tests into `viewer::rendering` without
  changing route responses or renderer selection.

Risks Addressed:

- `R-01`: extraction could weaken timeout, response-status, content-type,
  invalid-diagram header, or output-size enforcement.
- Local rendering could change command arguments, standard input, process
  cleanup, SVG validation, or error context.
- Moving asynchronous code could accidentally hold viewer state locks across an
  external request.

Artifacts to Start:

- This M3 iteration record: captures transport invariants and verification.

Artifacts to Refine:

- Diagram rendering implementation and tests:
  [`src/viewer/rendering.rs`](../../src/viewer/rendering.rs)
- Viewer route coordination:
  [`src/viewer/routes.rs`](../../src/viewer/routes.rs) - delegate the existing diagram request.

Artifacts Consulted:

- `ADR-001`, public PlantUML rendering:
  [`docs/decisions/adr-001-public-plantuml-rendering.md`](../decisions/adr-001-public-plantuml-rendering.md)
- `ADR-009`, selectable rendering:
  [`docs/decisions/adr-009-selectable-plantuml-rendering.md`](../decisions/adr-009-selectable-plantuml-rendering.md)
- `DCD-01`, target rendering-module ownership:
  [`docs/features/markdown-viewing/uml-design.md`](../features/markdown-viewing/uml-design.md)

Decisions to Record:

- Keep the existing closed `DiagramRenderer` enum as the renderer selection
  boundary; the module needs no trait or wrapper type.
- Expose only client construction and `request_diagram` to sibling viewer code;
  keep public/local transport details and their constants private.

Trace:

- Proposal 13 -> `DCD-01` rendering module -> ADR-001/ADR-009 constraints ->
  five transport tests -> browser renderer scenarios

Exit Criteria:

- Public rendering retains its timeout, status, error-header, content-type, and
  two-stage size checks.
- Local rendering retains `-tsvg -pipe`, piped input/output, kill-on-drop,
  timeout, exit-status, size, and SVG checks.
- Route status codes and visible failure text are unchanged.
- Rendering tests reside with the new module, and all required checks pass.

Results:

- Moved renderer client construction, public HTTP streaming, local process
  execution, limits, and their five tests to `src/viewer/rendering.rs`.
- Kept the route's diagram clone before `.await`, so no viewer document lock is
  held during public or local rendering.
- Focused verification passed:
  `cargo test --locked viewer::rendering::tests` (five tests).
- Full verification passed: `cargo fmt --check`, `cargo test --locked` (53
  library tests and three CLI tests), `cargo clippy --locked --all-targets
  --all-features -- -D warnings`, and all 14 `npm run test:browser` scenarios.

Artifact Outcomes:

- started: `viewer::rendering` - owns renderer transport policy, resource
  limits, client construction, and transport tests.
- refined: viewer route coordination - delegates rendering through a narrow
  function while preserving response mapping and lock lifetime.
