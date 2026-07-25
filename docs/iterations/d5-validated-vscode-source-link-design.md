---
type: "Iteration Record"
title: "Iteration: D5 Validated VS Code Source-Link Design"
description: "Resolves the filesystem-authorization and editor-handoff design before source-link construction begins."
id: "D5"
phase: "elaboration"
status: "completed"
tags: [iteration]
---

# Iteration: D5 Validated VS Code Source-Link Design

Status: completed

Phase Intent:

- Resolve the security-sensitive boundary and Rust responsibility design for a
  direct editor handoff before changing executable behavior.

Goal:

- Define how a known Lens document can present a repository source-file link
  that opens in VS Code without adding source serving, browser-supplied path
  resolution, or Lens-owned process launch.

Risks Addressed:

- `R-02`: an added URL scheme and markup must preserve safe rendering and the
  restrictive browser policy.
- `R-03`: hidden, symbolic, absolute, and root-crossing targets must never
  receive generated editor authority.
- `R-04`: the formerly ambiguous source-viewing goal needs a bounded,
  testable post-V1 behavior.

Artifacts to Start:

- `SSD-06`, open a referenced repository file:
  [`docs/features/markdown-viewing/ssd-06-open-source-link.md`](../features/markdown-viewing/ssd-06-open-source-link.md) -
  separate Lens system events from the browser and operating-system handoff.
- `OC-06`, request a document with source links:
  [`docs/features/markdown-viewing/oc-06-request-document-source-links.md`](../features/markdown-viewing/oc-06-request-document-source-links.md) -
  make authorization and no-route postconditions testable.
- `ADR-020`, validated VS Code source links:
  [`docs/decisions/adr-020-validated-vscode-source-links.md`](../decisions/adr-020-validated-vscode-source-links.md) -
  choose direct validated URLs over source serving or process launch.
- `RZ-06` and `DCD-05`, source-link realization and Rust design:
  [`docs/features/markdown-viewing/source-link-design.md`](../features/markdown-viewing/source-link-design.md) -
  assign fixed-root, filesystem-resolution, and presentation responsibilities.

Artifacts to Refine:

- `FEAT-01`, Markdown-viewing use cases:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md) -
  fully dress `UC-06` as an editor-handoff goal.
- Risks and documentation navigation:
  [`docs/risk-list.md`](../risk-list.md) and
  [`docs/index.md`](../index.md) - link the accepted design and selected
  mitigation.
- Proposal lifecycle:
  [`docs/proposals/open-source-links-in-vscode.md`](../proposals/open-source-links-in-vscode.md) -
  record acceptance for construction.

Artifacts Consulted:

- `ADR-003`, document-root authorization:
  [`docs/decisions/adr-003-document-root-discovery.md`](../decisions/adr-003-document-root-discovery.md)
- `ADR-019`, repository-scoped sessions:
  [`docs/decisions/adr-019-repository-scoped-target-sessions.md`](../decisions/adr-019-repository-scoped-target-sessions.md)
- Existing document rendering and session state:
  [`src/markdown.rs`](../../src/markdown.rs) and
  [`src/viewer/state.rs`](../../src/viewer/state.rs)

Decisions to Record:

- Emit only validated `vscode://file/` links during Markdown rendering; do not
  add an HTTP path operation or launch `code`.
- Retain the canonical root in session state and place filesystem
  authorization plus URL encoding in one cohesive Rust module.
- Give known Lens documents precedence and keep the accessible handoff
  indication inside the rendered source link.

Trace:

- `PROP-OPEN-SOURCE-LINKS-IN-VSCODE` -> `FEAT-01` (`UC-06`) -> `SSD-06` ->
  `OC-06` -> `ADR-020` -> `RZ-06` -> `DCD-05` -> C8 and C9 executable slices

Exit Criteria:

- `UC-06` states the actor goal, fixed-root rule, alternate targets,
  accessibility behavior, and optional-editor failure.
- The SSD and contract distinguish a Lens document request from the later
  browser-to-editor handoff and forbid a browser-supplied filesystem operation.
- An accepted decision records path validation, URL serialization, and
  compatibility scope.
- Rust responsibilities establish one immutable resolver owned by viewer state
  and borrowed by Markdown rendering.
- Construction can proceed with independent authorization/serialization and
  renderer/browser verification slices.

Results:

- Refined `UC-06` from a deferred code-viewing idea into a bounded,
  actor-goal-oriented VS Code handoff with explicit failure and accessibility
  behavior.
- Established `SSD-06` and `OC-06`, separating Lens's known-document request
  from the user-selected browser and operating-system handoff and making the
  no-route, no-source-content, and no-process guarantees explicit.
- Accepted ADR-020 and assigned the implementation to an immutable
  `SourceLinkResolver`, Markdown link transformation, and session-root
  ownership without adding a trait, lock, or browser path operation.
- Recorded C8 as the authorization and serialization slice and C9 as the
  renderer, browser, accessibility, and transition slice.
- `git diff --check`, `cargo fmt --check`, unique artifact-identity checks, and
  canonical-path existence checks passed. This iteration added no PlantUML
  block, so PlantUML validation was not applicable.

Artifact Outcomes:

- started: `SSD-06`, `OC-06`, `ADR-020`, and
  `SOURCE-LINK-DESIGN` at their canonical feature and decision paths.
- refined: `FEAT-01` - fully details `UC-06` and links its realization and Rust
  design.
- accepted: `PROP-OPEN-SOURCE-LINKS-IN-VSCODE` - approved for the C8 and C9
  construction slices.
- refined: `R-03` and `R-04` - record the selected authorization mitigation
  and bounded post-V1 source-viewing scope.
- refined: documentation index - links the source-link design and ADR.
