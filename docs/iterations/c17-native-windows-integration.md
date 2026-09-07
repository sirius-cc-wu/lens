---
type: "Iteration Record"
title: "Iteration: C17 Native Windows Integration"
description: "Uses the first native Windows pull-request run to remove platform assumptions from CLI verification and endpoint errors."
id: "C17"
phase: "transition"
status: "completed"
tags: [iteration, transition, windows, continuous-integration]
---

# Iteration: C17 Native Windows Integration

Status: completed

Phase Intent:

- Turn evidence from the first native pull-request run into the smallest
  portability correction, then return the delivery gate to native verification.

Goal:

- Make the CLI help check accept the native executable name and keep Unix-only
  endpoint failures out of Windows builds without changing user-visible service
  behavior.

Risks Addressed:

- `R-12`: the newly required Windows job exposed assumptions that Linux-only
  local verification could not detect.

Artifact Budget:

- create: `docs/iterations/c17-native-windows-integration.md` - preserve the
  post-publication integration objective, its CI evidence, and its one-commit
  iteration mapping independently from the completed C16 transition record.
- keep with implementation: native executable-name expectation and endpoint
  error availability - the integration test and conditional Rust definitions
  are their authoritative owners.
- update: `docs/risk-list.md` - extend `R-12`'s delivery trace through the
  native integration correction.
- omit: design and requirements changes - the correction neither changes the
  service collaboration nor its observable contract.

Artifacts to Start:

- This C17 integration record - preserve the failing native evidence, focused
  correction, verification, and residual runner state.

Artifacts to Refine:

- Cross-platform CLI verification: [`tests/cli.rs`](../../tests/cli.rs)
- Platform endpoint error definitions:
  [`src/service/endpoint.rs`](../../src/service/endpoint.rs)
- Current risk trace: [`docs/risk-list.md`](../risk-list.md)

Artifacts Consulted:

- C16 native verification gate:
  [`c16-background-service-transition.md`](c16-background-service-transition.md)
- Windows endpoint implementation:
  [`src/service/endpoint/windows.rs`](../../src/service/endpoint/windows.rs)

Decisions to Record:

- Derive help-test expectations from Cargo's native executable path, including
  the `.exe` suffix on Windows.
- Compile endpoint failures only on the platform whose implementation can
  construct them.

Trace:

- `R-12` -> C16 native pull-request matrix -> Windows job failure -> C17
  portability correction

Test-Driven Evidence:

- Oracle: Cargo supplies the native binary path to integration tests, while
  Clap displays that binary's native file name in its usage line; Rust's
  warning-free Windows build must not define errors only Unix code constructs.
- Slice size: one expectation and three conditional variants directly account
  for all observed Lens-owned Windows diagnostics.
- Discrimination: the Windows native job passed 94 library tests and four CLI
  tests, then `help_flag_then_describes_optional_target_without_renderer_selection`
  rejected the native usage line because it expected the Unix-only name
  `lens`. The same build warned that `UnsafeRuntimeDirectory`, `UnsafeEndpoint`,
  and `UnauthorizedPeer` were never constructed on Windows, which would block
  the following Clippy step under `-D warnings`.

Exit Criteria:

- The help integration check derives the displayed command name from the
  platform-native executable path.
- Windows builds do not define Unix-only endpoint error variants.
- Formatting, locked tests, strict Clippy, package verification, and diff
  checks pass locally.
- C17 supplies warning-free Windows endpoint code for the updated pull-request
  matrix; the external rerun is reported separately without being claimed as
  pre-commit evidence.

Results:

- The help integration check now derives `lens` or `lens.exe` from Cargo's
  native executable path instead of embedding the Unix spelling.
- The three runtime-directory, stale-socket, and peer-user error variants, plus
  their `PathBuf` import, now compile only on Unix. The Windows named-pipe
  implementation continues to use the cross-platform ownership and I/O
  variants.
- An isolated crate compiled the canonical endpoint module for
  `x86_64-pc-windows-msvc` with warnings denied, then passed strict Clippy for
  that target. This directly checks the boundary without the full repository's
  unrelated MSVC `ring` archive-tool requirement on the Linux host.
- Local verification passed:

  - `cargo fmt --check`
  - focused cross-platform help integration test - 1 passed
  - `cargo test --locked` - 105 library, 5 CLI, and 0 documentation tests
  - `cargo clippy --locked --all-targets --all-features -- -D warnings`
  - `cargo package --allow-dirty --locked`
  - `npm run test:browser` - 26 compiled-browser scenarios
  - local Markdown links and `git diff --check`
- The pull-request rerun remains external evidence until C17 is committed and
  pushed. No PlantUML block changed, so configured-server diagram validation
  was not applicable.

Artifact Outcomes:

- created: C17 integration record - preserves the native failure, correction,
  pre-commit evidence, and boundary between local and pull-request validation.
- refined: CLI integration check - follows the native binary name reported by
  Cargo and rendered by Clap.
- refined: endpoint error definition - exposes only errors constructible by
  the selected platform implementation.
- refined: `R-12` history - traces the security-sensitive native endpoint
  delivery through C17.
- consulted: C16 transition gate and Windows named-pipe implementation - no
  design, requirements, release guidance, or lifecycle change was needed.
