# Iteration: N1 Document Navigation Pane Scope

Status: completed

Phase Intent:

- Establish the selected post-V1 feature's user value, security constraints,
  and durable artifact layout before choosing its browser interaction or Rust
  design.

Goal:

- Define a document-navigation pane that lets users locate and open only the
  documents already authorized for the current Lens viewing session.

Risks Addressed:

- `R-03`: a navigation aid could accidentally bypass the discovered-document
  authorization boundary or reveal excluded documents.

Artifacts to Start:

- `FEAT-02`, document navigation-pane use cases:
  [`docs/features/document-navigation-pane/use-cases.md`](../features/document-navigation-pane/use-cases.md) - define the user goals, alternate flows, and authorization constraints.

Artifacts to Refine:

- Improvement proposals:
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - mark proposal 3 as selected.
- Risk list: [`docs/risk-list.md`](../risk-list.md) - make the sidebar's use of
  the existing document-set boundary explicit.
- Documentation index: [`docs/index.md`](../index.md) - link the canonical
  feature artifact.

Artifacts Consulted:

- `FEAT-01`, Markdown-viewing use cases:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md)
- `SSD-02`, open and navigate a document root:
  [`docs/features/markdown-viewing/ssd-02-open-document-root.md`](../features/markdown-viewing/ssd-02-open-document-root.md)
- `OC-02`, open a document root:
  [`docs/features/markdown-viewing/oc-02-open-document-root.md`](../features/markdown-viewing/oc-02-open-document-root.md)
- `ADR-003`, document-root discovery:
  [`docs/decisions/adr-003-document-root-discovery.md`](../decisions/adr-003-document-root-discovery.md)

Decisions to Record:

- Decide in N2 whether the catalog and search can be rendered entirely from
  existing session data without a new server endpoint.

Trace:

- Proposal 3 -> `FEAT-02` (`UC-07`, `UC-08`) -> planned `SSD-03` and `OC-03`
  -> construction acceptance checks

Exit Criteria:

- The feature has canonical, black-box use cases for browsing and filtering.
- The use cases state that the navigation pane may use only the existing
  authorized document set and cannot become a filesystem or content search.
- The iteration identifies the next design question and its executable
  verification needs.

Results:

- `FEAT-02` now defines browsing and filtering as distinct user goals. It
  preserves links without scripting and confines filtering to displayed,
  authorized identifiers.
- The feature is a package because its use cases, forthcoming system sequence,
  contract, and realization will change independently; it follows the existing
  feature-oriented documentation layout.
- N2 will detail the existing document-request event, catalog presentation,
  browser-side filter behavior, and the decision to avoid another server route.

Artifact Outcomes:

- started: `FEAT-02`, browse the discovered document set:
  [`docs/features/document-navigation-pane/use-cases.md`](../features/document-navigation-pane/use-cases.md) - defines `UC-07` and `UC-08`.
- refined: Improvement proposals:
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - proposal 3 is selected for implementation.
- refined: Risk list: [`docs/risk-list.md`](../risk-list.md) - adds the
  navigation-pane condition to `R-03`.
- refined: Documentation index: [`docs/index.md`](../index.md) - links
  `FEAT-02`.
