---
type: "Iteration Record"
title: "Iteration: C16 Background Service Transition"
description: "Measures the reusable service, establishes transition budgets, and aligns native verification and user-facing lifecycle guidance with the implementation."
id: "C16"
phase: "transition"
status: "completed"
tags: [iteration, transition, measurement, process-lifecycle]
---

# Iteration: C16 Background Service Transition

Status: completed

Phase Intent:

- Reconcile the implemented command and service lifecycle with current user,
  release, risk, quality, and design artifacts; establish reference budgets;
  and make native platform evidence a pull-request gate.

Goal:

- Deliver the background viewer service as a reviewable cross-platform change
  whose command behavior, failure bounds, retained-session cost, residual
  risks, migration impact, and verification path are explicit.

Risks Addressed:

- `R-06`: detached service lifetime and browser handoff vary by operating
  system and desktop environment.
- `R-09`: process-lifetime session retention and polling accumulate memory and
  CPU work.
- `R-11`: a stalled service must produce a bounded, truthful client failure.
- `R-12`: native endpoint and session-isolation implementations must remain in
  the ordinary pull-request verification path.

Artifacts to Start:

- This C16 transition record - preserve measurement method, results, residual
  limits, verification, and iteration-to-commit trace.

Artifacts to Refine:

- User and compatibility guidance: [`README.md`](../../README.md) and
  [`docs/release-notes.md`](../release-notes.md)
- Quality, risk, and verification guidance:
  [`docs/supplementary-specification.md`](../supplementary-specification.md),
  [`docs/risk-list.md`](../risk-list.md), and
  [`docs/release-readiness.md`](../release-readiness.md)
- Current requirements and design:
  [`UC-11`](../features/background-viewer-service/use-cases.md),
  [`OC-07`](../features/background-viewer-service/oc-07-request-target-view.md),
  and [`RZ-04` / `DCD-04`](../features/background-viewer-service/design.md)
- Native verification and documentation entry points:
  [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml),
  [`Cargo.toml`](../../Cargo.toml), and [`docs/index.md`](../index.md)

Artifacts Consulted:

- C13 through C15 endpoint, controller, and automatic-handoff iteration records
- `BTE-01`, the compiled-browser suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs)

Decisions to Record:

- Keep the three-second service-startup and ten-second acknowledgment hard
  bounds. Investigate when 95% of optimized cold-start or reuse samples no
  longer complete within 250 ms; use that 95th-percentile value only as a
  reference-host threshold, not a cross-platform service-level guarantee.
- Keep process-lifetime session retention for this delivery. Investigate
  one-document growth above 256 KiB per additional session after first use or
  idle polling above 2% of one CPU at 50 sessions. Do not add a lease, browser
  close detector, idle shutdown, ledger compaction, or public stop command
  without separate lifecycle and large-repository evidence.
- Run locked tests, Clippy, and package verification on native x86-64 Linux,
  macOS, and Windows pull-request runners. Keep formatting and compiled-browser
  behavior on Linux.

Trace:

- `UC-11` -> `SSD-07` -> `OC-07` -> ADR-022 -> `RZ-04` / `DCD-04` -> C10
  through C15 construction -> C16 measured transition and native PR gates

Test-Driven Evidence:

- Oracle: `OC-07` requires a delivered request without a response to finish
  with an acknowledgment timeout and no reported ready URL.
- Slice size: parameterize only the internal exchange deadline so production
  retains ten seconds while a focused test can exercise the same timeout path
  in milliseconds.
- Discrimination:
  `service_accepts_frame_without_acknowledgment_then_client_times_out` first
  failed to compile because `exchange_with_timeout` did not exist.
- Green: the client exchange became generic over asynchronous readers and
  writers, production continued to pass the ten-second constant, and the test
  passed with a five-millisecond controlled deadline and a service that read
  the complete frame but returned no response.
- A separate production-deadline check verifies that a candidate which never
  claims the endpoint returns the typed startup timeout after the three-second
  bound and never attempts browser launch.

Measurement Method:

- Built Lens with `cargo build --release --locked` on x86-64 Linux.
- Used a private temporary runtime directory, one Markdown document, and a
  controlled `xdg-open` command that returned success without desktop work.
- Measured ten separate automatic starts, stopping the test-owned service after
  each; then measured 30 commands against one already available service.
- Started an explicit test-owned service and read its resident-memory field
  (`VmRSS`) before any session and after 1, 10, and 50 one-document sessions.
- Read user-plus-system CPU time from `/proc/<pid>/stat` across five idle
  seconds before sessions and after 50 sessions. This host reports 100 process
  accounting ticks per second.
- Replaced the service with a same-user Unix socket that accepted the complete
  request but produced no response, then timed the optimized ordinary command.
- Used the compiled-browser automatic-refresh scenario as the browser-visible
  check after the ordinary client had already exited.

Results:

- Cold start: 31 ms minimum, 32 ms median, 95% of samples within 32 ms, and
  32 ms maximum across ten samples.
- Service reuse: 4 ms minimum, 4 ms median, 95% of samples within 5 ms, and
  5 ms maximum across 30 samples.
- No acknowledgment: exit status 1 after 10,005 ms with
  `Lens background service did not acknowledge the request within ten seconds`.
- Retained resident memory (RSS): 5,264 KiB before a session, 7,048 KiB after
  1, 8,160 KiB after 10, and 12,804 KiB after 50. The complete increase
  averaged about 151 KiB per session; sessions 2 through 50 averaged about
  117 KiB each after first-use initialization.
- Idle polling: no measurable process CPU before a session and 7 accounting
  ticks, about 70 ms CPU or 1.4% of one core, over five seconds with 50
  one-document sessions.
- The compiled-browser suite's save scenario refreshed the displayed document
  after its ordinary client had exited. All 26 browser scenarios passed in C15.
- The canonical detached-process module compiled in isolated checks for
  `x86_64-apple-darwin` and `x86_64-pc-windows-msvc`; the endpoint module and
  native endpoint tests had the same isolated evidence in C13. A full Windows
  cross-check on this Linux host still stops in `ring` before Lens because the
  MSVC `lib.exe` tool is unavailable. The new native CI matrix owns full
  repository evidence on actual Linux, macOS, and Windows hosts.
- Final local verification passed:

  - `cargo fmt --check`
  - `cargo test --locked` - 105 library, 5 CLI, and 0 documentation tests
  - `cargo clippy --locked --all-targets --all-features -- -D warnings`
  - `cargo package --allow-dirty --locked`
  - `npm run test:browser` - 26 compiled-browser scenarios
  - CI workflow YAML parsing, local links in all 10 changed Markdown files,
    and `git diff --check`
- No PlantUML block changed, so configured-server diagram validation was not
  applicable.

Exit Criteria:

- README and pending release notes explain automatic startup, process reuse,
  isolated views, short-lived commands, error/browser behavior, crash impact,
  and the retained foreground Rust API.
- Hard failure bounds, reference timing thresholds, memory growth, polling
  work, and residual large-set/lifetime risk are current and traceable.
- The background design is marked implemented and current requirements link
  the construction and transition evidence.
- Pull requests require native Linux, macOS, and Windows Rust verification plus
  the compiled-browser process-lifecycle behavior.
- Formatting, locked Rust tests, Clippy, package verification, browser tests,
  documentation links, and the complete iteration diff pass before publication.

Artifact Outcomes:

- started: C16 transition record - preserves method, raw summarized results,
  thresholds, residuals, and validation gaps.
- refined: client timeout verification - mechanically tests the production
  acknowledgment-timeout path with a short controlled deadline.
- refined: README, release notes, and package description - present Lens as a
  cross-platform short-lived client backed by isolated background sessions.
- refined: supplementary specification, risks, and release readiness - replace
  pending construction thresholds with measured baselines and explicit review
  limits while retaining large-document-set work.
- implemented: background viewer service design - C10 through C16 now realize
  and transition ADR-022, `UC-11`, `OC-07`, `RZ-04`, and `DCD-04`.
- refined: native verification - makes platform-specific Rust behavior part of
  every pull request instead of tag-only packaging.
