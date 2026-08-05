---
type: "Iteration Record"
title: "Iteration: C10 Crate Browser Launch"
description: "Promotes browser launching to a crate-level capability without changing foreground viewing behavior."
id: "C10"
phase: "construction"
status: "completed"
tags: [iteration, refactoring, browser]
---

# Iteration: C10 Crate Browser Launch

Status: completed

Phase Intent:

- Begin construction by making the existing browser-launch capability reusable
  by both the foreground viewer and the planned short-lived client.

Goal:

- Move browser command construction, platform selection, process spawning, and
  their tests from `viewer::browser` to a crate-level `browser` module while
  preserving every observable foreground behavior.

Risks Addressed:

- A structural move could change a platform launcher program, argument order,
  manual-URL fallback, or the public `lens::serve` path before service behavior
  is introduced.

Artifact Budget:

- create: `docs/iterations/c10-crate-browser-launch.md` - preserve the
  construction boundary, invariant, and verification evidence for the
  iteration-to-commit audit.
- keep with implementation: browser command behavior and module ownership -
  the moved code and existing platform test are the authoritative owners.
- omit: new design artifact - `BACKGROUND-VIEWER-DESIGN` already specifies the
  crate-level browser responsibility and this iteration does not change it.

Artifacts to Start:

- This C10 iteration record - capture the mechanical extraction and evidence.

Artifacts to Refine:

- Browser implementation and tests:
  [`src/browser.rs`](../../src/browser.rs)
- Crate and viewer composition roots:
  [`src/lib.rs`](../../src/lib.rs) and
  [`src/viewer/mod.rs`](../../src/viewer/mod.rs)

Artifacts Consulted:

- `BACKGROUND-VIEWER-DESIGN`, module placement and C10 handoff:
  [`docs/features/background-viewer-service/design.md`](../features/background-viewer-service/design.md)
- `ADR-013`, native browser commands:
  [`docs/decisions/adr-013-cross-platform-support.md`](../decisions/adr-013-cross-platform-support.md)

Decisions to Record:

- None. This iteration implements the already selected responsibility boundary.

Trace:

- `FEAT-04` -> `ADR-022` -> `DCD-04` crate-level `browser` module -> C10

Behavior-Preserving Evidence:

- Invariant: foreground `lens::serve` constructs the same platform commands,
  reports the same launch failure and manual URL, then serves until shutdown.
- Baseline: `cargo test --locked viewer::browser::tests` passed the platform
  command check, and `npm run test:browser` passed all 26 scenarios before the
  move.
- Transformation: move the cohesive module and its owning test, expose its
  launch function only within the crate, and redirect the existing viewer call.

Exit Criteria:

- Linux, macOS, and Windows commands retain the same programs and arguments.
- Foreground `serve` calls the same launch operation and retains its error and
  server-lifetime behavior.
- Browser implementation and tests have one crate-level owner usable by the
  future client.
- Formatting, locked Rust tests, Clippy, compiled-browser tests, and diff
  checks pass.

Results:

- Moved browser command construction, platform selection, process spawning, and
  the platform contract test unchanged to `src/browser.rs`.
- Kept `open_browser` visible only inside the crate and redirected the existing
  foreground viewer call without changing its launch or fallback flow.
- Focused verification passed at the relocated test path:
  `cargo test --locked browser::tests` (one test).
- Full verification passed: `cargo fmt --check`, `cargo test --locked` (77
  library and five CLI tests), `cargo clippy --locked --all-targets
  --all-features -- -D warnings`, `npm run test:browser` (26 scenarios), and
  `git diff --check`.
- No PlantUML block changed, so diagram validation was not applicable.

Artifact Outcomes:

- started and completed: C10 crate browser launch - records the preserved
  foreground behavior, mechanical move, and passing evidence.
- refined: crate and viewer composition roots - the crate owns the reusable
  capability and the viewer remains its existing caller.
- refined: browser implementation and tests - one cohesive crate-level module
  now owns the unchanged platform behavior.
