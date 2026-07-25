---
type: "Iteration Record"
title: "Iteration: D3 Repository-Scoped Target Design"
description: "Broadens repository-root selection to directory and current-directory targets while preserving an explicit narrow scope."
id: "D3"
phase: "elaboration"
status: "completed"
tags: [iteration]
---

# Iteration: D3 Repository-Scoped Target Design

Status: completed

Phase Intent:

- Incorporate product feedback that target-type-specific scope is surprising,
  and stabilize the broader target contract before construction.

Goal:

- Make repository-internal links work consistently for file, directory, and
  no-target invocations without removing a deliberate narrow session mode.

Risks Addressed:

- `R-03`: broader directory and current-directory discovery must remain fixed
  at the nearest repository and preserve hidden, symbolic, and outside-root
  exclusions.
- `R-09`: repository scope can increase local discovery and refresh work, so a
  visible target-scoped override remains necessary.
- Product usability: `lens docs/iterations` should not reject valid links that
  work when an individual iteration file is selected.

Artifacts to Start:

- `ADR-019`, repository-scoped target sessions:
  [`docs/decisions/adr-019-repository-scoped-target-sessions.md`](../decisions/adr-019-repository-scoped-target-sessions.md) -
  records the unified default scope, explicit override, and initial-selection
  anchor.

Artifacts to Refine:

- Proposal outcome - broaden the accepted outcome from direct files to all
  target kinds.
- `FEAT-01`, `SSD-02`, and `OC-02`:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md),
  [`docs/features/markdown-viewing/ssd-02-open-document-root.md`](../features/markdown-viewing/ssd-02-open-document-root.md), and
  [`docs/features/markdown-viewing/oc-02-open-document-root.md`](../features/markdown-viewing/oc-02-open-document-root.md) -
  define observable root selection, the target scope option, and directory
  initial-selection behavior.
- `ADR-003` and `ADR-018`:
  [`docs/decisions/adr-003-document-root-discovery.md`](../decisions/adr-003-document-root-discovery.md) and
  [`docs/decisions/adr-018-repository-scoped-direct-file-sessions.md`](../decisions/adr-018-repository-scoped-direct-file-sessions.md) -
  preserve the historical decisions while pointing to their successor.
- Glossary, supplementary specification, risk list, and documentation index:
  [`docs/glossary.md`](../glossary.md),
  [`docs/supplementary-specification.md`](../supplementary-specification.md),
  [`docs/risk-list.md`](../risk-list.md), and
  [`docs/index.md`](../index.md) - align shared vocabulary, constraints,
  mitigation, and navigation.

Artifacts Consulted:

- Target resolution:
  [`src/target.rs`](../../src/target.rs) - separate discovery-root selection
  from initial-document selection without introducing another filesystem
  reader.
- CLI and compiled-browser boundaries:
  [`src/main.rs`](../../src/main.rs),
  [`tests/cli.rs`](../../tests/cli.rs), and
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - identify
  the stable option and end-to-end verification surfaces.

Decisions to Record:

- Default `--scope repository` for file, directory, and current-directory
  targets; `--scope target` preserves the exact directory or file-parent
  boundary.
- Treat a selected directory as the initial-selection anchor inside the
  repository document set, with repository-level fallback only when that
  directory contains no supported document.
- Reject a repository-scoped target below a hidden repository-relative entry;
  use target scope for a visible nested directory below a hidden parent.

Trace:

- Product feedback -> `FEAT-01` (`UC-02` through `UC-04`) -> `SSD-02` ->
  `OC-02` -> `ADR-019` -> D4 target-loader, CLI, and compiled-browser checks

Exit Criteria:

- The default and explicit scope modes have unambiguous root-selection rules
  for file, directory, current-directory, repository, and non-repository
  targets.
- Directory initial selection remains stable after repository identifiers
  become root-relative.
- Hidden-entry, nested-repository, fixed-document-set, and outside-root
  constraints remain explicit.
- D4 has focused public-boundary oracles and release checks.

Results:

- Accepted ADR-019, superseding the target-root rules in ADR-003 and ADR-018.
- Unified repository recognition across all target kinds and retained a
  visible `--scope target` compatibility and privacy boundary.
- Defined the selected directory as an initial-selection anchor, including a
  repository-level fallback when the selected subtree contains no supported
  document.
- Refined the use cases, system operation, contract, proposal, shared
  vocabulary, quality constraints, risks, and index.
- No PlantUML block changed; the existing SSD event sequence remains valid.
- `git diff --check` and `cargo fmt --check` are the design-iteration
  verification handoff.

Artifact Outcomes:

- started: `ADR-019`, repository-scoped target sessions - records the new
  default and explicit narrow mode.
- refined: proposal, `FEAT-01`, `SSD-02`, and `OC-02` - define the broadened
  behavior and executable construction oracles.
- refined: `ADR-003` and `ADR-018` - retain history and point to ADR-019.
- refined: glossary, supplementary specification, `R-03`, `R-09`, and index -
  align terminology, constraints, risks, and navigation.
