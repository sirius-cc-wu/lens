# Iteration: M2 Browser Launch Module

Status: completed

Phase Intent:

- Begin construction with the smallest independent viewer capability and prove
  that tests can move with their owning module without changing behavior.

Goal:

- Place browser-launch command construction, platform selection, spawning, and
  platform tests in one cohesive `viewer::browser` module.

Risks Addressed:

- A moved platform branch could change the executable, argument order, or empty
  Windows `start` title argument.
- Moving launch code could change the manual-URL fallback or the public
  `lens::serve` path.

Artifacts to Start:

- This M2 iteration record: captures the extraction and verification evidence.

Artifacts to Refine:

- Browser launch implementation and tests:
  [`src/viewer/browser.rs`](../../src/viewer/browser.rs)
- Viewer composition root:
  [`src/viewer/mod.rs`](../../src/viewer/mod.rs) - delegate the existing launch call to the new module.

Artifacts Consulted:

- `DCD-01`, target module ownership:
  [`docs/features/markdown-viewing/uml-design.md`](../features/markdown-viewing/uml-design.md)
- `ADR-013`, cross-platform browser launch:
  [`docs/decisions/adr-013-cross-platform-support.md`](../decisions/adr-013-cross-platform-support.md)

Decisions to Record:

- Keep `BrowserPlatform`, `BrowserCommand`, command construction, and platform
  selection private to `viewer::browser`; expose only `open_browser` to its
  parent module.

Trace:

- Proposal 13 -> `DCD-01` browser module -> ADR-013 command contract -> platform
  unit test -> complete Rust and browser suites

Exit Criteria:

- Linux, macOS, and Windows construct the same programs and arguments as before.
- `serve` calls the same `open_browser` operation and retains its error fallback.
- The platform test resides with the browser module.
- Formatting, locked Rust tests, Clippy, and the complete browser suite pass.

Results:

- Moved browser platform and command types, command construction, platform
  selection, process spawning, and the existing platform test to
  `src/viewer/browser.rs`.
- Kept `open_browser` as the only parent-visible function and left the `serve`
  call and fallback messages unchanged.
- Focused verification passed:
  `cargo test --locked viewer::browser::tests` (one test).
- Full verification passed: `cargo fmt --check`, `cargo test --locked` (53
  library tests and three CLI tests), `cargo clippy --locked --all-targets
  --all-features -- -D warnings`, and all 14 `npm run test:browser` scenarios.

Artifact Outcomes:

- started: `viewer::browser` - owns browser-launch construction and its platform
  contract tests.
- refined: viewer composition root - delegates launch without changing its
  public API, server lifecycle, or error behavior.
