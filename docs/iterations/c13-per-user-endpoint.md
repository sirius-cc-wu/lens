---
type: "Iteration Record"
title: "Iteration: C13 Per-User Endpoint"
description: "Claims one authenticated local command endpoint per operating-system user and recovers verified stale Unix sockets."
id: "C13"
phase: "construction"
status: "completed"
tags: [iteration, ipc, security, concurrency]
---

# Iteration: C13 Per-User Endpoint

Status: completed

Phase Intent:

- Resolve service ownership and cross-user authorization before any target can
  cross the local process boundary.

Goal:

- Provide one atomically claimed per-user Unix socket or Windows named pipe,
  authorize only the current user, and recover stale Unix sockets without ever
  deleting an unverified filesystem entry.

Risks Addressed:

- `R-11`: concurrent first commands or stale state could create competing
  services or require manual cleanup.
- `R-12`: loopback locality alone cannot prevent another operating-system user
  from submitting filesystem targets.

Artifact Budget:

- create: `docs/iterations/c13-per-user-endpoint.md` - preserve the security
  and ownership slice, platform evidence, and iteration-to-commit mapping.
- keep with implementation: endpoint paths, permissions, access-control list,
  peer authorization, stale cleanup, and owner-election tests - the platform
  code is their authoritative owner.
- update: platform dependencies and Tokio network capability - make direct
  native API use explicit.
- omit: new architecture decision - ADR-022 already selects the platform
  mechanisms and this iteration implements that accepted decision.

Artifacts to Start:

- This C13 iteration record - capture endpoint construction and evidence.
- Endpoint capability:
  [`src/service/endpoint.rs`](../../src/service/endpoint.rs)

Artifacts to Refine:

- Service module root and dependency declarations:
  [`src/service/mod.rs`](../../src/service/mod.rs),
  [`Cargo.toml`](../../Cargo.toml), and [`Cargo.lock`](../../Cargo.lock)
- `BACKGROUND-VIEWER-DESIGN`, construction handoff:
  [`docs/features/background-viewer-service/design.md`](../features/background-viewer-service/design.md) -
  keep detached process startup with the C15 end-to-end auto-start slice where
  its observable outcome can be tested.

Artifacts Consulted:

- `ADR-022`, per-user endpoint and owner election:
  [`docs/decisions/adr-022-per-user-background-service.md`](../decisions/adr-022-per-user-background-service.md)
- `BACKGROUND-VIEWER-DESIGN`, endpoint responsibility:
  [`docs/features/background-viewer-service/design.md`](../features/background-viewer-service/design.md)

Decisions to Record:

- Unix uses a verified private runtime directory and an owned socket inode;
  Windows names the pipe by current-user SID and applies an explicit protected
  access-control list for that SID and LocalSystem.

Trace:

- `UC-11` -> `OC-07` startup failure and isolation -> `ADR-022` endpoint -> C13 tests

Test-Driven Evidence:

- Oracle: ADR-022 requires atomic ownership, current-user authorization, and
  stale Unix cleanup only after failed connection plus owned-socket inspection.
- Slice size: endpoint location, ownership, authorization, and cleanup are one
  security boundary; protocol handling and session creation remain later slices.
- Discrimination: with the endpoint types and `claim_at` seam present but
  returning `EndpointError::NotImplemented`, `cargo test
  unclaimed_private_endpoint_then_listener_becomes_owner -- --nocapture` failed
  because no listener became owner. The failure isolated missing ownership from
  fixture setup or platform compilation.

Exit Criteria:

- Exactly one contender claims an unowned endpoint; later contenders receive a
  typed already-owned result.
- Unix runtime directories and sockets are verified as owner-only; peers are
  checked by effective user ID.
- A failed connection permits removal only of a socket owned by the current
  user, with non-sockets and foreign-owned entries preserved and rejected.
- Windows first-instance ownership uses a current-user-and-LocalSystem protected
  access-control list and a SID-specific name.
- Native Linux tests plus macOS and Windows compile checks, formatting, locked
  Rust tests, Clippy, and diff checks pass.

Results:

- Added a verified Unix runtime directory: Lens creates an owner-only fallback
  when necessary and rejects non-directories, foreign ownership, or any group
  or other access. Claimed sockets are mode `0600`.
- Added atomic Unix listener binding, effective-user peer credentials, and
  inode-preserving cleanup. A stale socket is removed only after failed connect,
  owner/socket checks, and a second device/inode check; a regular file is
  preserved and rejected.
- Added a SID-specific Windows named-pipe name, first-instance ownership,
  remote-client rejection, and a protected access-control list granting full
  access only to the current user SID and LocalSystem. Each accepted connection
  creates the next explicitly protected pipe instance.
- Kept the common endpoint error and re-export surface in `endpoint.rs`, with
  Unix and Windows implementations beside it in dedicated platform modules so
  their native dependencies, lifecycle code, and tests can change independently.
- Kept detached candidate process startup in C15, where the internal service
  mode and observable reconnect outcome can be tested together; refined the
  construction handoff accordingly.
- Seven native Linux endpoint checks passed, including concurrent owner
  election, stale recovery, negative path and peer authorization, private
  permissions, and same-user acceptance.
- The canonical endpoint source and its platform test modules passed isolated
  `cargo check --target x86_64-pc-windows-msvc --tests` and
  `cargo check --target x86_64-apple-darwin --tests` checks from a temporary
  minimal crate with the same pinned Tokio and native dependencies. A full
  repository Windows cross-check was attempted but stopped in `ring` before
  Lens compiled because this Linux host lacks MSVC `lib.exe`; native package
  execution remains C16 evidence.
- Full local verification passed: `cargo fmt --check`, `cargo test --locked`
  (93 library and five CLI tests), `cargo clippy --locked --all-targets
  --all-features -- -D warnings`, and `git diff --check`.
- No PlantUML block changed, so diagram validation was not applicable.

Artifact Outcomes:

- started and completed: C13 per-user endpoint - records ownership,
  authorization, stale cleanup, platform compilation, and residual native risk.
- started: endpoint capability - owns Unix socket and Windows named-pipe
  location, authorization, lifecycle, platform APIs, and focused checks.
- refined: dependency declarations and service root - expose the pinned native
  APIs and Tokio network capability used directly.
- refined: `BACKGROUND-VIEWER-DESIGN` construction handoff - detached process
  startup moves to the end-to-end auto-start slice without changing ADR-022.
