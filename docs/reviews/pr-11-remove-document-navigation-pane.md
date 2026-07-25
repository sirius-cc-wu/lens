---
type: "Code Review"
title: "PR #11 Review: Remove the Document Navigation Pane"
description: "Reviews the focused document-view change, retained authorization boundary, compatibility paths, tests, and current architecture documentation."
pull_request: 11
status: "completed"
date: "2026-07-26"
tags: [review, navigation, viewer]
---

# PR #11 Review: Remove the Document Navigation Pane

Review range:
`ebf474ed8d89669ad4632ba342cf1d74e9cb70be..50338720d4025e0fb060429b02f0197c83b64dd9`.
The base is the merge base with the fetched `origin/main`; the head is the
pull request's reported head commit.

## Findings

1. **[Low] The active refresh design still describes the replaced state fields
   — `docs/features/automatic-refresh/design.md:98`**

   Explanation and impact: This pull request replaces
   `ViewerState::known_documents: BTreeSet<String>` with two explicitly
   different responsibilities:
   `known_documents: KnownDocuments` authorizes document and revision routes,
   while `known_document_ids: BTreeSet<String>` authorizes authored-link
   rewriting during initial rendering and refresh. The active FEAT-03 design
   still shows `document_ids: BTreeMap<String, usize>` and
   `known_documents: BTreeSet<String>`, and its construction result repeats
   those old names. After this pull request, the current markdown-viewing
   design and automatic-refresh design therefore disagree about the same
   `ViewerState`. A maintainer following FEAT-03 can assign a route or refresh
   change to the wrong structure or mistake a removed field for current code.

   Reported behavior and impact:

   ```plantuml
   @startuml
   title Stale automatic-refresh design after PR #11
   left to right direction

   actor Maintainer
   artifact "src/viewer/state.rs\nPR head" as Code
   artifact "markdown-viewing/uml-design.md\nupdated current view" as ViewerDesign
   artifact "automatic-refresh/design.md\nactive stale view" as RefreshDesign

   note bottom of Code
     known_documents: KnownDocuments
     known_document_ids: BTreeSet<String>
   end note

   note bottom of RefreshDesign
     document_ids: BTreeMap<String, usize>
     known_documents: BTreeSet<String>
   end note

   Code --> ViewerDesign : agrees
   Code -[#red,dashed]-> RefreshDesign : disagrees
   RefreshDesign --> Maintainer : supplies obsolete\nfield responsibilities
   @enduml
   ```

   Proposed fix: Update FEAT-03's DCD-03, responsibility notes, and construction
   result to show `KnownDocuments` as the route-authorization lookup and
   `known_document_ids` as the fixed link-authorization set used when refreshed
   Markdown is rendered. Remove the obsolete `document_ids` field from that
   current design.

   Suggested solution:

   ```plantuml
   @startuml
   title Align FEAT-03 with the implemented ViewerState
   left to right direction
   skinparam classAttributeIconSize 0

   class ViewerState {
     documents: RwLock<Vec<ViewerDocument>>
     known_documents: KnownDocuments
     known_document_ids: BTreeSet<String>
   }
   class KnownDocuments {
     document_indices: BTreeMap<String, usize>
     index(identifier): Option<usize>
   }
   class "Fixed authored-link identifiers" as KnownDocumentIds {
     BTreeSet<String>
   }
   class "Revision handler" as Revision
   class "Refresh renderer" as Refresh

   ViewerState *-- KnownDocuments : route authorization
   ViewerState *-- KnownDocumentIds : link authorization
   Revision --> KnownDocuments : resolve requested identifier
   Refresh --> KnownDocumentIds : render refreshed Markdown links
   @enduml
   ```

   Test coverage: No runtime test is missing for this documentation-only
   defect. After updating FEAT-03, render its modified PlantUML blocks through
   the configured server, verify local Markdown links, and compare every
   documented `ViewerState` field with `src/viewer/state.rs`.

## Validation

- `git diff --check origin/main...HEAD` passed.
- `cargo fmt --check` passed.
- `cargo test --locked` passed 61 library tests and 5 CLI integration tests.
- `cargo clippy --locked --all-targets --all-features -- -D warnings` passed.
- `cargo package --allow-dirty` built and verified the package.
- `npm run test:browser -- --reporter=line` passed all 21 browser scenarios.
- Local targets in all 20 changed, non-deleted Markdown files exist.
- The three materially changed design diagrams and both review diagrams
  returned non-empty `image/svg+xml` responses with HTTP 200 from the
  configured PlantUML server.
- The worktree contained no unrelated tracked changes before this review
  record was added.

## Residual Risks

- Large-repository discovery, eager rendering, memory use, and repeated refresh
  reads remain unmeasured; the pull request intentionally retains them and
  tracks them under `R-09` and improvement 14.
- Browser checks verify the single-column geometry and absence of removed
  controls at narrow and wide viewports. This review did not repeat the
  proposal author's separate subjective visual walkthrough.
