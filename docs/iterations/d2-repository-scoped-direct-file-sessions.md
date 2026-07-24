---
type: "Iteration Record"
title: "Iteration: D2 Repository-Scoped Direct-File Sessions"
description: "Implements and verifies nearest-repository discovery for directly opened Markdown and PlantUML files."
id: "D2"
phase: "construction"
status: "completed"
tags: [iteration]
---

# Iteration: D2 Repository-Scoped Direct-File Sessions

Status: completed

Phase Intent:

- Implement the D1 authorization decision through target-loader and
  compiled-browser behavior while preserving fixed known-document routing.

Goal:

- Open a repository document directly and follow its links across repository
  directories without admitting hidden, symbolic, or outside-repository
  documents.

Risks Addressed:

- `R-03`: root recognition and broader discovery must stop at the nearest
  supported repository boundary and retain guidance for outside paths.
- `R-09`: the broader local discovery scope must be visible in user and release
  documentation.

Artifacts to Start:

- None. D1 established the durable requirements, contract, and decision needed
  for this construction slice.

Artifacts to Refine:

- Target resolution and unit checks:
  [`src/target.rs`](../../src/target.rs) - select the nearest supported marker,
  retain parent fallback and explicit-directory scope, and reject a hidden
  selected path before discovery.
- `BTE-01`, browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - start the
  compiled command with a nested direct file, cross its parent through a known
  route, and reject an outside-repository link without source disclosure.
- User and release guidance:
  [`README.md`](../../README.md) and
  [`docs/release-readiness.md`](../release-readiness.md) - explain repository
  recognition, broader local discovery, explicit narrower scope, and
  acceptance checks.
- Proposal and risk list:
  [`docs/proposals/repository-scoped-direct-file-sessions.md`](../proposals/repository-scoped-direct-file-sessions.md)
  and [`docs/risk-list.md`](../risk-list.md) - record implementation and
  executable authorization evidence.

Artifacts Consulted:

- `FEAT-01`, direct-file and navigation requirements:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md)
- `OC-02`, root-selection postconditions:
  [`docs/features/markdown-viewing/oc-02-open-document-root.md`](../features/markdown-viewing/oc-02-open-document-root.md)
- `ADR-017`, repository-scoped direct-file sessions:
  [`docs/decisions/adr-017-repository-scoped-direct-file-sessions.md`](../decisions/adr-017-repository-scoped-direct-file-sessions.md)

Decisions to Record:

- None. D2 implements ADR-017 without changing its selected boundary or
  alternatives.

Trace:

- Proposal -> `FEAT-01` (`UC-02` through `UC-04`) -> `SSD-02` -> `OC-02` ->
  `ADR-017` -> target-loader examples and `BTE-01`

Test-Driven Evidence:

- Oracle: `OC-02` requires nearest supported `.git` marker selection, parent
  fallback outside a repository, unchanged explicit-directory scope,
  initial-file preservation, and hidden-path rejection. The proposal requires
  browser-visible navigation beyond the initial parent plus guidance without
  outside source disclosure.
- Slice size: the root-selection cases form one cohesive public
  `load_markdown_target` behavior, while two compiled-browser scenarios are the
  narrowest stable checks for link rewriting and outside-source protection.
- Discrimination: `cargo test --locked target::tests` failed five new examples
  while the parent fallback and explicit-directory examples passed. The
  repository-crossing browser scenario reached `/iterations/evidence.md`
  instead of `/documents/iterations/evidence.md`; the outside-repository
  scenario already passed through the existing guidance boundary.
- Green evidence: all 18 focused target-loader tests and both focused
  `direct_file_` browser scenarios pass after the target-resolution change.
  Final validation passed the formatter, all 59 library and three CLI tests,
  warnings-as-errors Clippy, package verification, and all 18 browser
  scenarios.

Exit Criteria:

- A direct `.md`, `.markdown`, or `.puml` file uses the nearest supported
  repository marker and remains the initial document.
- Regular `.git` directory and file markers are recognized without reading
  their contents; a symbolic marker is ignored.
- Nested repositories, no-repository fallback, explicit-directory scope, and
  hidden-path failure have focused unit evidence.
- A compiled-browser scenario follows a known document outside the initial
  parent, and another returns guidance without exposing an outside file.
- User and release documentation explain both the broader local discovery and
  the explicit narrower-directory option.
- Formatting, tests, Clippy, package verification, and the browser suite pass.

Results:

- Added nearest-ancestor repository recognition using only metadata for a
  non-symbolic-link `.git` directory or regular file. The selected file's
  canonical parent remains the fallback when no marker is recognized.
- Preserved the file as the initial document, left current-directory and
  explicit-directory branches unchanged, and reject a selected file whose
  repository-relative path crosses a hidden entry.
- Added target-loader evidence for `.md`, `.markdown`, and `.puml` files across
  ordinary repositories, worktrees, nested repositories, parent fallback,
  explicit-directory scope, symbolic `.git` markers, and hidden paths.
- Added compiled-browser evidence that rewrites a repository-crossing link to
  its known document route and returns guidance without outside source
  disclosure.
- Documented the wider local discovery scope and the explicit-directory option
  in user and release guidance.
- A simplification review found no duplicate configuration path or worthwhile
  extraction. The target loader remains the typed owner, and the explicit
  symbolic-link check remains visible as a security invariant.
- Final validation passed: `cargo fmt --check`, `cargo test --locked` (59
  library tests and three CLI tests), `cargo clippy --locked --all-targets
  --all-features -- -D warnings`, `cargo package --allow-dirty`, and all 18
  `npm run test:browser` scenarios.

Artifact Outcomes:

- refined: target resolution and unit checks - recognize the nearest supported
  repository root while preserving fallback and explicit scope.
- refined: `BTE-01` - verifies repository-crossing navigation and
  outside-source protection through the compiled command.
- refined: README, release readiness, proposal, and risk list - describe the
  implemented behavior, local scope increase, verification, and narrower
  directory option.
