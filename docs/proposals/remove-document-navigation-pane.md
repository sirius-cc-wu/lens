---
type: "Improvement Proposal"
title: "Remove the Document Navigation Pane"
description: "Focuses Lens on rapid human review by delegating document discovery and selection to coding agents and command-line tools."
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

Lens is a human review surface, not a repository browser. A coding agent or
command-line tool should locate and select a file, then start Lens with that
target. Lens should concentrate on what a shell does not provide well: rendered
Markdown, diagrams, document metadata, readable wide content, visible failure
states, safe authored links, and automatic refresh while a human reviews the
result.

A coding agent can pass a known path directly:

```bash
lens docs/features/markdown-viewing/use-cases.md
```

A user working directly in a shell can compose Lens with a file finder:

```bash
lens "$(fd --type file --full-path '<file-pattern>' .)"
```

The shell replaces the `fd` expression with its output before starting Lens
(shell command substitution). The query must produce exactly one path because
Lens accepts one optional positional target. `fd` is only an example; its
search root, matching, and ignore behavior are not Lens behavior and need not
reproduce the session's discovered set.

Lens should remain a linked-Markdown viewer after the pane is removed.
Relative links between discovered Markdown documents should continue to open
inside the same viewing session. Standalone PlantUML files should remain
selectable as direct CLI targets and reachable through known document URLs
during an active session. Browser history, authored Markdown links, automatic
refresh, `--scope repository|target`, and the initial-document selection rules
should remain unchanged.

## Motivation

Lens should not implement generic operations that coding agents and
command-line tools can already perform. Repository-aware tools can search
filenames and paths, apply project context, combine filters, and select a target
before Lens starts. A coding agent often already knows the relevant path from
the task, repository search, or the code it is discussing with the human.

The navigation pane moves that generic selection work back to the human. The
human must scan a catalog, enter an identifier query, submit it, inspect a
bounded result page, possibly paginate, and select another entry. The current
interface therefore maintains a session catalog, submitted identifier search,
result counts, pagination, a current-document marker, responsive sidebar
styling, and a per-tab visibility preference for a workflow that belongs in the
agent or shell. For a human already working through a coding-agent CLI, that
extra browser interaction is slower than asking the agent to identify and open
the target.

Selecting before launch has several advantages:

- A coding agent can use repository context and existing search results to
  select the relevant document without asking the human to browse identifiers.
- The shell retains the user's preferred search tools, filters, aliases,
  completion, and history.
- Lens opens the selected document immediately as a focused human review
  surface.
- The browser can dedicate its width to the document, tables, code examples,
  and diagrams.

The pane's collapsible state partly acknowledges that it competes with the
content for space. Removing the pane makes the reading surface consistently
focused and eliminates UI state whose only purpose is hiding another Lens
control.

This proposal accepts reduced in-browser discoverability. A document that has
no authored inbound link will not be reachable through a visible Lens catalog;
the coding agent or user must select it in the shell or open a known document
URL during the active session. That is the intended boundary: generic resource
discovery remains with the agent or command line, while Lens optimizes the
selected material for human review.

Authored Markdown links remain different from a generic file catalog. They
express relationships chosen by the document author and help a human follow
the document's meaning while reviewing it. Lens should continue to render
those links safely and preserve ordinary browser history.

## Goals

- Make the document the only primary content on a Lens page.
- Make Lens quick to launch from a coding agent or command-line workflow.
- Delegate generic repository discovery and file selection to tools that
  already perform those jobs.
- Optimize rendered documents and diagrams for focused human review.
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
| A coding agent already knows the relevant document | Start Lens with that path and display it immediately for human review. |
| A shell command selects one file | Apply the existing direct-file target, selected scope, and initial-document rules. |
| The human wants to review another unlinked document | Use the coding agent or shell to start Lens with that document instead of searching inside Lens. |
| The current Markdown document links to another discovered Markdown document | Preserve the existing Lens document route and display the linked document. |
| The user selects a standalone PlantUML file as the CLI target | Preserve direct PlantUML rendering as the initial document. |
| The user uses browser Back or Forward after following document links | Preserve normal browser history behavior. |
| A request uses a known `/documents/{identifier}` route | Continue to display that authorized document without a catalog. |
| A known document URL contains `query` or `page` parameters from the former catalog | Display the document and ignore the obsolete parameters. |
| A requested identifier is not in the discovered set | Preserve Lens-owned not-found guidance without reading a browser-supplied filesystem path. |
| A document changes on disk | Preserve the current automatic refresh behavior. |

The page should not add breadcrumbs, previous/next document controls, a command
palette, or another generic catalog replacement. Coding-agent and shell
invocations select resources; authored links and browser history support human
review of meaningful document relationships.

## User Scenario

Primary actor: Human reviewer

Supporting actor: Coding agent or command-line user

Goal: Review one relevant repository document quickly in an uncluttered Lens
page.

Preconditions:

- The user can run Lens.
- A coding agent or the user can identify the relevant path with repository
  context or command-line tools.

Main success scenario:

1. The human asks a coding agent to open a relevant document for review, or
   selects the document with a shell command.
2. The coding agent or shell starts Lens with the selected path and optional
   scope.
3. Lens applies its existing target validation, selected-scope, and
   document-root rules.
4. Lens opens the selected document in the browser.
5. The browser displays the document without a catalog, search form, or pane
   visibility control.
6. The human reviews the document.
7. The human optionally follows an authored link to another discovered Markdown
   document and uses browser history to return.

Extensions:

- 1a. A user without a coding agent can pass a literal path or use `fd`, `find`,
  a fuzzy finder, shell completion, or another command-line tool.
- 2a. If a shell command returns no path or multiple paths, Lens retains its
  existing command-line or target-validation behavior. Lens does not interpret
  or correct the search output.
- 3a. If the selected target is missing, hidden, symbolic, unreadable, or
  unsupported, Lens retains its existing actionable target error.
- 7a. If an authored local link does not resolve to a discovered document,
  Lens retains its existing guidance behavior and does not read the path from
  the browser request.
- 7b. To review an unlinked document, the coding agent or user starts Lens with
  that document as a new target.

## Accepted Trade-Offs

### Documents without inbound links require command-line selection

The pane currently makes every discovered identifier reachable even when no
document links to it. After removal, the browser exposes only the initial
document and destinations authored by the repository. This makes documentation
structure more dependent on useful indexes and links during a review, but it
keeps generic file discovery in the coding agent or shell instead of
maintaining a second file finder inside Lens.

### Reviewing another unlinked document uses another Lens invocation

The intended way to switch to an unlinked document is to ask the coding agent
or use the shell to stop the current process and run Lens with another selected
file. This keeps Lens processes simple and makes resource selection
scriptable. Process reuse, a persistent Lens daemon, and browser-session
retargeting are separate product decisions.

### Standalone PlantUML files require direct selection

Lens does not rewrite authored Markdown links to standalone `.puml` files.
After pane removal, a standalone PlantUML file is not exposed through visible
in-browser discovery. A coding agent or user must select it as a direct CLI
target or use its known URL during the active session. This follows the same
product boundary as unlinked Markdown documents.

### `fd` is optional and user-managed

Lens should show `fd` only as an example of command composition. It should not
invoke `fd`, add it as an installation requirement, depend on its output
format, or promise that its results equal Lens's discovered set. `fd` search
roots, path matching, and ignore-file behavior remain the user's
responsibility. Users and coding agents may instead pass a literal path or use
`find`, `rg`, a fuzzy finder, shell completion, repository indexes, or another
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
routes, and filesystem boundary do not change. Repository scope remains the
default, and `--scope target` remains the explicit narrow-session option
defined by
[ADR-019](../decisions/adr-019-repository-scoped-target-sessions.md).

Existing document URLs continue to open while that Lens process is running and
their identifier remains in the session's discovered set. Lens uses a
session-specific loopback port, so the proposal does not make document URLs
persistent across processes. Former `query` and `page` parameters become inert
and may be dropped by later navigation. No redirect or error is needed for
those parameters.

User guidance should replace pane instructions with coding-agent and
command-line selection examples. A safe command-substitution example should
state that the search must return one path. Lens should not present `fd` as a
runtime requirement or imply that one file finder has the same discovery rules
as Lens.

Historical iteration records should continue to describe what C3, C5, and C6
implemented. Current documentation should instead:

- mark `FEAT-02` and `UC-07`, `UC-08`, and `UC-11` as retired;
- supersede ADR-008 and ADR-016 with the decision that accepts this proposal;
- retain ADR-006 as historical and already superseded by ADR-008;
- remove the pane from current README and release-readiness guidance;
- update the documentation index so statuses are not misleading; and
- identify the removal in release notes as a user-interface compatibility
  change.

After implementation, remove this proposal according to the repository's
proposal-retirement convention. Preserve the durable outcome in the successor
architecture decision, active requirements, iteration record, release notes,
and verification documentation.

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
   the dedicated catalog module, and the now-unused direct query-encoding
   dependency.
3. Replace catalog-owned identifier lookup with a minimal known-document
   lookup used only for authorization and routing.
4. Remove navigation-pane and toggle markup from successful document pages.
5. Remove navigation-only JavaScript, temporary browser storage, and CSS; make
   the document layout single-column.
6. Remove or rewrite catalog-specific unit and browser tests while retaining
   linked-document and unknown-document coverage.
7. Update current user, architecture, risk, release, and verification
   documentation without rewriting historical iteration evidence.
8. Remove this implemented proposal after its durable outcome is represented
   by the successor decision and implementation evidence.

The removal should not be combined with changes to initial-document selection,
repository or target scope, document-root discovery, Markdown link syntax,
source-file editor handoff, automatic refresh, or visual redesign. Those
concerns can change independently.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Users cannot discover an unlinked document from the browser. | Document the coding-agent and command-line workflow and encourage authored indexes and links where semantic in-browser traversal matters. |
| A coding agent is unavailable. | Preserve literal paths and ordinary shell composition with the user's preferred file-search tools. |
| A shell expression returns multiple paths and Lens cannot open the intended single target. | State that command substitution must produce exactly one target; keep search-tool behavior outside Lens. |
| A file finder's results differ from Lens's discovered set. | Treat search tools as user-managed selectors and do not promise identical roots, ignore behavior, or supported-type filters. |
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

A search field without the result pane still moves generic file selection from
the coding agent or shell into Lens. It also requires query parsing, result
rendering, authorization indexing, empty states, limits, and pagination.

### Replace the pane with a command palette

A command palette moves the same catalog into a more script-dependent
interface. It adds keyboard interaction and focus-management responsibilities
while retaining resource-selection behavior outside Lens's human-review
responsibility.

### Add previous and next document controls

Lexical adjacency does not necessarily represent meaningful reading order.
Repositories can express intended sequences with ordinary Markdown links.

### Invoke `fd` from Lens

Running an external search command would add an installation dependency,
process and error handling, platform differences, result parsing, and another
filesystem selection path. It would also make Lens mediate work that belongs to
the calling coding agent or shell. Composition already provides the desired
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
- Repository scope remains the default, and `--scope target` retains its
  existing narrow filesystem boundary and initial-selection behavior.
- A coding agent or shell can start Lens with a known Markdown or PlantUML path
  without an intermediate Lens selection page.
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
  and documentation index consistently describe Lens as a human review surface
  launched from coding-agent and command-line workflows.
- The durable successor decision records the boundary between generic resource
  selection and Lens's human-review responsibilities.

## Verification

Automated coverage should include behavior-named scenarios such as:

- `document_page_then_omits_document_navigation_controls`;
- `document_page_with_catalog_query_then_ignores_query_and_page`;
- `repository_scoped_direct_target_then_preserves_repository_document_links`;
- `target_scoped_direct_target_then_preserves_narrow_document_boundary`;
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
- **Actions:** Ask a coding agent to locate the nested document and start Lens
  with its path, or perform the equivalent selection in a shell. Repeat once in
  repository scope and once with `--scope target`. Inspect the page at narrow
  and wide viewport widths, follow the authored Markdown links, use browser
  Back, start a new Lens invocation with the unlinked document, restart Lens
  with the PlantUML file selected directly, request a known document URL on the
  active session's loopback port with former `query` and `page` parameters, and
  request the hidden and symbolic-link identifiers.
- **Expected result:** The selected nested document opens first in a
  single-column page with no catalog or pane control. Authored links and browser
  history work within the selected scope. A new invocation opens the unlinked
  document directly. Former catalog parameters have no effect. Disallowed
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
- Preserved repository and target scope:
  [`ADR-019`](../decisions/adr-019-repository-scoped-target-sessions.md)
- Construction history:
  [`C3`](../iterations/c3-document-navigation-pane.md),
  [`C5`](../iterations/c5-scalable-document-navigation-search.md), and
  [`C6`](../iterations/c6-collapsible-document-navigation-pane.md)
- Future elaboration: record the boundary between coding-agent or shell
  resource selection and Lens's human-review responsibilities, retire catalog
  system operations, and design the smallest known-document lookup that keeps
  routing authorization explicit.

## Out of Scope

- Changing document discovery or the fixed filesystem boundary.
- Changing `--scope repository|target` or its default.
- Changing direct-file, directory, or current-directory initial selection.
- Removing authored Markdown document links.
- Adding Markdown link rewriting for standalone PlantUML files.
- Adding content search, fuzzy finding, a command palette, or breadcrumbs.
- Adding a persistent Lens process or retargeting an active browser session.
- Bundling, invoking, or configuring `fd`.
- Implementing the VS Code source-link proposal.
- Adding code-file rendering or other source-code review behavior.
- Redesigning document typography, metadata, diagrams, or other controls.
