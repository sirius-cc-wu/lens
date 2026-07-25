---
type: "Architecture Decision"
title: "ADR-020: Delegate Resource Selection and Focus Lens on Document Review"
description: "Delegates generic document discovery to coding agents and command-line tools while retaining fixed-session authorization for routes and authored links."
id: "ADR-020"
status: "accepted"
date: "2026-07-26"
tags: [architecture, decision, navigation, command-line]
---

# ADR-020: Delegate Resource Selection and Focus Lens on Document Review

Status: accepted

Date: 2026-07-26

## Context

Lens is most useful where a shell is weak: rendered Markdown, diagrams,
metadata, readable wide content, safe authored links, visible failures, and
automatic refresh. Its document navigation pane duplicates generic discovery
and selection that coding agents and command-line tools already perform with
more repository context and user-controlled search behavior.

The pane also requires a searchable identifier index, query parsing,
pagination, current-result state, responsive sidebar layout, a visibility
control, and browser-tab storage. Removing those responsibilities must not
weaken ADR-003's fixed viewing-session boundary. Browser requests still need a
safe way to name only documents discovered when the session started, and
relative Markdown links still need that same authorized set.

## Decision

Generic resource discovery and selection happen before Lens starts. A coding
agent or user passes one optional target path directly or composes Lens with a
command-line file selector. Lens does not invoke, bundle, or require `fd`,
`find`, or another search tool, and it does not add a replacement catalog,
command palette, breadcrumb, or previous/next control.

Every successful document response uses one focused reading column. It contains
the document header and rendered content but no discovered-document list,
identifier search, result pagination, current-result marker, pane visibility
control, collapsed-state attribute, or navigation value in browser storage.

The fixed discovered set remains the authorization source. Session creation
builds a minimal known-document lookup: an immutable mapping from each
repository-relative identifier to its document index. Document and revision
routes resolve identifiers only through that lookup. Relative Markdown link
rewriting uses the corresponding immutable identifier set. Neither
responsibility scans the filesystem or treats a browser-supplied identifier as
a path.

The `query` and `page` parameters from the former catalog have no meaning.
Known document routes ignore them and return the same authorized document as a
request without those parameters. Unknown identifiers retain a Lens-owned
not-found page, now described as an unavailable document rather than unavailable
navigation.

Target validation, initial-document selection, direct Markdown and PlantUML
targets, repository and target scope, authored Markdown links, browser history,
known document URLs, diagram rendering, and automatic refresh remain unchanged.
An unlinked document is selected through a new Lens invocation or a known URL
for the active session.

## Consequences

- The document receives the available page width at narrow and wide viewports,
  and Lens no longer maintains catalog-only server, markup, CSS, script, or
  browser state.
- Users give up visible in-browser discovery of unlinked documents. Coding
  agents, literal paths, shell completion, file finders, and repository indexes
  provide that generic selection instead.
- Authored links continue to express meaningful document relationships and use
  normal browser history within the fixed authorized session.
- Standalone PlantUML files remain valid direct targets and known document
  routes, but Lens does not make them visible through a generic browser list.
- The authorization model becomes easier to distinguish from presentation:
  the known-document lookup permits routes, while no searchable catalog is
  exposed.
- Former catalog URLs remain compatible for their document route; obsolete
  query parameters are inert and may disappear on later navigation.

## Alternatives Considered

- Keep the pane collapsed by default: rejected because it retains the catalog,
  toggle, storage, layout branches, and browser interaction while hiding their
  main output.
- Keep only identifier search or replace the pane with a command palette:
  rejected because both preserve generic selection inside Lens and retain an
  additional interaction and authorization-adjacent surface.
- Add previous and next controls: rejected because lexical adjacency does not
  express a meaningful reading relationship; authored links should express
  that relationship.
- Stop discovering documents for direct targets: rejected because discovery
  still authorizes links, routes, diagram sources, and refresh behavior.
- Invoke `fd` from Lens: rejected because it adds an external dependency and a
  second selection path whose roots, ignores, errors, and output would become
  Lens responsibilities.

## Trace

- Proposal: remove the document navigation pane (retired after R3 transition)
- Elaboration: [R1 focused document review boundary](../iterations/r1-focused-document-review-boundary.md)
- Construction: planned R2 navigation-pane removal
- Preserved authorization: [ADR-003](adr-003-document-root-discovery.md)
- Preserved session scope: [ADR-019](adr-019-repository-scoped-target-sessions.md)
- Preserved authored links: `UC-04` in
  [`FEAT-01`](../features/markdown-viewing/use-cases.md)
- Supersedes:
  [ADR-008](adr-008-paginated-session-catalog-search.md) and
  [ADR-016](adr-016-collapsible-document-navigation-pane.md)
