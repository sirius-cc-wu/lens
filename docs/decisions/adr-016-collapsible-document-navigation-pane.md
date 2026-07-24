---
type: "Architecture Decision"
title: "ADR-016: Persist Navigation Pane Visibility in the Browser Tab"
description: "Keeps navigation-pane visibility as accessible browser-tab presentation state without changing authorization."
id: "ADR-016"
status: "accepted"
date: "2026-07-19"
tags: [architecture, decision, navigation]
---

# ADR-016: Persist Navigation Pane Visibility in the Browser Tab

Status: accepted

Date: 2026-07-19

## Context

The authorized document navigation pane helps users move through a Lens viewing
session, but it also consumes horizontal space needed for long documents,
diagrams, and code examples. Proposal 12 requires a user to hide and restore
that pane without turning a presentation choice into a change to the session's
document authorization or routes.

## Decision

Each rendered document with a navigation pane includes a button outside the
pane. Once the application script loads, the button is visible, keyboard
operable, names the pane with `aria-controls`, and reports its current exposed
state with `aria-expanded`. Activating it hides or restores the pane, updates
the button label, and gives the document column the released horizontal space.
The restore button stays visible when the pane is hidden.

Lens remembers only this presentation choice in the browser tab's short-lived
storage for the current loopback site (`sessionStorage`). The stored value is a
Boolean collapsed state. It survives ordinary document navigation and reloads
within that browser tab, but a different tab or a later Lens process begins
with the pane visible. If that storage is unavailable, the button still changes
the current page and later pages use the visible default.

The server does not receive this preference. `ViewerState`, `DocumentCatalog`,
the discovered document set, and all known-document routes remain unchanged.
Without browser scripting, Lens keeps the pane visible and leaves the control
hidden from the page, preserving native document navigation rather than
exposing a button that cannot act.

## Consequences

- The document can use additional browser width without sacrificing a
  keyboard-reachable way to restore navigation.
- Assistive technology receives the pane relationship and its exposed state
  through the button semantics; a hidden pane is removed from the accessibility
  tree.
- No new browser route, server request, filesystem lookup, or authorization
  rule is introduced.
- The preference is intentionally local to a browser tab rather than shared
  across tabs, processes, or users.

## Alternatives Considered

- Store the visibility in `ViewerState`: rejected because a visual preference
  does not belong to the authorized document session and would require a new
  mutation path.
- Add a query parameter to document routes: rejected because it would clutter
  document links and make presentation state appear to affect navigation.
- Use a native disclosure element without script: rejected because it cannot
  retain the choice across a server-rendered document change while also giving
  the collapsed layout its recovered width.

## Trace

- Proposal: [Collapsible Document Navigation Pane](../improvement-proposals.md#12-collapsible-document-navigation-pane)
- Requirements: [`FEAT-02`](../features/document-navigation-pane/use-cases.md)
- Authorization basis: [ADR-003](adr-003-document-root-discovery.md)
- Iteration: [`C6`](../iterations/c6-collapsible-document-navigation-pane.md)
