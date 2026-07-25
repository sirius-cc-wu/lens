# Iteration: M6 Viewer Routes and Composition Root

Status: completed

Phase Intent:

- Complete construction by separating HTTP controllers from viewer startup and
  leave the module root focused on composition and public API declaration.

Goal:

- Move Axum router construction, handlers, response mapping, asset endpoints,
  and route tests into `viewer::routes`, then make `viewer/mod.rs` the thin
  owner of `lens::serve` and session startup.

Risks Addressed:

- `R-02` and `R-03`: moving handlers could change security headers, route
  matching, unknown-document guidance, or authorized catalog lookup.
- `R-01`: the diagram controller could change renderer-disable, missing-diagram,
  success, or failure status mapping, or hold a state lock across `.await`.
- Converting the module root could change the public `lens::serve` re-export or
  server/browser/shutdown lifecycle.

Artifacts to Start:

- This M6 iteration record: captures final construction and acceptance evidence.
- Route controllers and owned tests:
  [`src/viewer/routes.rs`](../../src/viewer/routes.rs)

Artifacts to Refine:

- Viewer composition root:
  [`src/viewer/mod.rs`](../../src/viewer/mod.rs)
- `DCD-01` and `DCD-02`, implemented viewer responsibility designs:
  [`docs/features/markdown-viewing/uml-design.md`](../features/markdown-viewing/uml-design.md) and
  [`docs/features/document-navigation-pane/design.md`](../features/document-navigation-pane/design.md)
- Proposal 13:
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - record implementation.

Artifacts Consulted:

- `ADR-002`, loopback route and authorization boundary:
  [`docs/decisions/adr-002-loopback-viewer-scope.md`](../decisions/adr-002-loopback-viewer-scope.md)
- `ADR-011`, diagram failure and disable controls:
  [`docs/decisions/adr-011-diagram-failure-controls.md`](../decisions/adr-011-diagram-failure-controls.md)
- `BTE-01`, complete browser suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs)

Decisions to Record:

- Keep handlers as private controller functions and expose only `router` at
  `pub(super)` to the composition root.
- Keep `serve` in `viewer/mod.rs` so `lib.rs` continues to re-export the same
  `lens::serve` path. The module root owns target consumption, renderer/client
  selection, refresh-task startup, listener binding, browser launch, server
  startup, and shutdown coordination.

Trace:

- Proposal 13 -> `DCD-01`/`DCD-02` -> route/controller extraction -> route unit
  tests -> complete Rust and `BTE-01` suites

Exit Criteria:

- Every existing route, method, extractor, header, status code, response body,
  and fallback remains unchanged.
- No document lock is held during an asynchronous renderer request.
- Route tests reside with `viewer::routes` and `viewer/mod.rs` contains no route,
  page, renderer-transport, state-refresh, or browser-command implementation.
- `lens::serve` retains its signature, path, startup behavior, and shutdown
  behavior.
- The full formatting, locked Rust test, Clippy, and browser suites pass, and
  the refined PlantUML design renders successfully.

Results:

- Moved seven registered routes, the fallback handler, response composition,
  asset endpoints, and six route tests into `src/viewer/routes.rs`.
- Replaced the former 398-line `src/viewer.rs` with a 58-line
  `src/viewer/mod.rs` composition root. `src/lib.rs` still declares `mod viewer`
  and re-exports `viewer::serve` unchanged.
- Kept the diagram clone before the renderer `.await`, preserving the short
  document-lock scope and the existing response mapping.
- Refined the canonical navigation design to assign catalog lookup, route
  control, and markup to their implemented modules. Its changed `DCD-02`
  PlantUML block rendered successfully through the configured server (HTTP 200).
- Focused verification passed: `cargo test --locked viewer::routes::tests` (six
  tests).
- Full verification passed: `cargo fmt --check`, `cargo test --locked` (53
  library tests and three CLI tests), `cargo clippy --locked --all-targets
  --all-features -- -D warnings`, and all 14 `npm run test:browser` scenarios.

Artifact Outcomes:

- started: `viewer::routes` - owns Axum controllers, response mapping, asset
  endpoints, and route tests.
- refined: viewer module root - is now the composition root and sole owner of
  the unchanged public `serve` function.
- refined: `DCD-01`, `DCD-02`, and proposal 13 - record the implemented
  responsibility boundaries and complete construction evidence.
