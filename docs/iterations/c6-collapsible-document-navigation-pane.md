# Iteration: C6 Collapsible Document Navigation Pane

Status: completed

Phase Intent:

- Implement proposal 12 as a browser-only presentation slice while keeping the
  session's document authorization and routes unchanged.

Goal:

- Let a user hide and restore the document navigation pane, retain that choice
  while navigating documents in the same browser tab, and preserve an
  accessible restore path.

Risks Addressed:

- `R-03`: a presentation control must not expose a document or create a route
  outside the viewing session's fixed authorized set.
- Accessibility: hiding the pane must leave a keyboard-operable, stateful
  control that users can use to restore it.

Artifacts to Start:

- `ADR-016`, navigation-pane visibility:
  [`docs/decisions/adr-016-collapsible-document-navigation-pane.md`](../decisions/adr-016-collapsible-document-navigation-pane.md) - separates browser presentation state from the server session.

Artifacts to Refine:

- `FEAT-02`, document navigation-pane use cases:
  [`docs/features/document-navigation-pane/use-cases.md`](../features/document-navigation-pane/use-cases.md) - add the hide, persist, and restore scenario.
- `RZ-04`, navigation-pane design:
  [`docs/features/document-navigation-pane/design.md`](../features/document-navigation-pane/design.md) - assign the presentation state to the browser script.
- User and quality documentation:
  [`README.md`](../../README.md), [`docs/supplementary-specification.md`](../supplementary-specification.md), and [`docs/release-readiness.md`](../release-readiness.md) - state the scope and browser evidence.
- Improvement proposals, risk list, and documentation index:
  [`docs/improvement-proposals.md`](../improvement-proposals.md), [`docs/risk-list.md`](../risk-list.md), and [`docs/index.md`](../index.md) - record implementation, authorization evidence, and the decision.
- Browser fixture suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - add the user-visible persistence and restore check.

Artifacts Consulted:

- `ADR-008`, bounded session-catalog search:
  [`docs/decisions/adr-008-paginated-session-catalog-search.md`](../decisions/adr-008-paginated-session-catalog-search.md) - retain the existing authorized-catalog and document-route boundary.
- Viewer page composition:
  [`src/viewer.rs`](../../src/viewer.rs) - retain its existing loopback routes and session state.

Decisions Recorded:

- `ADR-016`: keep the collapsed state in browser-tab storage, not in
  `ViewerState` or a route parameter.

Trace:

- Proposal 12 -> `UC-11` -> `RZ-04` -> ADR-016 -> page template and browser
  script -> unit and browser checks

Test-Driven Evidence:

- Oracle: proposal 12 requires an accessible hide/show control, a persistent
  choice across a document change, a usable restore action, and no change to
  authorized navigation.
- Slice size: one browser interaction sequence crosses the template, script,
  browser-tab storage, document navigation, and restore behavior, so an
  end-to-end test is the narrowest stable boundary.
- Discrimination: `npm run test:browser -- --grep
  'navigation_pane_toggle_then_persists_visibility_and_restores_the_pane'`
  initially timed out looking for the absent **Hide documents** button.
- Green evidence: the focused unit and browser checks pass after adding the
  control, script, and layout behavior; final repository validation is recorded
  below.

Exit Criteria:

- A visible keyboard-operable control exposes an expanded state and remains
  available after the pane is hidden.
- The same browser tab remembers the hidden choice while viewing another
  authorized document.
- Hiding the pane changes neither the authorized document set nor a document
  route.
- The full formatting, test, lint, package, and browser checks pass.

Results:

- Added a button outside the navigation pane that synchronizes its visible
  label and `aria-expanded` state with the pane's `hidden` state.
- Persisted only the Boolean presentation choice in `sessionStorage`; the Axum
  routes, `ViewerState`, and immutable `DocumentCatalog` receive no new input.
- Kept no-script navigation visible by revealing the control only after the
  application script is active.
- Added unit markup coverage and a browser scenario that hides the pane,
  loads another known document, then restores the pane and its active marker.
- Final validation passed: `cargo fmt --check`, `cargo test --locked` (53
  library tests and three CLI tests), `cargo clippy --locked --all-targets
  --all-features -- -D warnings`, `cargo package --allow-dirty`, and all 14
  `npm run test:browser` scenarios.

Artifact Outcomes:

- started: `ADR-016`, navigation-pane visibility - records the browser-only
  persistence and authorization boundary.
- refined: `FEAT-02` and `RZ-04` - define and realize the presentation
  interaction.
- refined: `BTE-01`, browser end-to-end suite - verifies hiding, persistence,
  document routing, and restoration.
- refined: user, quality, release, proposal, risk, and index documentation -
  describes the behavior and its evidence.
