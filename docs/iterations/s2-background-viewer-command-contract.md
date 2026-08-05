---
type: "Iteration Record"
title: "Iteration: S2 Background Viewer Command Contract"
description: "Specifies the actor-system sequence and exact state effects for one acknowledged non-blocking Lens command."
id: "S2"
phase: "elaboration"
status: "completed"
tags: [iteration, process-lifecycle]
---

# Iteration: S2 Background Viewer Command Contract

Status: completed

Phase Intent:

- Turn the S1 feature boundary into precise system events and testable state
  effects before assigning internal process and object responsibilities.

Goal:

- Specify when `lens <target>` may return success, which invocation context
  crosses the system boundary, and how one background process hosts each new
  viewing session without changing existing sessions.

Risks Addressed:

- `R-11`: distinguish successful acknowledgment from target, startup,
  delivery, timeout, and stale-state failures, including transport retry.
- `R-12`: make each accepted request's root, document set, scope, source-link
  authority, and PlantUML configuration independent.
- `R-06`: keep the browser handoff and manual URL observable at the command
  boundary without requiring the background process to retain a terminal.

Artifact Budget:

- create: `SSD-07` - `OC-07`, S3 controller design, and browser acceptance
  scenarios need one stable black-box sequence; the command/background split
  makes event ordering independently useful beyond the prose use case.
- create: `OC-07` - session creation, command completion, retry, and failure
  effects are too stateful and security-sensitive to leave implicit in
  `UC-11`; S3 and construction tests will consume the contract directly.
- create: `docs/iterations/s2-background-viewer-command-contract.md` - preserve
  this elaboration objective, artifact decisions, and validation evidence
  separately from the evolving analysis artifacts.
- update: `FEAT-04` - replace planned analysis links with the canonical S2
  artifacts while retaining product-level behavior.
- update: `docs/risk-list.md` - record the state and failure rules now available
  to S3.
- defer: architecture decision, use-case realization, design class diagram, and
  Rust adaptation - S3 will choose and map the mechanism from the completed
  contract.
- omit: separate failure SSDs - target, communication, and browser-launch
  extensions change responses rather than introducing another actor-system
  operation; the significant responses fit beside `SSD-07`.

Artifacts to Start:

- `SSD-07`, request a target view:
  [`docs/features/background-viewer-service/ssd-07-request-target-view.md`](../features/background-viewer-service/ssd-07-request-target-view.md) -
  establish the black-box operation and response order.
- `OC-07`, request a target view:
  [`docs/features/background-viewer-service/oc-07-request-target-view.md`](../features/background-viewer-service/oc-07-request-target-view.md) -
  specify success, target failure, coordination failure, browser failure,
  isolation, and retry effects.
- This S2 iteration record - preserve the elaboration result and S3 handoff.

Artifacts to Refine:

- `FEAT-04`, background viewer service use cases:
  [`docs/features/background-viewer-service/use-cases.md`](../features/background-viewer-service/use-cases.md) -
  link the completed SSD and contract.
- Risk list: [`docs/risk-list.md`](../risk-list.md) - record the selected
  operation boundary and state guarantees as mitigation evidence.

Artifacts Consulted:

- `OC-02`, open a document root:
  [`docs/features/markdown-viewing/oc-02-open-document-root.md`](../features/markdown-viewing/oc-02-open-document-root.md)
- `ADR-002`, fixed loopback viewing-session resources:
  [`docs/decisions/adr-002-loopback-viewer-scope.md`](../decisions/adr-002-loopback-viewer-scope.md)
- `ADR-017`, session-fixed PlantUML server:
  [`docs/decisions/adr-017-session-plantuml-server.md`](../decisions/adr-017-session-plantuml-server.md)
- Current browser launch and viewer composition:
  [`src/viewer/browser.rs`](../../src/viewer/browser.rs) and
  [`src/viewer/mod.rs`](../../src/viewer/mod.rs)

Decisions to Record:

- Name the system operation
  `request_target_view(target?, invocation_directory, scope, plantuml_server?)`
  so relative paths, omitted targets, scope, and environment configuration
  retain the invoking command's meaning.
- Create one new viewing session for every accepted command while reusing the
  background process. Existing sessions remain immutable authorization and
  configuration contexts.
- Acknowledge only after the new session's loopback URL is ready; keep browser
  launch and manual URL guidance within the command completion boundary.
- Treat an internal delivery retry as the same request and a new CLI invocation
  as a deliberate new request.

Trace:

- `FEAT-04` (`UC-11`) -> `SSD-07` -> `OC-07` -> S3 `ADR-022` / `RZ-04` /
  `DCD-04` -> startup, retry, isolation, CLI, and browser acceptance tests

Exit Criteria:

- `SSD-07` represents Lens as one black box and includes only data obtained from
  the invocation or external actor interactions.
- The operation preserves relative and omitted target meaning despite the
  long-lived process having a different working directory.
- `OC-07` defines testable success, target failure, coordination failure, and
  browser-handoff effects without prescribing controllers or transport APIs.
- Session creation is exact: every accepted command gets a new fixed document
  and configuration boundary, while all pre-existing sessions remain
  unchanged.
- Transport retry and separately invoked command semantics are distinct enough
  to assign idempotency responsibilities in S3.
- Every added PlantUML block validates through the configured PlantUML server.

Results:

- `SSD-07` discovers one significant operation,
  `request_target_view(target?, invocation_directory, scope,
  plantuml_server?)`, followed by the existing browser document request.
- `OC-07` requires the URL to be ready before acknowledgment, keeps browser
  handoff failure recoverable through the manual URL, and forbids an
  acknowledgment timeout from becoming false success.
- Each accepted command creates a new viewing session so document discovery and
  PlantUML selection retain current per-invocation semantics. The background
  process, not the session state, is the reused resource.
- S3 can now choose coordination and request-identity mechanisms against
  explicit startup, retry, user-isolation, and session-isolation oracles.
- The `SSD-07` PlantUML block rendered successfully through the configured
  PlantUML server (HTTP 200, `image/svg+xml`).
- Verification passed: `git diff --check`, `cargo fmt --check`,
  `cargo test --locked` (77 library tests and five CLI tests), and
  `cargo clippy --locked --all-targets --all-features -- -D warnings`.

Artifact Outcomes:

- started: `SSD-07`, request a target view - owns the black-box interaction and
  discovered system operation.
- started: `OC-07`, request a target view - owns exact state and failure
  effects for one command.
- started: S2 background viewer command contract - records the completed
  elaboration slice and design handoff.
- refined: `FEAT-04` and risk list - link the canonical analysis and record its
  mitigation value.
