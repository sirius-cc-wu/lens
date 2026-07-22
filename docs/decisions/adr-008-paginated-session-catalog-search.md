---
type: "Architecture Decision"
title: "ADR-008: Search the Session Document Catalog in Bounded Pages"
description: "Defines bounded, paginated identifier search over the immutable authorized session catalog."
id: "ADR-008"
status: "accepted"
date: "2026-07-19"
tags: [architecture, decision, navigation]
---

# ADR-008: Search the Session Document Catalog in Bounded Pages

Status: accepted

Date: 2026-07-19

## Context

ADR-006 made every document response carry the complete, browser-filtered
catalog of authorized document identifiers. That was a safe first navigation
pane, but its response size and browser work grow with every discovered
document. Proposal 11 needs navigation to remain usable for large document
trees without widening the fixed viewing-session boundary from ADR-003.

## Decision

At session creation, Lens will derive one immutable identifier-search index
from the already authorized document identifiers. A search compares only an
entered query with those identifiers; it never reads document contents, scans
the filesystem, resolves a new path, or changes session membership.

Every document response will render a native `GET` search form whose action is
the current known-document route. The form uses `query` and one-based `page`
parameters. An empty query lists the first lexical page of authorized
identifiers. A non-empty query matches identifier text case-insensitively. Each
response contains at most 50 matching identifiers and native previous/next
links preserve the query and page. The current document is marked only when it
is present in that returned page.

Lens accepts at most 256 UTF-8 bytes of query text. A longer query leaves the
current document readable and reports the limit without searching the index. A
missing, zero, malformed, or out-of-range page becomes the first result page.
The browser submits a search only when the user submits the form or follows a
page link; Lens sends no request for each keystroke and adds no background
search polling or separate search route. There is no independent rate limiter
for this loopback, single-session page request: the fixed in-memory index,
query limit, and 50-item response cap bound the work and output of each search
request.

The existing document route remains the only route that can select a document.
An unknown document identifier keeps the ADR-003 guidance response regardless
of any query or page parameters.

## Consequences

- A response no longer exposes the complete authorized catalog, but every
  returned identifier remains from that exact catalog.
- Ordinary browser navigation works without JavaScript, including submitting a
  search, moving between pages, and opening a returned document.
- The result cap bounds response markup while pagination keeps any authorized
  identifier reachable through deliberate searches or page navigation.
- Search matching is identifier-only, not document-content search. Content
  search remains a separate proposal with its own privacy and performance
  design.
- The index is intentionally session-local and immutable. Its matching cost
  should be measured before promising a supported upper bound for unusually
  large document trees.

## Alternatives Considered

- Keep browser-local filtering and the complete catalog from ADR-006: rejected
  because each response and browser page still scale with the complete set.
- Add a JSON search endpoint with JavaScript-driven requests: rejected because
  it would leave no-JavaScript navigation incomplete and introduce another
  request surface without serving this user goal better.
- Re-scan the document root for each search: rejected because it would violate
  the fixed authorization boundary and make search depend on current filesystem
  state.

## Trace

- Proposal: [Scalable Document Navigation Search](../improvement-proposals.md#11-scalable-document-navigation-search)
- Requirements: [`FEAT-02`](../features/document-navigation-pane/use-cases.md)
- Supersedes: [ADR-006](adr-006-document-navigation-pane.md)
- Authorization basis: [ADR-003](adr-003-document-root-discovery.md)
- Performance risk: [`R-09`](../risk-list.md)
