---
type: "Iteration Record"
title: "Iteration: C12 Bounded Service Protocol"
description: "Introduces a versioned, size-bounded local command protocol with lossless native paths and typed outcomes."
id: "C12"
phase: "construction"
status: "completed"
tags: [iteration, protocol, security]
---

# Iteration: C12 Bounded Service Protocol

Status: completed

Phase Intent:

- Establish a safe process boundary before endpoint ownership and concurrent
  service behavior begin depending on it.

Goal:

- Encode one versioned request or response as bounded length-prefixed JSON,
  preserve platform-native paths without lossy text conversion, and reject
  malformed, oversized, incompatible, or wrong-platform input with typed errors.

Risks Addressed:

- `R-11`: an unbounded or ambiguous stream could hang a command or allocate
  attacker-declared memory before validating a request.
- `R-12`: lossy or cross-platform path decoding could resolve a different
  filesystem target from the one submitted by the invoking client.

Artifact Budget:

- create: `docs/iterations/c12-bounded-service-protocol.md` - preserve the
  protocol slice, security oracle, discrimination evidence, and commit mapping.
- keep with implementation: frame limit, schema, native conversion, and
  executable examples - `src/service/protocol.rs` is their authoritative owner.
- update: Cargo dependency declarations - make directly used serialization and
  asynchronous I/O capabilities explicit.
- omit: separate wire-schema document - the protocol is crate-local and the
  typed Rust schema plus tests are the maintained source of truth.

Artifacts to Start:

- This C12 iteration record - capture protocol construction and evidence.
- Service capability root and protocol implementation:
  [`src/service/mod.rs`](../../src/service/mod.rs) and
  [`src/service/protocol.rs`](../../src/service/protocol.rs)

Artifacts to Refine:

- Target scope serialization:
  [`src/target.rs`](../../src/target.rs)
- Dependency manifest and lockfile:
  [`Cargo.toml`](../../Cargo.toml) and [`Cargo.lock`](../../Cargo.lock)

Artifacts Consulted:

- `OC-07`, retry identity and failure effects:
  [`docs/features/background-viewer-service/oc-07-request-target-view.md`](../features/background-viewer-service/oc-07-request-target-view.md)
- `ADR-022`, protocol and path decision:
  [`docs/decisions/adr-022-per-user-background-service.md`](../decisions/adr-022-per-user-background-service.md)

Decisions to Record:

- Set the first frame maximum to 64 KiB; the fixed request fields and local
  error messages require far less, leaving headroom without permitting
  request-sized allocation growth.

Trace:

- `UC-11` -> `OC-07` -> `ADR-022` bounded idempotent messages -> C12 protocol tests

Test-Driven Evidence:

- Oracle: ADR-022 requires versioned, length-prefixed, size-bounded messages,
  lossless native paths, a request identifier, and typed closed outcomes.
- Slice size: framing, schema, and native conversion change together because
  endpoint and controller iterations need one validated message boundary.
- Discrimination: with typed messages and the stable framing functions present
  but returning `ProtocolError::NotImplemented`, `cargo test
  framed_request_then_preserves_version_scope_and_native_paths -- --nocapture`
  failed at `test frame should encode`. The failure distinguished missing frame
  transfer from schema construction or path conversion.

Exit Criteria:

- A 32-bit big-endian prefix is checked against 64 KiB before payload allocation.
- Requests preserve version, identifier, scope, PlantUML value, and lossless
  current-platform paths through an in-memory stream round trip.
- Malformed JSON, incompatible versions, null path units, and wrong-platform
  path variants are rejected before target resolution.
- Ready, rejected, and incompatible responses remain distinct typed outcomes.
- Formatting, locked Rust tests, Clippy, and diff checks pass.

Results:

- Added directly pinned `serde` and `serde_json` dependencies and the Tokio
  in-memory I/O capability; the lockfile retains the already selected versions.
- Added a 64 KiB frame maximum, four-byte big-endian prefix, complete reads and
  writes, and size checks before payload allocation or transfer.
- Added versioned open requests, 128-bit request identifiers, target scope and
  PlantUML fields, and distinct ready, rejected, and incompatible responses.
- Added lossless Unix-byte and Windows-wide path variants. Current-platform
  conversion rejects null units and the other platform's encoding before target
  discovery.
- Seven focused protocol checks passed for round trips and every specified
  rejection class.
- Full verification passed: `cargo fmt --check`, `cargo test --locked` (86
  library and five CLI tests), `cargo clippy --locked --all-targets
  --all-features -- -D warnings`, and `git diff --check`.
- No PlantUML block changed, so diagram validation was not applicable.

Artifact Outcomes:

- started and completed: C12 bounded service protocol - records the security
  boundary, 64 KiB decision, discrimination evidence, and passing checks.
- started: service capability root and protocol module - own the closed local
  message schema, framing, native paths, and focused tests.
- refined: target scope and dependency declarations - support explicit wire
  encoding through direct, pinned dependencies.
