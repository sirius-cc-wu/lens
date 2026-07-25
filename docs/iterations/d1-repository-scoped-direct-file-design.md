---
type: "Iteration Record"
title: "Iteration: D1 Repository-Scoped Direct-File Design"
description: "Defines the nearest-repository authorization boundary and testable construction targets for directly opened files."
id: "D1"
phase: "elaboration"
status: "completed"
tags: [iteration]
---

# Iteration: D1 Repository-Scoped Direct-File Design

Status: completed

Phase Intent:

- Reduce the authorization, compatibility, and repository-recognition risks
  enough to implement a narrow target-resolution change.

Goal:

- Let a directly opened repository document navigate to supported documents
  outside its parent while retaining one fixed, repository-bounded document
  set.

Risks Addressed:

- `R-03`: broader direct-file discovery must not authorize files outside the
  nearest repository or let browser paths become filesystem paths.
- `R-09`: repository-wide discovery may increase startup and refresh work even
  though catalog responses remain bounded.

Artifacts to Start:

- `ADR-018`, repository-scoped direct-file sessions:
  [`docs/decisions/adr-018-repository-scoped-direct-file-sessions.md`](../decisions/adr-018-repository-scoped-direct-file-sessions.md) -
  records the marker, nearest-root, fallback, and fixed-session decisions.

Artifacts to Refine:

- `FEAT-01`, primary feature and use cases:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md) -
  distinguish current-directory, directory, repository-file, and
  non-repository-file root selection.
- `SSD-02`, open and navigate a document root:
  [`docs/features/markdown-viewing/ssd-02-open-document-root.md`](../features/markdown-viewing/ssd-02-open-document-root.md) -
  make the repository-crossing direct-file scenario explicit without adding
  internal operations.
- `OC-02`, open a document root:
  [`docs/features/markdown-viewing/oc-02-open-document-root.md`](../features/markdown-viewing/oc-02-open-document-root.md) -
  state marker recognition, nearest-root, fallback, and initial-document
  postconditions.
- `ADR-003`, document-root discovery:
  [`docs/decisions/adr-003-document-root-discovery.md`](../decisions/adr-003-document-root-discovery.md) -
  identify its partially superseded direct-file rule.
- Proposal, risk list, and documentation index:
  [`docs/proposals/repository-scoped-direct-file-sessions.md`](../proposals/repository-scoped-direct-file-sessions.md),
  [`docs/risk-list.md`](../risk-list.md), and [`docs/index.md`](../index.md) -
  record acceptance, residual risks, and the durable decision.

Artifacts Consulted:

- Target resolution and discovery:
  [`src/target.rs`](../../src/target.rs) - preserve the existing typed target,
  canonical discovery, and initial-document responsibilities.
- Browser fixture suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - retain
  compiled-command evidence at the user-visible link boundary.

Decisions to Record:

- `ADR-018`: recognize only a non-symbolic-link `.git` directory or regular
  `.git` file, select the nearest canonical ancestor, retain parent fallback,
  and leave explicit roots unchanged.
- Reject a selected file below a hidden repository entry because discovery
  cannot admit it while preserving hidden-entry exclusion.

Trace:

- Proposal -> `FEAT-01` (`UC-02` through `UC-04`) -> `SSD-02` -> `OC-02` ->
  `ADR-018` -> D2 target-loader and compiled-browser checks

Exit Criteria:

- Requirements distinguish all target kinds and both supported `.git` marker
  forms without exposing implementation classes.
- The SSD retains a black-box system boundary and identifies no unnecessary
  new system event.
- The contract states testable root, document-set, and initial-document
  postconditions plus hidden-path failure behavior.
- The architecture decision preserves known-document routing and explains the
  local discovery increase.
- D2 has focused public-boundary unit checks and a compiled-browser scenario.

Results:

- `FEAT-01`, `SSD-02`, and `OC-02` now define nearest-repository selection for
  direct files, parent fallback outside a repository, and unchanged explicit
  directory and current-directory roots.
- ADR-018 accepts both standard and worktree/submodule `.git` forms, ignores a
  symbolic-link marker, and keeps repository recognition independent of Git.
- The existing target loader remains the cohesive owner of canonical target
  resolution and discovery; no new object collaboration, class model, or
  source-module boundary is needed.
- D2 will demonstrate absent behavior with target-loader examples for ordinary,
  worktree, nested, fallback, explicit-directory, symbolic-marker, and hidden
  paths plus a compiled-browser link crossing the direct file's parent.
- The SSD PlantUML block was not changed, so no new diagram validation was
  required.

Artifact Outcomes:

- started: `ADR-018`, repository-scoped direct-file sessions - records the
  accepted authorization boundary.
- refined: `FEAT-01`, `SSD-02`, and `OC-02` - define observable root selection
  and construction oracles.
- refined: `ADR-003` - points to the successor for the direct-file rule while
  retaining its other decisions.
- refined: proposal, `R-03`, `R-09`, and documentation index - record
  acceptance, risk implications, and navigation.
