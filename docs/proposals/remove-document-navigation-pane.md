---
type: "Improvement Proposal"
title: "Remove the Document Navigation Pane"
description: "Removes the in-browser document catalog, identifier search, pagination, and pane visibility control in favor of selecting the initial document from the command line."
id: "PROP-REMOVE-DOCUMENT-NAVIGATION-PANE"
status: "proposed"
tags: [proposal, navigation, command-line, user-interface]
---

# Remove the Document Navigation Pane

Status: proposed

## Summary

Remove the document navigation pane from Lens. Document pages should no longer
contain the discovered-document catalog, identifier search form, result
pagination, current-result marker, or **Hide documents** and **Show documents**
control.

Selecting the document before starting Lens is faster for the intended
workflow. A user can find a file with `fd` and pass the resulting path directly
to Lens:

```bash
lens "$(fd '<file-pattern>')"
```

This is shell command substitution: the shell replaces the `fd` expression
with its output before starting Lens. The query must produce exactly one path
because Lens accepts one optional target.

Lens should remain a linked-Markdown viewer after the pane is removed.
Relative links between discovered Markdown documents should continue to open
inside the same viewing session. Standalone PlantUML files should remain
selectable as direct CLI targets and reachable through known document URLs.
Browser history, authored Markdown links, automatic refresh, and the
initial-document selection rules should remain unchanged.

## Motivation

The navigation pane duplicates a workflow that command-line file search
already performs more quickly and flexibly. The current interface provides a
session catalog, submitted identifier search, result counts, pagination, a
current-document marker, responsive sidebar styling, and a per-tab visibility
preference. Those behaviors are substantial compared with the simple user
goal of selecting a known file.

For a command-line user, searching before launch has several advantages:

- `fd` searches the repository tree from the current directory and returns a
  path Lens already accepts.
- The shell retains the user's preferred filters, aliases, completion, and
  history.
- Lens opens the selected document immediately instead of first opening one
  document and then searching a second interface.
- The browser can dedicate its width to the document, tables, code examples,
  and diagrams.

The pane's collapsible state partly acknowledges that it competes with the
content for space. Removing the pane makes the reading surface consistently
focused and eliminates UI state whose only purpose is hiding another Lens
control.

This proposal accepts reduced in-browser discoverability. A document that has
no authored inbound link will not be reachable through a visible Lens catalog;
the user must select it from the command line or open a known document URL.
That trade-off matches the intended command-line-first workflow.

## Goals

- Make the document the only primary content on a Lens page.
- Let users select an initial file with their existing command-line search
  tools.
- Preserve safe authored links between Markdown documents in the fixed session
  set.
- Preserve document discovery as the authorization basis for known links,
  routes, and refresh behavior.
- Remove catalog-only server logic, page markup, styling, script, browser
  state, and tests.
- Keep Lens independent of `fd` or any other external file-search command.

## Proposed Behavior

| Situation | Lens behavior |
|---|---|
| Lens displays the initial or another known document | Render the document header and content without a document navigation pane or pane toggle. |
| The user starts Lens with one file selected by `fd` or another shell command | Apply the existing direct-file target and initial-document rules. |
| The current Markdown document links to another discovered Markdown document | Preserve the existing Lens document route and display the linked document. |
| The user selects a standalone PlantUML file as the CLI target | Preserve direct PlantUML rendering as the initial document. |
| The user uses browser Back or Forward after following document links | Preserve normal browser history behavior. |
| A request uses a known `/documents/{identifier}` route | Continue to display that authorized document without a catalog. |
| A known document URL contains `query` or `page` parameters from the former catalog | Display the document and ignore the obsolete parameters. |
| A requested identifier is not in the discovered set | Preserve Lens-owned not-found guidance without reading a browser-supplied filesystem path. |
| A document changes on disk | Preserve the current automatic refresh behavior. |

The page should not add breadcrumbs, previous/next document controls, a command
palette, or another catalog replacement. Authored links and the command line
are the intentional navigation surfaces.

## User Scenario

Primary actor: Developer or technical writer

Goal: Find one repository document from the command line and read it in an
uncluttered Lens page.

Preconditions:

- The user can run Lens.
- The user knows the path or has a file-search tool such as `fd`.

Main success scenario:

1. The user searches for a document in the shell.
2. The shell passes the one resulting path to Lens.
3. Lens applies its existing target validation and document-root rules.
4. Lens opens the selected document in the browser.
5. The browser displays the document without a catalog, search form, or pane
   visibility control.
6. The user reads the document.
7. The user optionally follows an authored link to another discovered Markdown
   document and uses browser history to return.

Extensions:

- 2a. If the shell command returns no path or multiple paths, Lens retains its
  existing command-line or target-validation behavior. Lens does not interpret
  or correct the search output.
- 3a. If the selected target is missing, hidden, symbolic, unreadable, or
  unsupported, Lens retains its existing actionable target error.
- 7a. If an authored local link does not resolve to a discovered document,
  Lens retains its existing guidance behavior and does not read the path from
  the browser request.

## Accepted Trade-Offs

### Documents without inbound links require command-line selection

The pane currently makes every discovered identifier reachable even when no
document links to it. After removal, the browser exposes only the initial
document and destinations authored by the repository. This makes documentation
structure more dependent on useful indexes and links, but it avoids maintaining
a second file finder inside Lens.

### Starting another document may start another Lens process

The quickest way to switch to an unlinked document may be to stop the current
process and run Lens with another selected file. Process reuse, a persistent
Lens daemon, and browser-session retargeting are separate product decisions.

### `fd` is optional and user-managed

Lens should show `fd` only as an example of command composition. It should not
invoke `fd`, add it as an installation requirement, depend on its output
format, or promise that an arbitrary query returns one path. Users may instead
pass a literal path or use `find`, a fuzzy finder, shell completion, or another
tool. The example uses POSIX shell syntax; PowerShell and other supported user
environments may require different command-substitution syntax.

### Browser content search is unchanged

The removed search matches only document identifiers. Browser Find continues
to search the displayed page, and this proposal does not add repository
content search.

## Authorization and Routing

Removing the pane must not remove the fixed session document set. Lens still
needs discovered documents to:

- recognize and rewrite safe relative document links;
- resolve known `/documents/{identifier}` requests without accepting
  filesystem paths from the browser;
- retain an authorized source and diagram set for each displayed document; and
- detect changes to already discovered documents.

The immutable identifier-search index (document catalog) can be removed, but
the identifier-to-document lookup responsibility remains. The implementation
should retain the smallest cohesive authorization index needed by document
routing and link rewriting instead of preserving catalog terminology for data
that is no longer searchable.

Unknown document identifiers must continue to reach a Lens-owned not-found
page. Its title and copy should change from "Document navigation unavailable"
to language such as "Document unavailable," because the response no longer
refers to a removed navigation feature.

## User Interface

Every successful document page should use a single reading column at all
supported viewport widths. Removing the pane includes removing:

- the `Discovered documents` navigation landmark;
- the **Documents** heading and repository-relative identifier results;
- the identifier search field and submitted query status;
- previous and next result-page links;
- the current-result `aria-current` marker;
- the **Hide documents** and **Show documents** button;
- collapsed-pane data attributes and `sessionStorage` state;
- sidebar grid columns, sticky positioning, responsive pane rules, and
  navigation-only component styles.

This removal should not weaken semantic structure within the document. The
document title, metadata, Markdown content, diagrams, errors, retry controls,
focus indicators, and readable overflow remain subject to their current
accessibility and responsive requirements.

## Compatibility and Migration

The CLI syntax, supported target types, browser-launch behavior, document
routes, and filesystem boundary do not change.

Existing bookmarked document URLs continue to open while their identifier
remains in the session's discovered set. Former `query` and `page` parameters
become inert and may be dropped by later navigation. No redirect or error is
needed for those parameters.

User guidance should replace pane instructions with command-line selection
examples. A safe example should state that the substituted search must return
one path. Lens should not present `fd` as a runtime requirement.

Historical iteration records should continue to describe what C3, C5, and C6
implemented. Current documentation should instead:

- mark `FEAT-02` and `UC-07`, `UC-08`, and `UC-11` as retired;
- supersede ADR-008 and ADR-016 with the decision that accepts this proposal;
- retain ADR-006 as historical and already superseded by ADR-008;
- remove the pane from current README and release-readiness guidance;
- update the documentation index so statuses are not misleading; and
- identify the removal in release notes as a user-interface compatibility
  change.

The proposed
[Lens design system](establish-design-system.md) currently treats navigation
search, pagination, and pane visibility as required concept surfaces. If both
proposals remain active, accepting this removal should first revise those
design-system requirements so visual exploration does not preserve UI that
Lens has decided to retire.

## Implementation Approach

An implementation should proceed as a narrow removal:

1. Add the successor decision and retire the active navigation-pane use cases.
2. Remove raw catalog query parsing, result paging, search-status rendering,
   and the dedicated catalog module.
3. Replace catalog-owned identifier lookup with a minimal known-document
   lookup used only for authorization and routing.
4. Remove navigation-pane and toggle markup from successful document pages.
5. Remove navigation-only JavaScript, temporary browser storage, and CSS; make
   the document layout single-column.
6. Remove or rewrite catalog-specific unit and browser tests while retaining
   linked-document and unknown-document coverage.
7. Update current user, architecture, risk, release, and verification
   documentation without rewriting historical iteration evidence.

The removal should not be combined with changes to initial-document selection,
document-root discovery, Markdown link syntax, source-file editor handoff,
automatic refresh, or visual redesign. Those concerns can change
independently.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Users cannot discover an unlinked document from the browser. | Document the command-line-first workflow and encourage repository indexes and authored links where in-browser traversal matters. |
| An `fd` expression returns multiple paths and Lens cannot open the intended single target. | State that command substitution must produce exactly one target; keep search-tool behavior outside Lens. |
| Removing `DocumentCatalog` accidentally weakens known-document routing. | Preserve a typed identifier lookup and retain linked, direct-known, traversal, hidden, and symbolic-link route tests. |
| Layout removal regresses narrow or wide reading behavior. | Verify the single-column page at representative viewport widths with long tables, code, identifiers, and diagrams. |
| Historical documents imply the pane still exists. | Retain historical records but clearly mark current use cases and decisions as retired or superseded. |
| Pane removal is mixed with a broader redesign and regressions are difficult to attribute. | Keep the implementation mechanical and retain existing document styling until a separate design proposal is accepted. |

## Rejected Alternatives

### Keep the pane collapsed by default

A collapsed default retains the toggle, browser state, markup branches,
responsive rules, accessibility behavior, and catalog implementation while
hiding their primary output. It does not achieve the simplification goal.

### Keep only identifier search

A search field without the result pane still duplicates command-line file
selection and requires query parsing, result rendering, authorization indexing,
empty states, limits, and pagination.

### Replace the pane with a command palette

A command palette moves the same catalog into a more script-dependent
interface. It adds keyboard interaction and focus-management responsibilities
without improving the command-line-first workflow.

### Add previous and next document controls

Lexical adjacency does not necessarily represent meaningful reading order.
Repositories can express intended sequences with ordinary Markdown links.

### Invoke `fd` from Lens

Running an external search command would add an installation dependency,
process and error handling, platform differences, result parsing, and another
filesystem selection path. Shell composition already provides the desired
behavior before Lens starts.

### Stop discovering documents when a direct file is selected

Discovery still authorizes relative document links, known routes, diagrams,
and automatic refresh. Removing it would be a security and navigation redesign,
not a consequence of removing the pane.

## Acceptance Criteria

- Successful document pages contain no `Discovered documents` navigation
  landmark, catalog result list, identifier search form, search status, or
  result pagination.
- Successful document pages contain no pane visibility button, collapsed state
  attribute, or navigation visibility value in `sessionStorage`.
- The page uses a single document-focused layout at narrow and wide viewports.
- Initial-document selection remains unchanged for current-directory,
  directory, Markdown-file, and PlantUML-file targets.
- A relative link to a discovered Markdown document still opens that document
  through a known Lens route.
- A standalone PlantUML file remains usable as a direct target and through a
  known document route.
- A direct request for a known document identifier still displays only that
  authorized document.
- Unknown, hidden, symbolic-link, traversal, and out-of-root identifiers retain
  Lens-owned guidance without exposing file contents.
- `query` and `page` parameters no longer affect a known document response.
- Automatic refresh and per-diagram rendering behavior remain unchanged.
- Lens does not invoke or depend on `fd`.
- Current requirements, decisions, README, release readiness, release notes,
  and documentation index consistently describe the command-line-first
  navigation model.

## Verification

Automated coverage should include behavior-named scenarios such as:

- `document_page_then_omits_document_navigation_controls`;
- `document_page_with_catalog_query_then_ignores_query_and_page`;
- `known_markdown_link_then_displays_linked_document`;
- `direct_plantuml_target_then_displays_diagram_without_navigation_pane`;
- `known_document_route_then_displays_authorized_document`;
- `unknown_document_route_then_returns_guidance_without_source`; and
- `changed_displayed_document_then_refreshes_without_navigation_state`.

Each new or changed test should use explicit setup, one primary action, and
verification sections. Remove tests whose only observable behavior is catalog
search, result pagination, current-result marking, pane toggling, or visibility
storage. Preserve browser evidence for authored document links rather than
relying on a link that formerly came from the pane.

### Manual end-to-end test

- **Setup:** Prepare a repository with a root README, an unlinked nested
  Markdown document, two Markdown documents that link to each other, a
  standalone PlantUML file, a hidden document, and a symbolic-link document.
- **Actions:** Use an `fd` command that returns the nested document and pass its
  single result to Lens. Inspect the page at narrow and wide viewport widths,
  follow the authored Markdown links, use browser Back, restart Lens with the
  PlantUML file selected directly, request a known document URL with former
  `query` and `page` parameters, and request the hidden and symbolic-link
  identifiers.
- **Expected result:** The selected nested document opens first in a
  single-column page with no catalog or pane control. Authored links and browser
  history work. Former catalog parameters have no effect. Disallowed
  identifiers show Lens guidance without exposing source.

## Analysis and Design Trace

- Feature to retire:
  [`FEAT-02`](../features/document-navigation-pane/use-cases.md)
- Current catalog decision:
  [`ADR-008`](../decisions/adr-008-paginated-session-catalog-search.md)
- Current visibility decision:
  [`ADR-016`](../decisions/adr-016-collapsible-document-navigation-pane.md)
- Preserved linked-document behavior: `UC-04` in
  [`FEAT-01`](../features/markdown-viewing/use-cases.md)
- Preserved authorization boundary:
  [`ADR-003`](../decisions/adr-003-document-root-discovery.md)
- Construction history:
  [`C3`](../iterations/c3-document-navigation-pane.md),
  [`C5`](../iterations/c5-scalable-document-navigation-search.md), and
  [`C6`](../iterations/c6-collapsible-document-navigation-pane.md)
- Future elaboration: record the successor decision, retire catalog system
  operations, and design the smallest known-document lookup that keeps routing
  authorization explicit.

## Out of Scope

- Changing document discovery or the fixed filesystem boundary.
- Changing direct-file, directory, or current-directory initial selection.
- Removing authored Markdown document links.
- Adding Markdown link rewriting for standalone PlantUML files.
- Adding content search, fuzzy finding, a command palette, or breadcrumbs.
- Adding a persistent Lens process or retargeting an active browser session.
- Bundling, invoking, or configuring `fd`.
- Implementing the VS Code source-link proposal.
- Redesigning document typography, metadata, diagrams, or other controls.
