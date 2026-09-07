---
type: "Iteration Record"
title: "Iteration: C18 Supported Intel macOS Runner"
description: "Restores executable macOS pull-request verification by moving the existing x86-64 target from a retired runner label to GitHub's supported Intel label."
id: "C18"
phase: "transition"
status: "completed"
tags: [iteration, transition, macos, continuous-integration]
---

# Iteration: C18 Supported Intel macOS Runner

Status: completed

Phase Intent:

- Remove the final verification-infrastructure blocker after the C17 Windows
  correction and obtain full native pull-request evidence.

Goal:

- Run the existing `x86_64-apple-darwin` tests, strict Clippy, and package
  verification on a currently supported GitHub-hosted Intel macOS runner.

Risks Addressed:

- `R-12`: a required platform check that never receives a runner cannot provide
  evidence for the native endpoint and isolated-session implementation.

Artifact Budget:

- create: `docs/iterations/c18-supported-intel-macos-runner.md` - preserve the
  post-publication runner failure, correction, and one-commit integration trace
  independently from C17's Windows code correction.
- keep with implementation: supported runner label and unchanged Rust target -
  the workflow matrix is their authoritative owner.
- update: `docs/risk-list.md` - extend the native delivery trace through the
  runner-availability integration iteration.
- omit: requirements, design, and user guidance - runner selection does not
  change Lens behavior, platform scope, or service collaboration.

Artifacts to Start:

- This C18 integration record - preserve the queued-run evidence, supported
  label decision, validation, and external rerun boundary.

Artifacts to Refine:

- Native pull-request matrix: [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml)
- Current risk trace: [`docs/risk-list.md`](../risk-list.md)

Artifacts Consulted:

- C16 native verification gate:
  [`c16-background-service-transition.md`](c16-background-service-transition.md)
- C17 native Windows integration:
  [`c17-native-windows-integration.md`](c17-native-windows-integration.md)
- [GitHub-hosted runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
- [GitHub macOS 15 Intel availability announcement](https://github.com/actions/runner-images/issues/13045)

Decisions to Record:

- Replace retired `macos-13` with the supported standard
  `macos-15-intel` label while retaining `x86_64-apple-darwin` as the tested
  Rust target.

Trace:

- `R-12` -> C16 native pull-request matrix -> two zero-step macOS queues ->
  C18 supported Intel runner

Verification Evidence:

- Oracle: GitHub's hosted-runner reference lists `macos-15-intel` as a standard
  Intel label, and its runner-image announcement directs x86-64 users from
  deprecated macOS 13 to that label.
- Discrimination: both pull-request runs `30990416784` and `30996145395`
  retained the `x86_64-apple-darwin` job in the queued state with no setup or
  command steps. In the second run, native Linux, native Windows, and compiled
  browser jobs all completed successfully.
- Scope: runner routing is the only failed boundary. The Rust target, toolchain,
  commands, matrix fail-fast policy, and other runner labels remain unchanged.

Exit Criteria:

- The macOS matrix entry uses GitHub's supported standard Intel label and
  retains the x86-64 Apple target.
- Workflow YAML, the exact runner-to-target mapping, local Markdown links,
  repository checks, and the staged diff pass.
- C18 is represented by exactly one completed iteration record and one commit.
- The pushed commit triggers a fresh matrix; its external result is reported
  separately without being claimed as pre-commit evidence.

Results:

- Replaced only the retired `macos-13` runner label with
  `macos-15-intel`. The matrix still installs Rust 1.75 for
  `x86_64-apple-darwin` and runs the same locked tests, strict Clippy, and
  package verification.
- Parsed the workflow as YAML and mechanically asserted that the supported
  Intel label maps to the Apple x86-64 target and that no `macos-13` entry
  remains.
- Local verification passed:

  - `cargo fmt --check`
  - `cargo test --locked` - 105 library, 5 CLI, and 0 documentation tests
  - `cargo clippy --locked --all-targets --all-features -- -D warnings`
  - `cargo package --allow-dirty --locked`
  - `npm run test:browser` - 26 compiled-browser scenarios
  - local Markdown links and `git diff --check`
- The pull-request matrix remains external evidence until C18 is committed and
  pushed. No PlantUML block changed, so configured-server diagram validation
  was not applicable.

Artifact Outcomes:

- created: C18 integration record - preserves the runner-routing failure,
  supported-label basis, pre-commit evidence, and external rerun boundary.
- refined: native pull-request matrix - routes the unchanged Intel macOS Rust
  target to GitHub's supported standard Intel runner.
- refined: `R-12` history - traces the native verification delivery through
  the final runner-availability correction.
- consulted: C16 and C17 transition evidence plus GitHub's current runner
  reference - no product design, platform scope, or user guidance changed.
