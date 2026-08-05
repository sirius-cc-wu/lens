---
type: "Iteration Record"
title: "Iteration: S1 Background Viewer Service Scope"
description: "Establishes the user-visible process-lifetime behavior, isolation boundary, and elaboration risks for non-blocking Lens commands."
id: "S1"
phase: "inception"
status: "completed"
tags: [iteration, process-lifecycle]
---

# Iteration: S1 Background Viewer Service Scope

Status: completed

Phase Intent:

- Establish the selected post-V1 feature's value, behavioral boundary, and
  architectural risks before choosing a command-channel or process-lifecycle
  mechanism.

Goal:

- Let a developer open several Lens targets from one shell without retaining a
  foreground process for every browser view, while preserving isolated viewing
  sessions.

Risks Addressed:

- `R-06`: desktop process and browser-launch behavior varies across supported
  operating systems.
- `R-11`: automatic startup and concurrent commands could create competing
  background processes, hang on stale discovery state, or report success for a
  lost request.
- `R-12`: a long-lived process could merge authorized resources or
  configuration across users, repositories, or target scopes.

Artifact Budget:

- create: `docs/features/background-viewer-service/use-cases.md` - product
  owner, S2 analysis, and acceptance tests need one canonical statement of the
  selected workflow; existing `FEAT-01` owns document viewing rather than
  command and process lifetime, and the new feature evolves independently.
- create: `docs/iterations/s1-background-viewer-service-scope.md` - future
  contributors need the historical iteration objective, artifact choices, and
  exit evidence without turning the feature artifact into a work log.
- update: `docs/supplementary-specification.md` - owns cross-cutting runtime,
  resilience, and security constraints.
- update: `docs/glossary.md` - owns shared meanings for the command-side client,
  background Lens process, and viewing session.
- update: `docs/risk-list.md` - owns ranked uncertainty and mitigation evidence.
- update: `docs/index.md` - owns navigation to current durable artifacts.
- defer: SSD, operation contract, architecture decision, realization, and
  design class diagram - S1 first fixes observable scope; S2 and S3 will create
  only the significant analysis and design artifacts justified by the selected
  startup, isolation, and acknowledgment risks.
- omit: standalone domain model - the feature introduces technical lifecycle
  responsibilities rather than independently maintained business concepts;
  the glossary and later design are sufficient owners.

Artifacts to Start:

- `FEAT-04`, background viewer service use cases:
  [`docs/features/background-viewer-service/use-cases.md`](../features/background-viewer-service/use-cases.md) -
  define the foreground-terminal problem, ordinary command workflow, failure
  behavior, isolation rule, and deliberately unspecified idle shutdown policy.
- This S1 iteration record - preserve the inception decision and elaboration
  handoff.

Artifacts to Refine:

- Cross-cutting requirements:
  [`docs/supplementary-specification.md`](../supplementary-specification.md) -
  add short-lived command, bounded acknowledgment, per-user locality, and
  per-session isolation constraints.
- Shared terms: [`docs/glossary.md`](../glossary.md) - distinguish the Lens
  client, background Lens service, and viewing session.
- Risk list: [`docs/risk-list.md`](../risk-list.md) - add startup coordination
  and cross-session isolation risks.
- Documentation index: [`docs/index.md`](../index.md) - expose `FEAT-04` as a
  current artifact.

Artifacts Consulted:

- `FEAT-01`, current viewing use cases:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md)
- `ADR-002`, fixed loopback viewing-session scope:
  [`docs/decisions/adr-002-loopback-viewer-scope.md`](../decisions/adr-002-loopback-viewer-scope.md)
- `ADR-017`, session-fixed PlantUML server:
  [`docs/decisions/adr-017-session-plantuml-server.md`](../decisions/adr-017-session-plantuml-server.md)
- Current CLI and viewer composition:
  [`src/main.rs`](../../src/main.rs) and [`src/viewer/mod.rs`](../../src/viewer/mod.rs)

Decisions to Record:

- Treat one background process per operating-system user as the reusable
  product capability while keeping each viewing session's authorization and
  configuration independent.
- Require ordinary `lens` invocations to start or reuse that capability and
  return after bounded acknowledgment and browser handoff, without a separate
  server-start command.
- Leave the exact idle shutdown policy open because it does not change the
  confirmed user outcome.

Trace:

- Confirmed user intent -> `FEAT-04` (`UC-11`) -> S2 `SSD-07` / `OC-07` -> S3
  `RZ-04` / `DCD-04` -> command, process-coordination, and browser acceptance
  tests

Exit Criteria:

- The feature explains the terminal-ownership problem and intended result
  before introducing lifecycle terminology.
- `UC-11` remains a black-box actor-goal scenario and distinguishes target,
  startup, delivery, browser-launch, and browser-time failures.
- Requirements preserve current root, document-set, target-scope, source-link,
  and PlantUML configuration boundaries across repositories.
- Startup coordination, stale discovery, acknowledgment, cross-user access,
  and cross-session state are explicit elaboration risks.
- S2 and S3 have bounded analysis and design questions without committing to a
  mechanism in inception.

Results:

- `FEAT-04` now defines a normal `lens` command as a short-lived request to one
  automatically available background Lens process for the current user.
- `UC-11` requires a browser view for every accepted command, prompt return
  after acknowledgment and browser handoff, actionable failures, and no manual
  cleanup or start command after stale process state.
- The feature preserves one authorization and configuration boundary per
  viewing session even when one process hosts several repositories.
- S2 will specify the command-to-Lens system events and exact state effects;
  S3 will select the cross-platform coordination mechanism and Rust
  responsibilities.
- No PlantUML block changed in this iteration, so diagram validation was not
  required.
- Verification passed: `git diff --check`, `cargo fmt --check`,
  `cargo test --locked` (77 library tests and five CLI tests), and
  `cargo clippy --locked --all-targets --all-features -- -D warnings`.

Artifact Outcomes:

- started: `FEAT-04`, open documents through a background Lens service:
  [`docs/features/background-viewer-service/use-cases.md`](../features/background-viewer-service/use-cases.md) -
  owns `UC-11`, its extensions, constraints, and open elaboration questions.
- started: S1 background viewer service scope - records the selected feature
  boundary and artifact budget.
- refined: supplementary specification, glossary, risk list, and documentation
  index - establish cross-cutting constraints, shared terms, ranked risks, and
  navigation.
