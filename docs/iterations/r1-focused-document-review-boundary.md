---
type: "Iteration Record"
title: "Iteration: R1 Focused Document Review Boundary"
description: "Records the product and architecture boundary that delegates generic document selection while preserving fixed-session authorization."
id: "R1"
phase: "elaboration"
status: "completed"
tags: [iteration]
---

# Iteration: R1 Focused Document Review Boundary

Status: completed

Phase Intent:

- Resolve the product and authorization decisions behind navigation-pane
  removal before changing the implemented route and page responsibilities.

Goal:

- Establish Lens as a focused human-review surface whose fixed discovered set
  authorizes document routes and authored links without supplying a generic
  in-browser catalog.

Risks Addressed:

- `R-03`: removing `DocumentCatalog` could accidentally make a browser
  identifier a filesystem path or weaken the fixed discovered-document
  authorization boundary.
- Product compatibility: retiring the pane could leave active requirements,
  decisions, or proposed design work implying that catalog search remains part
  of Lens.

Artifacts to Start:

- `ADR-020`, focused document review and external resource selection:
  [`docs/decisions/adr-020-focused-document-review.md`](../decisions/adr-020-focused-document-review.md) - record the retained authorization lookup and the boundary between Lens and coding-agent or command-line selection.

Artifacts to Refine:

- `FEAT-02`, document navigation-pane use cases:
  [`docs/features/document-navigation-pane/use-cases.md`](../features/document-navigation-pane/use-cases.md) - retire `UC-07`, `UC-08`, and `UC-11` while preserving implementation history.
- Navigation-pane analysis and design:
  [`docs/features/document-navigation-pane/`](../features/document-navigation-pane/) - mark the catalog system operation, contract, and realization as retired.
- `ADR-008` and `ADR-016`:
  [`docs/decisions/`](../decisions/) - supersede searchable catalog and
  visibility-state decisions with ADR-020.
- Design-system proposal:
  [`docs/proposals/establish-design-system.md`](../proposals/establish-design-system.md) - remove catalog-only concept surfaces and constraints.
- Documentation index: [`docs/index.md`](../index.md) - make current and
  retired lifecycle status visible.

Artifacts Consulted:

- Navigation-pane removal proposal:
  [`docs/proposals/remove-document-navigation-pane.md`](../proposals/remove-document-navigation-pane.md)
- `FEAT-01`, safe authored Markdown navigation:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md)
- `ADR-003`, document-root discovery:
  [`docs/decisions/adr-003-document-root-discovery.md`](../decisions/adr-003-document-root-discovery.md)
- `ADR-019`, repository and target scope:
  [`docs/decisions/adr-019-repository-scoped-target-sessions.md`](../decisions/adr-019-repository-scoped-target-sessions.md)

Decisions to Record:

- Keep one immutable identifier-to-document lookup for route authorization and
  one fixed identifier set for authored-link rewriting; remove query parsing,
  search, pagination, and catalog presentation.
- Treat coding agents and command-line tools as the generic resource selectors.
  Lens continues to own target validation, initial selection, safe known
  routes, rendering, and refresh.

Trace:

- Navigation-pane removal proposal -> ADR-020 -> retired `FEAT-02` / ADR-008 /
  ADR-016 -> planned R2 production and browser checks

Exit Criteria:

- ADR-020 defines external selection, retained route authorization, inert
  former query parameters, and compatibility constraints.
- Navigation-pane requirements and design artifacts are visibly retired
  without rewriting historical iteration records.
- The still-proposed design system no longer requires navigation search,
  pagination, pane visibility, or catalog-specific states.
- The documentation index distinguishes retired navigation artifacts from
  active behavior.

Results:

- ADR-020 assigns generic discovery and selection to coding agents and
  command-line tools while retaining an immutable identifier-to-document
  lookup for routes and the corresponding fixed identifier set for authored
  links.
- ADR-008 and ADR-016 are superseded. `FEAT-02`, `UC-07`, `UC-08`, `UC-11`,
  `SSD-03`, `OC-03`, and the navigation-pane realization are retired without
  changing the historical C3, C5, or C6 records. ADR-006 remains historically
  superseded by ADR-008.
- The design-system proposal now explores the focused single-column document,
  authored links, and browser history instead of requiring catalog search,
  pagination, pane visibility, or current-result states.
- `git diff --check` passed, and every new Markdown link in the R1 decision and
  iteration record resolved to a repository file. No PlantUML block changed,
  so the configured PlantUML server did not need to be invoked.
- Residual implementation risk moves to R2: the production lookup, routes,
  page shell, assets, and tests still implement the superseded catalog
  behavior until that construction iteration completes.

Artifact Outcomes:

- started: `ADR-020`, focused document review:
  [`docs/decisions/adr-020-focused-document-review.md`](../decisions/adr-020-focused-document-review.md) - records external resource selection and retained fixed-session authorization.
- refined: `FEAT-02`, document navigation-pane use cases:
  [`docs/features/document-navigation-pane/use-cases.md`](../features/document-navigation-pane/use-cases.md) - retires the feature and its three use cases while preserving their scenarios as history.
- refined: navigation-pane analysis and design:
  [`docs/features/document-navigation-pane/`](../features/document-navigation-pane/) - retires `SSD-03`, `OC-03`, and the implemented realization.
- refined: `ADR-008` and `ADR-016`:
  [`docs/decisions/`](../decisions/) - mark both decisions superseded by
  ADR-020 while leaving ADR-006's historical succession unchanged.
- refined: Design-system proposal:
  [`docs/proposals/establish-design-system.md`](../proposals/establish-design-system.md) - replaces catalog-only surfaces and criteria with focused reading and authored-link states.
- refined: Documentation index: [`docs/index.md`](../index.md) - labels
  `FEAT-02` retired and links ADR-020.
