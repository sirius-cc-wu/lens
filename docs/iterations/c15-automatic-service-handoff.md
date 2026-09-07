---
type: "Iteration Record"
title: "Iteration: C15 Automatic Service Handoff"
description: "Makes ordinary Lens commands auto-start or reuse the background service, open the acknowledged URL, and return without ending the view."
id: "C15"
phase: "construction"
status: "completed"
tags: [iteration, cli, process-lifecycle, browser]
---

# Iteration: C15 Automatic Service Handoff

Status: completed

Phase Intent:

- Complete the user-visible vertical slice through the already verified
  protocol, endpoint, controller, and viewer-session boundaries.

Goal:

- Make an ordinary `lens [target]` command start or reuse one background
  service, wait only for a ready URL, attempt browser launch once, and return
  while the page and automatic refresh remain available.

Risks Addressed:

- `R-06`: detached process and browser launch behavior differs by platform and
  desktop environment.
- `R-11`: concurrent first commands, delivery failure, stale endpoints, and
  acknowledgment timeout could hang or report false success.
- `R-12`: the CLI must carry invocation-specific root, scope, and PlantUML
  authority without moving browser launch into the background process.

Artifact Budget:

- create: `docs/iterations/c15-automatic-service-handoff.md` - preserve the
  end-to-end behavior, process evidence, and iteration-to-commit mapping.
- keep with implementation: client orchestration, detached process flags,
  service connection handling, deadlines, and Rust integration checks - their
  source modules and tests are the authoritative owners.
- update: compiled-browser harness and CLI checks - prove the real command exits
  while its retained page and refresh behavior continue.
- omit: separate operator guide in this iteration - C16 owns user-facing
  transition documentation after thresholds and native evidence are known.

Artifacts to Start:

- This C15 iteration record - capture automatic startup and handoff evidence.
- Client and detached-process capabilities:
  [`src/service/client.rs`](../../src/service/client.rs) and
  [`src/service/process.rs`](../../src/service/process.rs)

Artifacts to Refine:

- Background server connection handling and service root:
  [`src/service/server.rs`](../../src/service/server.rs) and
  [`src/service/mod.rs`](../../src/service/mod.rs)
- CLI entry point, crate composition, CLI checks, and browser harness:
  [`src/main.rs`](../../src/main.rs), [`src/lib.rs`](../../src/lib.rs),
  [`tests/cli.rs`](../../tests/cli.rs), and
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs)

Artifacts Consulted:

- `OC-07`, acknowledgment and browser failure effects:
  [`docs/features/background-viewer-service/oc-07-request-target-view.md`](../features/background-viewer-service/oc-07-request-target-view.md)
- `RZ-04`, client, process, endpoint, server, and browser collaboration:
  [`docs/features/background-viewer-service/design.md`](../features/background-viewer-service/design.md)

Decisions to Record:

- Use a three-second startup deadline and ten-second request acknowledgment
  deadline initially; C16 measures ordinary startup and reuse before finalizing
  performance guidance.

Trace:

- `UC-11` -> `SSD-07` -> `OC-07` -> `RZ-04` -> C15 CLI and browser acceptance

Test-Driven Evidence:

- Oracle: OC-07 requires a ready listener before success, exactly one browser
  attempt, no browser attempt on rejection, manual URL on launch failure, and
  command completion without waiting for the page to close.
- Slice size: client connect/start, detached candidate, server frame handling,
  browser handoff, and compiled CLI acceptance form one end-to-end behavior.
- Discrimination: the first focused run of
  `missing_service_then_command_starts_service_and_returns_after_view_ready`
  failed because the client returned its explicit `NotImplemented` error before
  any service connection or browser attempt.

Exit Criteria:

- A missing service starts automatically and the command returns after a ready
  URL; an available service is reused.
- Concurrent first commands elect one service and both receive isolated ready views.
- A verified stale Unix socket recovers without a cleanup command.
- Browser launch failure reports the ready manual URL and leaves the session available.
- Target rejection returns the actionable CLI error and makes no browser attempt.
- The compiled-browser suite proves the ordinary command exits while its page
  and automatic refresh continue to work.
- Formatting, locked Rust and CLI tests, Clippy, compiled-browser tests, and
  diff checks pass.

Results:

- Added a short-lived client composition root that captures the invoking
  directory, target, scope, and PlantUML environment value; creates a random
  request identifier; starts or reuses the service; applies three-second
  startup and ten-second acknowledgment deadlines; and launches the browser
  only after a matching ready response.
- Added native detached-process construction. Unix candidates start a new
  session with `setsid`; Windows candidates use `DETACHED_PROCESS` and
  `CREATE_NEW_PROCESS_GROUP`; both discard inherited standard streams and
  enter a hidden CLI mode.
- Added the background accept loop. It atomically claims the per-user endpoint,
  treats a losing startup candidate as normal completion, authorizes each
  connection before decoding its bounded frame, and delegates valid open
  requests to the retained-session controller.
- Changed the ordinary CLI to call `lens::open`, print its acknowledged
  loopback URL, and return. The existing `lens::serve(MarkdownTarget)` public
  path remains foreground-compatible; the service runner is hidden from normal
  help.
- Expanded the client tests across automatic startup, concurrent first
  commands, verified stale-socket recovery, browser failure with a still-live
  URL, and target rejection with zero browser attempts.
- Updated compiled CLI fixtures to own an explicit background service, so
  rejection checks cannot leak detached processes. Updated the browser fixture
  to wait for the ordinary command to exit before Chromium visits the URL; all
  browser behaviors, including automatic refresh after a save, therefore run
  against a session retained only by the service.
- Manually ran the compiled ordinary client with no endpoint, a private runtime
  directory, and a controlled browser command. The command exited, the returned
  URL responded successfully, and the test-owned background process was then
  stopped and its endpoint removed.
- Verification passed:

  - `cargo fmt --check`
  - `cargo test --locked` - 103 library, 5 CLI, and 0 documentation tests
  - `cargo clippy --locked --all-targets --all-features -- -D warnings`
  - `npm run test:browser` - 26 compiled-browser scenarios
  - `git diff --check`
- An isolated crate compiled the canonical detached-process module for
  `x86_64-pc-windows-msvc` and `x86_64-apple-darwin`. A full Windows-target
  check on this Linux host again stopped in the `ring` dependency before Lens
  code because the MSVC `lib.exe` tool is unavailable; native full-repository
  checks remain a C16 transition gate.

Artifact Outcomes:

- Created [`src/service/client.rs`](../../src/service/client.rs) and
  [`src/service/process.rs`](../../src/service/process.rs) as the client and
  detached-process capability owners planned by `RZ-04`.
- Refined [`src/service/server.rs`](../../src/service/server.rs) with endpoint
  ownership and one-frame connection handling while preserving the C14 actor
  and request ledger.
- Refined [`src/main.rs`](../../src/main.rs) and [`src/lib.rs`](../../src/lib.rs)
  with the ordinary background-open path and hidden internal service entry.
- Refined [`tests/cli.rs`](../../tests/cli.rs) and
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) with
  process-lifecycle-aware fixtures.
- Added `getrandom` as a direct dependency because request identity is now a
  Lens responsibility.
- No operator guidance or lifecycle-status changes were added; measured
  budgets, native transition evidence, README/release guidance, and final
  feature status remain the C16 transition scope.
