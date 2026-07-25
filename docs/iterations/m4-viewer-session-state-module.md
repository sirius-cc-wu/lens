# Iteration: M4 Viewer Session State Module

Status: completed

Phase Intent:

- Consolidate the viewing session's shared state and automatic refresh behavior
  behind one cohesive Rust module boundary.

Goal:

- Move document state, authorized catalog ownership, renderer session state,
  document rendering, revision tracking, and the refresh loop together into
  `viewer::state` without changing ownership or concurrency behavior.

Risks Addressed:

- `R-03`: the extraction could rebuild or widen the fixed authorized document
  set used for routes and link rewriting.
- `R-08`: refresh could replace the last readable content after a failed read,
  change revision rules, or alter polling behavior.
- Moving shared state could add a lock, expand a lock lifetime across I/O or
  `.await`, or change atomic renderer-disable visibility.

Artifacts to Start:

- This M4 iteration record: captures session and concurrency invariants.

Artifacts to Refine:

- Viewer session state and refresh tests:
  [`src/viewer/state.rs`](../../src/viewer/state.rs)
- Viewer composition and route coordination:
  [`src/viewer/mod.rs`](../../src/viewer/mod.rs) and
  [`src/viewer/routes.rs`](../../src/viewer/routes.rs) - use the extracted session state.

Artifacts Consulted:

- `ADR-002`, fixed loopback viewing-session scope:
  [`docs/decisions/adr-002-loopback-viewer-scope.md`](../decisions/adr-002-loopback-viewer-scope.md)
- `ADR-007`, poll known document paths:
  [`docs/decisions/adr-007-poll-known-document-paths.md`](../decisions/adr-007-poll-known-document-paths.md)
- `DCD-01`, target state-module ownership:
  [`docs/features/markdown-viewing/uml-design.md`](../features/markdown-viewing/uml-design.md)

Decisions to Record:

- Keep `Arc<ViewerState>`, `RwLock<Vec<ViewerDocument>>`, the immutable known
  document set, and the atomic renderer-disable flag unchanged.
- Expose state and document members only at `pub(super)` when existing sibling
  route/page code needs them; add no crate-public surface.
- Keep filesystem reads outside the write-lock scope and renderer requests
  outside all document-lock scopes.

Trace:

- Proposal 13 -> `DCD-01` state module -> ADR-002/ADR-007 invariants -> refresh
  unit tests -> automatic-refresh and authorization browser scenarios

Exit Criteria:

- Session creation renders the same fixed document set and retains the same
  initial-document and renderer configuration.
- A changed readable document advances one revision and replaces its rendering;
  an unreadable document retains its rendering and revision.
- The refresh interval and lock/atomic ordering remain unchanged.
- State and refresh tests reside with the new module, and all checks pass.

Results:

- Moved `ViewerState`, `ViewerDocument`, state construction, document rendering,
  revision tracking, renderer session state, and the refresh loop into
  `src/viewer/state.rs`.
- Preserved the fixed `DocumentCatalog`-derived identifier set, 500-millisecond
  interval, read-before-render/write-after-render flow, compare-before-replace
  guard, and atomic acquire/release operations.
- Focused verification passed: `cargo test --locked viewer::state::tests` (two
  tests).
- Full verification passed: `cargo fmt --check`, `cargo test --locked` (53
  library tests and three CLI tests), `cargo clippy --locked --all-targets
  --all-features -- -D warnings`, and all 14 `npm run test:browser` scenarios.

Artifact Outcomes:

- started: `viewer::state` - owns viewing-session data, document rendering and
  revisions, renderer session state, refresh behavior, and refresh tests.
- refined: viewer composition and routes - use the same session value through a
  crate-private module boundary without changing the public API.
