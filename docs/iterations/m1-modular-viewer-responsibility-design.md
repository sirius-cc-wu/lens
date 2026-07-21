# Iteration: M1 Modular Viewer Responsibility Design

Status: completed

Phase Intent:

- Elaborate proposal 13 into cohesive, idiomatic Rust module responsibilities
  before moving implementation.

Goal:

- Define a mechanical viewer split that preserves all observable behavior, the
  public `lens::serve` path, session ownership, and the fixed authorized
  document set.

Risks Addressed:

- Structural drift: moving code could accidentally change route behavior,
  renderer errors, refresh ordering, browser launching, or public API paths.
- Concurrency drift: separating state could alter `Arc`, `RwLock`, or atomic
  ownership, or hold a blocking lock across asynchronous work.
- Module churn: splitting by file size could create pass-through modules rather
  than cohesive capability boundaries.

Artifacts to Start:

- This M1 iteration record: captures the selected boundaries, invariants, exit
  criteria, and construction handoff without duplicating the canonical design.

Artifacts to Refine:

- `DCD-01`, Rust module and type view:
  [`docs/features/markdown-viewing/uml-design.md`](../features/markdown-viewing/uml-design.md) - assign viewer responsibilities to modules and show their dependency direction.

Artifacts Consulted:

- Proposal 13, modular viewer responsibilities:
  [`docs/improvement-proposals.md`](../improvement-proposals.md)
- Viewer implementation and owned tests:
  [`src/viewer.rs`](../../src/viewer.rs)
- `R-01`, `R-02`, `R-03`, and `R-08`, renderer, browser-content,
  authorization, and refresh risks: [`docs/risk-list.md`](../risk-list.md)
- `BTE-01`, complete browser suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs)

Decisions to Record:

- Keep `viewer::serve` as the composition root and public re-export target.
- Keep session data and refresh behavior together in `viewer::state`, including
  the existing `Arc`, document `RwLock`, renderer configuration, and atomic
  session-disable flag.
- Give browser launching, diagram transport, page composition, and Axum routes
  separate cohesive modules. Retain `viewer::catalog` as the authorized
  navigation index owner.
- Store JavaScript and CSS as dedicated assets read with `include_str!`, so
  deployment remains a single binary with no runtime asset lookup.
- Use crate-private module functions and the existing closed renderer enum; add
  no traits, service wrappers, or public API surface.

Trace:

- Proposal 13 -> `DCD-01` -> M2 browser extraction -> M3 rendering extraction
  -> M4 state extraction -> M5 page/assets extraction -> M6 route extraction
  -> complete Rust and `BTE-01` verification

Exit Criteria:

- Every existing viewer responsibility and owned test has one planned module.
- The dependency direction leaves `viewer::serve` as a thin composition root
  and avoids circular module ownership.
- Ownership, errors, route paths, refresh behavior, renderer limits, and
  browser-visible markup are explicitly unchanged.
- The modified PlantUML design renders through the configured PlantUML server.
- The pre-refactoring baseline passes formatting, locked Rust tests, Clippy, and
  the complete browser suite.

Results:

- Selected `browser`, `state`, `rendering`, `page`, and `routes` as capability
  modules around the existing `catalog` module. Each concern has its own
  dependencies and tests and can change independently.
- Assigned construction to five mechanical iterations so each move remains
  independently reviewable and reversible. Tests move with their behavior.
- Preserved `Arc<ViewerState>`, its document `RwLock`, the atomic renderer flag,
  and the closed `DiagramRenderer` enum; no new concurrency or polymorphism was
  introduced.
- The modified `DCD-01` block rendered successfully through the configured
  PlantUML server (HTTP 200).
- Baseline verification passed: `cargo fmt --check`, `cargo test --locked` (53
  library tests and three CLI tests), `cargo clippy --locked --all-targets
  --all-features -- -D warnings`, and all 14 `npm run test:browser` scenarios.

Artifact Outcomes:

- refined: `DCD-01`, Rust module and type view - records the cohesive viewer
  modules, owned state, and dependency direction selected for construction.
- consulted: proposal 13, repository risks, viewer tests, and `BTE-01` - define
  the behavior-preservation invariant and final verification boundary.
