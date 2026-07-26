---
type: "Iteration Record"
title: "Iteration: C9 Accessible Source-Link Handoff"
description: "Completes accessible source-link rendering, browser acceptance evidence, and user-facing transition guidance."
id: "C9"
phase: "transition"
status: "completed"
tags: [iteration]
---

# Iteration: C9 Accessible Source-Link Handoff

Status: completed

Phase Intent:

- Complete the user-visible editor handoff, verify it through the compiled
  browser boundary, and publish operational guidance for the optional
  integration.

Goal:

- Present qualifying source files as visibly and accessibly indicated VS Code
  links while preserving document, rejected-local, external, and fragment
  behavior and keeping the Lens page usable without an editor handler.

Risks Addressed:

- `R-02`: custom-scheme markup and its indication must remain escaped,
  ordinary, user-selected navigation under the restrictive page policy.
- `R-03`: the compiled browser response must expose editor URLs only for
  resolver-authorized targets and no source-content route.
- Adoption: users need to understand root scope, browser confirmation, optional
  VS Code installation, and the unsupported Insiders scheme.

Artifacts to Start:

- None. D5 established the durable requirements, contract, decision, and
  realization.

Artifacts to Refine:

- Markdown link presentation and stylesheet:
  [`src/markdown.rs`](../../src/markdown.rs) and
  [`src/viewer/assets/app.css`](../../src/viewer/assets/app.css) - keep the
  visible editor indication inside the link's accessible name.
- `BTE-01`, compiled browser suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - inspect
  qualifying, encoded, document, rejected, external, fragment, hover, and
  absent-route behavior without requiring an installed editor.
- User and release guidance:
  [`README.md`](../../README.md),
  [`docs/supplementary-specification.md`](../supplementary-specification.md),
  [`docs/release-readiness.md`](../release-readiness.md), and
  [`docs/release-notes.md`](../release-notes.md) - explain operation,
  constraints, and acceptance checks.
- Proposal, decision, requirements, and risks:
  [`docs/proposals/open-source-links-in-vscode.md`](../proposals/open-source-links-in-vscode.md),
  [`docs/decisions/adr-021-validated-vscode-source-links.md`](../decisions/adr-021-validated-vscode-source-links.md),
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md), and
  [`docs/risk-list.md`](../risk-list.md) - record completed construction and
  executable trace.

Artifacts Consulted:

- `OC-06`, response postconditions:
  [`docs/features/markdown-viewing/oc-06-request-document-source-links.md`](../features/markdown-viewing/oc-06-request-document-source-links.md)
- `C8`, source authorization:
  [`docs/iterations/c8-source-link-authorization.md`](c8-source-link-authorization.md)

Decisions to Record:

- None. C9 completes ADR-021 without adding configurable schemes, source
  positions, browser routes, or Lens-owned editor launch.

Trace:

- `UC-06` -> `SSD-06` -> `OC-06` -> `ADR-021` -> `RZ-06` -> C8 resolver ->
  C9 accessible markup, compiled-browser evidence, and user guidance

Test-Driven Evidence:

- Oracle: OC-06 requires visible text inside the link's accessible name and
  preserves all non-qualifying destinations; the proposal defines compiled
  browser and manual acceptance outcomes.
- Slice size: presentation, browser acceptance, and user guidance form one
  coherent transition outcome after the C8 authorization prerequisite.
- Discrimination:
  `cargo test --locked source_link_with_suffix_then_emits_vscode_url_without_suffix`
  exercised the already authorized source destination before presentation was
  added and failed on the missing visible indicator. The destination assertions
  passed, so the failure distinguished the C9 behavior rather than setup or C8
  authorization.
- Green: the same focused check passed after the indicator was added. The
  known-PlantUML precedence check also passed before the complete Rust and
  compiled-browser suites.

Exit Criteria:

- Every generated editor destination includes visible `(opens in VS Code)` text
  inside the link.
- Compiled-browser checks verify qualifying and space-containing paths,
  document precedence, rejected target classes, external and fragment
  preservation, hover without navigation, and the absence of a source route.
- User guidance explains the fixed root, optional stable VS Code handler,
  browser confirmation, and unsupported VS Code Insiders scheme.
- The proposal is marked implemented and links D5, C8, C9, and ADR-021.
- Required automated and manual acceptance checks pass, with any unavailable
  external validation reported accurately.

Results:

- Added fixed visible `(opens in VS Code)` text inside every Lens-generated
  source anchor, keeping the editor indication in the link's accessible name
  without changing authored external `vscode:` links.
- Added compiled-browser scenarios for canonical and space-containing
  destinations, visible accessible indication, known-document precedence,
  rejected target classes, external and fragment preservation, hover and
  refresh non-navigation, and absent source-content routes.
- Added README, release-readiness, release-note, supplementary-requirement, and
  risk guidance for the fixed root, optional stable VS Code handler, browser
  confirmation, unsupported Insiders scheme, and no-source-route boundary.
- Marked the proposal implemented and completed its D5, C8, C9, and ADR-021
  trace.
- `cargo test --locked` passed all 80 library and 5 CLI tests.
  `cargo clippy --locked --all-targets --all-features -- -D warnings`,
  `cargo fmt --check`, `npm run test:browser` with all 25 scenarios, and
  `git diff --check` also passed.
- The manual Linux transition check used the registered stable `vscode:`
  handler and VS Code 1.115.0 in a disposable nested desktop. VS Code displayed
  its native external-file confirmation, accepted the generated regular-file
  destination, and made the exact file the active editor. A second confirmed
  request decoded `%20` and made the space-containing filename the active
  editor. The live Lens walkthrough also kept a known Markdown document inside
  Lens and left rejected destinations unmodified.
- No PlantUML block changed, so PlantUML server validation was not applicable.

Artifact Outcomes:

- refined: Markdown rendering and viewer styles - add an accessible editor
  indication only when source authorization generated the destination.
- refined and verified: `BTE-01` - covers the compiled response and browser
  behavior without making automated tests depend on an installed editor.
- refined: README, supplementary specification, release readiness, and release
  notes - explain operation, compatibility, and manual acceptance.
- completed: `PROP-OPEN-SOURCE-LINKS-IN-VSCODE` - records implementation
  through D5, C8, and C9.
- refined: `ADR-021`, `FEAT-01`, `R-02`, and `R-03` - link executable
  construction and transition evidence.
