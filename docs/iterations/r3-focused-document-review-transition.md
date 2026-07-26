---
type: "Iteration Record"
title: "Iteration: R3 Focused Document Review Transition"
description: "Reconciles current documentation, completes acceptance evidence, and retires the implemented navigation-pane removal proposal."
id: "R3"
phase: "transition"
status: "completed"
tags: [iteration]
---

# Iteration: R3 Focused Document Review Transition

Status: completed

Phase Intent:

- Bring user guidance, architecture, risks, release records, and verification
  into agreement with the implemented focused-review behavior and prove
  readiness for review.

Goal:

- Complete the proposal acceptance walkthrough, preserve the durable outcome in
  current artifacts, and remove the implemented proposal from the active set.

Risks Addressed:

- Documentation drift: current guidance could continue directing users to a
  pane that no longer exists or imply that `fd` is a Lens dependency.
- Architecture drift: the current viewer design could retain
  `DocumentCatalog` responsibilities after production moved to
  `KnownDocuments`.
- Compatibility evidence: automated checks alone could miss the combined
  coding-agent/command-line, viewport, authored-link, history, direct PlantUML,
  obsolete-query, and unavailable-document workflow.

Artifacts to Start:

- None. The proposal, ADR-020, R1, and R2 already provide canonical scope,
  decision, and construction evidence.

Artifacts to Refine:

- User guidance: [`README.md`](../../README.md) - explain direct coding-agent
  and command-line selection, including a safe optional `fd` composition
  example.
- Current architecture:
  [`docs/features/markdown-viewing/uml-design.md`](../features/markdown-viewing/uml-design.md) - replace searchable catalog responsibilities with fixed known-document lookup.
- Current requirements and risks:
  [`docs/supplementary-specification.md`](../supplementary-specification.md),
  [`docs/risk-list.md`](../risk-list.md), and
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - remove
  catalog-only limits and measurement work while preserving discovery and
  refresh concerns.
- Release and verification:
  [`docs/release-readiness.md`](../release-readiness.md) and
  [`docs/release-notes.md`](../release-notes.md) - record focused-layout
  acceptance, compatibility impact, and executable evidence.
- Documentation index: [`docs/index.md`](../index.md) - describe the current
  focused human-review surface and R3 evidence.
- Navigation-pane removal proposal:
  `docs/proposals/remove-document-navigation-pane.md` - delete the implemented
  proposal after its durable outcome and acceptance evidence are current.

Artifacts Consulted:

- Navigation-pane removal proposal, retired in this iteration after ADR-020 and
  R1 through R3 preserved its durable outcome.
- `ADR-020`, focused document review:
  [`docs/decisions/adr-020-focused-document-review.md`](../decisions/adr-020-focused-document-review.md)
- R1 decision and R2 construction records:
  [`docs/iterations/r1-focused-document-review-boundary.md`](r1-focused-document-review-boundary.md) and
  [`docs/iterations/r2-remove-document-navigation-pane.md`](r2-remove-document-navigation-pane.md)

Decisions to Record:

- Retain `fd` only as a POSIX-shell composition example whose command must
  return exactly one path; do not add it as an installation or runtime
  requirement.
- Treat the removed pane, identifier search, pagination, and visibility state
  as a user-interface compatibility change in pending release notes.
- Remove the implemented proposal only after current artifacts and verification
  retain its durable outcome.

Trace:

- Navigation-pane removal proposal -> ADR-020 -> R1 -> R2 -> R3 acceptance and
  transition evidence -> proposal retirement

Exit Criteria:

- Current user, architecture, risk, release, verification, and index documents
  consistently describe focused review and external resource selection.
- The materially changed PlantUML design renders through the configured server,
  or server unavailability is recorded accurately.
- The proposal's manual end-to-end walkthrough passes for repository and target
  scope, narrow and wide layout, authored links/history, unlinked direct
  selection, direct PlantUML, inert former query parameters, and disallowed
  identifiers.
- Formatting, locked Rust tests, Clippy, browser tests, documentation checks,
  and the complete base-to-head diff pass the delivery gate.
- The implemented proposal is deleted only after its durable outcome is linked
  from ADR-020, R1, R2, R3, release notes, and current guidance.

Results:

- README now presents Lens as a focused human-review surface, shows direct
  coding-agent selection, and explains that the optional POSIX-shell `fd`
  example must return exactly one path and does not make `fd` a Lens
  dependency.
- Current requirements, scalability work, and risks no longer retain
  catalog-only controls, query limits, or search measurements. ADR-012 and the
  current UML design preserve direct and known-route PlantUML access plus the
  fixed `KnownDocuments` authorization map.
- All three PlantUML blocks in the materially changed UML design returned HTTP
  200 `image/svg+xml` from the configured PlantUML server. The rendered SVG
  responses were non-empty.
- The compiled-command acceptance walkthrough passed. Browser screenshots at
  390-pixel and 1440-pixel viewports were visually inspected and showed one
  centered reading column with no navigation pane or replacement catalog.
  Authored-link navigation, browser Back, direct and known-route PlantUML,
  repository and target scope, refresh, and diagram retry passed in `BTE-01`.
  Separate live-session walkthroughs confirmed that an unlinked Markdown target
  opens first in repository and target scopes, known-document responses are
  byte-identical with and without former `query` and `page` parameters, and
  hidden, symbolic-link, and traversal identifiers receive 404 Lens guidance
  without source disclosure.
- `cargo fmt --check`, `cargo test --locked` (61 library tests and 5 CLI
  integration tests),
  `cargo clippy --locked --all-targets --all-features -- -D warnings`,
  `cargo package --allow-dirty`, and
  `npm run test:browser -- --reporter=line` (21 scenarios) passed. Local link
  targets in all 13 changed Markdown files resolved.
- The proposal's durable outcome is retained in ADR-020, the retired FEAT-02
  package, R1 through R3, README, the documentation index, release notes,
  release readiness, current architecture, supplementary requirements, and
  risks. The implemented proposal was then deleted according to the repository
  retirement convention.
- The accepted trade-off remains: an unlinked document requires coding-agent
  or command-line selection instead of visible in-browser discovery. Large-set
  discovery and refresh cost remains tracked by `R-09` and improvement 14.

Artifact Outcomes:

- refined: User guidance: [`README.md`](../../README.md) - documents direct
  selection, optional shell composition, authored links, browser history, and
  the absence of an in-browser catalog.
- refined: Current architecture:
  [`docs/features/markdown-viewing/uml-design.md`](../features/markdown-viewing/uml-design.md) - replaces catalog/query/page responsibilities with `KnownDocuments` and the focused page signature.
- refined: Standalone PlantUML decision:
  [`docs/decisions/adr-012-standalone-plantuml-files.md`](../decisions/adr-012-standalone-plantuml-files.md) - records ADR-020's partial succession and current direct/known-route access.
- refined: Current requirements and risks:
  [`docs/supplementary-specification.md`](../supplementary-specification.md),
  [`docs/risk-list.md`](../risk-list.md), and
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - preserve
  focused layout, fixed route authorization, and remaining large-set discovery
  and refresh work without catalog-only requirements.
- refined: Release and verification:
  [`docs/release-readiness.md`](../release-readiness.md) and
  [`docs/release-notes.md`](../release-notes.md) - record the compatibility
  change, migration workflow, automated evidence, and focused-review
  walkthrough.
- refined: Documentation index: [`docs/index.md`](../index.md) - describes
  external target selection and labels superseded pane decisions.
- refined: `ADR-020`, focused document review:
  [`docs/decisions/adr-020-focused-document-review.md`](../decisions/adr-020-focused-document-review.md) - links R3 transition evidence and records proposal retirement.
- retired: Navigation-pane removal proposal:
  `docs/proposals/remove-document-navigation-pane.md` - deleted only after the
  durable outcome and acceptance evidence were current.
