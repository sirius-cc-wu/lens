---
type: "Iteration Record"
title: "Iteration: D4 Repository-Scoped Target Sessions"
description: "Implements and verifies repository-root discovery for directory and current-directory targets with an explicit target-scoped override."
id: "D4"
phase: "construction"
status: "completed"
tags: [iteration]
---

# Iteration: D4 Repository-Scoped Target Sessions

Status: completed

Phase Intent:

- Implement ADR-019 through the target loader, CLI, and compiled-browser
  boundary while retaining the fixed known-document authorization model.

Goal:

- Make repository-internal links work consistently from file, directory, and
  no-target invocations, with `--scope target` preserving deliberate narrow
  sessions.

Risks Addressed:

- `R-03`: repository broadening must stop at the nearest supported marker and
  target scope must not admit sibling repository documents.
- `R-09`: users must be able to avoid repository-wide local discovery and
  refresh work explicitly.
- Product usability: the selected directory must still determine the initial
  document after identifiers become repository-relative.

Artifacts to Start:

- None. D3 established the durable use case, contract, and ADR needed for this
  construction slice.

Artifacts to Refine:

- Target resolution and unit checks:
  [`src/target.rs`](../../src/target.rs) - separate repository discovery root
  selection from the file or directory initial-selection anchor.
- Public API and CLI:
  [`src/lib.rs`](../../src/lib.rs),
  [`src/main.rs`](../../src/main.rs), and
  [`tests/cli.rs`](../../tests/cli.rs) - expose `TargetScope`, default to
  repository scope, and document the accepted option values.
- `BTE-01`, browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - exercise
  directory, current-directory, and explicit target-scoped invocations through
  the compiled command.
- User and release guidance:
  [`README.md`](../../README.md) and
  [`docs/release-readiness.md`](../release-readiness.md) - explain the unified
  default, initial anchor, local-reading implications, and narrow override.
- Proposal outcome, decision trace, and risks:
  [`docs/decisions/adr-019-repository-scoped-target-sessions.md`](../decisions/adr-019-repository-scoped-target-sessions.md)
  and [`docs/risk-list.md`](../risk-list.md) - record implementation and
  executable authorization evidence.

Artifacts Consulted:

- `FEAT-01`, target and navigation requirements:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md)
- `OC-02`, root and initial-selection postconditions:
  [`docs/features/markdown-viewing/oc-02-open-document-root.md`](../features/markdown-viewing/oc-02-open-document-root.md)
- `ADR-019`, repository-scoped target sessions:
  [`docs/decisions/adr-019-repository-scoped-target-sessions.md`](../decisions/adr-019-repository-scoped-target-sessions.md)

Decisions to Record:

- None. D4 implements ADR-019 without changing its selected root, option, or
  fallback behavior.

Trace:

- Product feedback -> `FEAT-01` (`UC-02` through `UC-04`) -> `SSD-02` ->
  `OC-02` -> `ADR-019` -> target-loader examples, CLI help, and `BTE-01`

Test-Driven Evidence:

- Oracle: OC-02 requires nearest-repository discovery for file, directory, and
  current-directory targets; the selected target remains the initial-selection
  anchor; `--scope target` retains the former narrow root.
- Slice size: default directory and no-target navigation share one root and
  initial-selection change. The explicit target scope is a second cohesive
  compatibility and security slice.
- Discrimination: before implementation, the two new target-loader examples
  failed because a directory remained exact and an empty selected directory
  could not use the repository README. The compiled-directory scenario reached
  `/iterations/evidence.md` instead of the known
  `/documents/iterations/evidence.md` route.
- Negative control: temporarily broadening `TargetScope::Target` made all three
  focused target-scope examples fail, including the hidden-parent case. The
  correct non-broadening branch was restored before final verification.
- Green evidence: all 23 target-loader examples, the CLI help example, and the
  three new compiled-browser scenarios pass. Final validation passed the
  formatter, all 64 library and three CLI tests, warnings-as-errors Clippy,
  package verification, and all 21 browser scenarios.

Exit Criteria:

- Directory and current-directory targets inside a repository use its nearest
  supported root by default.
- The selected directory still chooses its README, `docs/index`, or first
  document before repository-level fallback.
- `--scope target` keeps directory and file-parent discovery narrow and remains
  usable below a hidden repository parent.
- Compiled-browser checks follow a cross-directory repository link in both
  directory and no-target sessions and retain guidance in target scope.
- User and release documentation explain the default and explicit boundary.
- Formatting, tests, Clippy, package verification, and browser checks pass.

Results:

- Added `TargetScope` with repository scope as the default and target scope as
  the explicit narrow choice.
- Applied nearest-repository discovery to file, directory, and
  current-directory targets while preserving non-repository fallbacks.
- Separated the document root from the initial-selection anchor so selected
  directories keep their local README, `docs/index`, and lexical preference.
- Added repository-level initial fallback for a selected directory containing
  no supported document.
- Retained hidden-entry rejection in repository scope and verified that target
  scope can open a visible directory below a hidden parent.
- Added unit, CLI, and compiled-browser evidence for the broadened default and
  narrow override.
- Reproduced the original `lens docs/iterations` command against the built
  binary: the feature link became
  `/documents/docs/features/automatic-refresh/use-cases.md` and returned HTTP
  200.
- Evaluated the expanded `src/target.rs` against the module-boundary guidance.
  Production code still has one target-resolution and discovery reason to
  change, and the added bulk is behavior-focused colocated tests, so no module
  split was warranted.
- Final validation passed: `cargo fmt --check`, `cargo test --locked` (64
  library tests and three CLI tests), `cargo clippy --locked --all-targets
  --all-features -- -D warnings`, `cargo package --allow-dirty`, and all 21
  `npm run test:browser` scenarios.

Artifact Outcomes:

- refined: target resolution, public API, and CLI - unify repository scope and
  retain an explicit target boundary.
- refined: `BTE-01` - verifies directory, current-directory, and target-scoped
  commands through the compiled executable.
- refined: README, release readiness, proposal, ADR trace, and risks - describe
  implemented behavior, broader local reads, and verification evidence.
