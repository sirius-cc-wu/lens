---
type: "Iteration Record"
title: "Iteration: C14 Service Controller"
description: "Coordinates idempotent open requests and retains independently configured viewing sessions in one state-owning loop."
id: "C14"
phase: "construction"
status: "completed"
tags: [iteration, service, concurrency, isolation]
---

# Iteration: C14 Service Controller

Status: completed

Phase Intent:

- Implement the stateful center of the accepted design after protocol and
  endpoint boundaries are independently verified.

Goal:

- Give one asynchronous controller exclusive ownership of request outcomes and
  session handles, make retries idempotent, and prove separate requests retain
  separate roots and PlantUML destinations.

Risks Addressed:

- `R-11`: a transport retry could create duplicate sessions or lose an
  in-flight outcome.
- `R-09`: retained session and request counts must be explicit and measurable.
- `R-12`: process reuse could accidentally merge root or rendering authority
  between accepted commands.

Artifact Budget:

- create: `docs/iterations/c14-service-controller.md` - preserve the controller
  slice, executable isolation evidence, and iteration-to-commit mapping.
- keep with implementation: request ledger, actor loop, session factory, and
  focused tests - `src/service/server.rs` is their authoritative owner.
- update: target loading and PlantUML normalization seams - let the service use
  captured invocation state instead of its own working directory or environment.
- omit: separate data model - `DCD-04` already owns the request and session
  responsibilities and this iteration implements those types directly.

Artifacts to Start:

- This C14 iteration record - capture coordination construction and evidence.
- Service controller and ledger:
  [`src/service/server.rs`](../../src/service/server.rs)

Artifacts to Refine:

- Target and PlantUML configuration seams:
  [`src/target.rs`](../../src/target.rs) and
  [`src/plantuml.rs`](../../src/plantuml.rs)
- Service capability root:
  [`src/service/mod.rs`](../../src/service/mod.rs)

Artifacts Consulted:

- `OC-07`, idempotency, isolation, and rejection effects:
  [`docs/features/background-viewer-service/oc-07-request-target-view.md`](../features/background-viewer-service/oc-07-request-target-view.md)
- `RZ-04` and `DCD-04`, controller and ledger responsibilities:
  [`docs/features/background-viewer-service/design.md`](../features/background-viewer-service/design.md)

Decisions to Record:

- Keep request and session collections in one `mpsc`-driven controller. Session
  creation runs outside its state mutation and completes through the same loop.

Trace:

- `UC-11` -> `SSD-07` -> `OC-07` -> `RZ-04` / `DCD-04` -> C14 controller tests

Test-Driven Evidence:

- Oracle: OC-07 requires one session per new request, no extra session for a
  transport retry, unchanged existing sessions, and no reachable URL on target
  rejection.
- Slice size: request identity, retained outcome, session factory, and
  cross-session isolation form the smallest meaningful state-owning behavior.
- Discrimination: with the controller handle and stable `open` boundary present
  but returning `ControllerError::NotImplemented`, `cargo test --locked
  same_request_retried_then_one_viewing_session_is_retained -- --nocapture`
  failed on the first request outcome. The test therefore detected absent
  coordination after request construction and before any session assertion.

Exit Criteria:

- Concurrent delivery of one request ID returns one ready URL and retains one
  request/session outcome.
- Different IDs receive distinct URLs whose documents, roots, and PlantUML
  servers remain independent.
- A target or native-path rejection is retained without a session or reachable URL.
- Target resolution uses the captured invocation directory for omitted and
  relative targets.
- Formatting, locked Rust tests, Clippy, and diff checks pass.

Results:

- Added one bounded `mpsc` controller loop with exclusive `RequestLedger`
  ownership. New requests insert in-flight waiters, session creation runs in a
  separate task, and completion returns through the controller before the
  response and optional session handle are retained.
- A completed response is cloned to retries and every in-flight waiter for the
  same 128-bit request ID. Controller statistics expose retained request and
  ready-session counts for executable checks and C16 measurement.
- Added service-side target creation from lossless paths, captured invocation
  directory, target scope, and captured PlantUML value. Target/path failures
  become typed rejections; viewer startup failures remain distinct session
  rejections.
- Added an explicit invocation-directory target loader without changing the
  existing public current-directory APIs, plus a focused relative-target check.
- Three controller checks passed: concurrent retry retains one ready session;
  separate requests serve distinct root contents and distinct controlled
  PlantUML SVGs; and target rejection retains one outcome with zero sessions.
- Full verification passed: `cargo fmt --check`, `cargo test --locked` (97
  library and five CLI tests), `cargo clippy --locked --all-targets
  --all-features -- -D warnings`, and `git diff --check`.
- No PlantUML block changed, so diagram validation was not applicable.

Artifact Outcomes:

- started and completed: C14 service controller - records idempotency,
  isolation, rejection, and verification evidence.
- started: service controller and request ledger - own all request state,
  retained outcomes, session handles, completion messages, and focused tests.
- refined: target and PlantUML seams - interpret a request entirely from the
  invoking client's captured context.
- consulted without change: `BACKGROUND-VIEWER-DESIGN` - the implementation
  follows `RZ-04` and `DCD-04` without a responsibility correction.
