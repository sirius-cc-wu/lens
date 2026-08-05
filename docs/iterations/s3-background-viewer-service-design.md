---
type: "Iteration Record"
title: "Iteration: S3 Background Viewer Service Design"
description: "Selects the per-user command channel and maps acknowledged target requests to cohesive Rust responsibilities and construction slices."
id: "S3"
phase: "elaboration"
status: "completed"
tags: [iteration, process-lifecycle, design]
---

# Iteration: S3 Background Viewer Service Design

Status: completed

Phase Intent:

- Resolve the startup, IPC security, retry, session ownership, and Rust mapping
  risks enough to begin test-driven construction in thin slices.

Goal:

- Design one automatically started per-user process that accepts bounded open
  requests, creates isolated browser-ready viewing sessions, and lets the
  invoking command retain browser and terminal responsibilities.

Risks Addressed:

- `R-06`: detached process and browser-launch behavior varies by operating
  system and desktop context.
- `R-09`: each accepted request retains another eagerly loaded and refreshed
  document set for the background process lifetime.
- `R-11`: concurrent startup, stale endpoints, retry, and crash recovery need
  one state owner and exact failure boundaries.
- `R-12`: the command endpoint and every hosted viewing session must retain the
  current user's, repository's, scope's, and PlantUML server's authority.

Artifact Budget:

- create: `ADR-022` - local IPC, client-side browser launch, one fresh session
  per request, and process ownership are cross-cutting, security-sensitive, and
  expensive to reverse; future maintainers need alternatives and consequences
  outside the implementation-facing design.
- create: `docs/features/background-viewer-service/design.md` - S3 and the
  planned construction iterations need one canonical owner for the realization,
  GRASP choices, class view, Rust mapping, module boundaries, and test handoff;
  these views evolve together for this cohesive feature.
- create: `docs/iterations/s3-background-viewer-service-design.md` - preserve
  the design objective, artifact budget, evidence, and residual risks without
  turning the canonical design into a historical log.
- update: `FEAT-04`, `OC-07`, risk list, and documentation index - link the
  selected design, close elaboration questions, record residual risk, and make
  the current artifacts navigable.
- embed: GRASP responsibility decisions, `RZ-04`, `DCD-04`, and Rust adaptation
  in the feature design - they share one consumer and lifecycle; separate files
  would fragment one implementation handoff.
- keep with implementation: concrete frame limits, startup deadlines, and
  acknowledgment performance budgets - executable tests and measurements are
  the authoritative owners once construction supplies evidence.
- omit: standalone domain model and pattern record - the glossary and contract
  already own the technical concepts, while compile-time platform modules and
  an actor-style controller require no additional pattern lifecycle.

Artifacts to Start:

- `ADR-022`, per-user background service:
  [`docs/decisions/adr-022-per-user-background-service.md`](../decisions/adr-022-per-user-background-service.md) -
  accept local IPC, automatic single-owner startup, client browser launch,
  bounded idempotent messages, and one loopback listener per request.
- `BACKGROUND-VIEWER-DESIGN`, feature design:
  [`docs/features/background-viewer-service/design.md`](../features/background-viewer-service/design.md) -
  own `RZ-04`, `DCD-04`, responsibility assignments, Rust semantics, module
  placement, and construction tests.
- This S3 iteration record - preserve the design result and construction
  handoff.

Artifacts to Refine:

- `FEAT-04`, background viewer use cases:
  [`docs/features/background-viewer-service/use-cases.md`](../features/background-viewer-service/use-cases.md) -
  link the selected design and decision.
- `OC-07`, request a target view:
  [`docs/features/background-viewer-service/oc-07-request-target-view.md`](../features/background-viewer-service/oc-07-request-target-view.md) -
  replace mechanism questions with selected outcomes and construction-owned
  thresholds.
- Risk list and documentation index:
  [`docs/risk-list.md`](../risk-list.md) and [`docs/index.md`](../index.md) -
  record selected mitigation, resource-lifetime residual risk, and navigation.

Artifacts Consulted:

- `FEAT-04`, `SSD-07`, and `OC-07`: the accepted behavior and state oracles.
- Current CLI, target, viewer, browser, and PlantUML composition:
  [`src/main.rs`](../../src/main.rs), [`src/target.rs`](../../src/target.rs),
  [`src/viewer/mod.rs`](../../src/viewer/mod.rs),
  [`src/viewer/browser.rs`](../../src/viewer/browser.rs), and
  [`src/plantuml.rs`](../../src/plantuml.rs)
- Pinned Tokio 1.35.1 source: Unix listener and peer credentials, Windows named
  pipe security attributes, and first-instance single-owner support.
- `ADR-002` and `ADR-017`: fixed resources and PlantUML destination per viewing
  session.

Decisions to Record:

- `ADR-022`: use authenticated per-user Unix-domain sockets or Windows named
  pipes instead of a TCP control port; elect one service through atomic endpoint
  ownership; and launch the browser from the client.
- Keep one fresh loopback viewer listener and `ViewerState` per accepted request
  inside the background process.
- Give `ServiceController` exclusive state ownership through an async message
  loop; retain sessions and their request outcomes without an
  `Arc<Mutex<ServiceState>>`.
- Preserve `lens::serve(MarkdownTarget)` as a foreground compatibility path and
  extract session startup beneath it.

Trace:

- `FEAT-04` (`UC-11`) -> `SSD-07` -> `OC-07` -> `ADR-022` -> `RZ-04` /
  `DCD-04` -> C10 through C16 construction and transition checks

Exit Criteria:

- The chosen endpoint establishes per-user access and single ownership on
  Linux, macOS, and Windows without making an HTTP filesystem operation.
- The realization satisfies every `OC-07` success and failure effect, including
  invocation-directory meaning, session isolation, idempotent retry, and manual
  browser URL behavior.
- Every non-trivial collaboration has a cohesive owner and GRASP rationale;
  the controller delegates target and session rules.
- Rust ownership, channels, task lifetime, error variants, platform variation,
  module placement, and public compatibility are explicit.
- Construction has thin test-first slices and native-platform verification
  targets rather than one all-at-once daemon change.
- Every added PlantUML block validates through the configured PlantUML server.

Results:

- ADR-022 accepts native local IPC with per-user authorization and atomic
  endpoint ownership. It rejects a fixed TCP control port and a global mutable
  HTTP session router.
- `RZ-04` assigns startup and browser launch to the client, request and session
  ownership to `ServiceController`, target rules to the existing `target`
  module, and browser-ready listener creation to `viewer::start_session`.
- `DCD-04` maps the design to owned Rust messages, closed enums, compile-time
  platform modules, an `mpsc`-driven controller, and a process-lifetime
  `RequestLedger` with RAII `ViewerSession` handles, without runtime trait
  objects or shared mutable service state.
- C10 through C16 separate mechanical browser promotion, behavior-preserving
  viewer extraction, the bounded protocol, native endpoint ownership, service
  coordination, CLI/browser acceptance, and transition measurement.
- Residual risks remain explicit for service crashes, incompatible live
  protocols, retained-session growth, concrete timeout budgets, and native
  Windows evidence.
- The `RZ-04` and `DCD-04` PlantUML blocks rendered successfully through the
  configured PlantUML server (HTTP 200, `image/svg+xml`).
- Verification passed: `git diff --check`, `cargo fmt --check`,
  `cargo test --locked` (77 library tests and five CLI tests), and
  `cargo clippy --locked --all-targets --all-features -- -D warnings`.

Artifact Outcomes:

- started: `ADR-022`, per-user background service - owns the cross-cutting IPC,
  process, browser, session, and compatibility choice.
- started: `BACKGROUND-VIEWER-DESIGN` with `RZ-04` and `DCD-04` - owns the
  implementation-facing collaboration, responsibilities, Rust mapping, and
  construction handoff.
- started: S3 background viewer service design - records the closed elaboration
  slice and residual risks.
- refined: `FEAT-04`, `OC-07`, risk list, and documentation index - link the
  accepted design and distinguish construction-owned evidence.
