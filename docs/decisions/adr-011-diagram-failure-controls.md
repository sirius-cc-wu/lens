---
type: "Architecture Decision"
title: "ADR-011: Keep Diagram Controls Inside a Viewing Session"
description: "Keeps renderer status, retry, and disable controls within the authority of an existing viewing session."
id: "ADR-011"
status: "partially superseded"
superseded_in_part_by: "ADR-017"
date: "2026-07-19"
tags: [architecture, decision, resilience]
---

# ADR-011: Keep Diagram Controls Inside a Viewing Session

Status: partially superseded by ADR-017

Date: 2026-07-19

## Context

A renderer failure should leave a user with a clear recovery path, and a user
must be able to stop renderer work when the service is unavailable or no longer
appropriate. Those controls must not turn the loopback viewer into an endpoint
for arbitrary renderer URLs, commands, source, or filesystem writes.

## Decision

Every rendered document displays the selected renderer status. A failed diagram
shows a retry button that reloads only its existing Lens-owned image route with
a cache-busting query. Lens continues to resolve that route through the
pre-rendered document and diagram identifier.

The page also offers a disable button while rendering is enabled. It sends a
POST request to a fixed loopback route that sets one atomic in-memory flag in
`ViewerState`. Once set, all diagram routes return a disabled result without
invoking the renderer. The client removes existing image sources, opens the
PlantUML source, and updates the visible status. The transition lasts only for
the current viewing session and cannot be reversed through a browser request.

## Consequences

- Users can distinguish the active renderer, retry transient failures, and stop
  further renderer requests without restarting Lens.
- Session disable does not change document membership, renderer configuration,
  or repository files.
- The fixed POST route is the only browser mutation; it has no user-supplied
  target, command, source, or path.
- Restoring rendering requires starting a new Lens session, which avoids
  silently resuming external work after an explicit stop.

## Current Scope

ADR-017 removes renderer-mode status, the session-disable control, its mutable
route, and its in-memory flag because the page cannot prevent diagram requests
that start during its own load. The per-diagram visible-source failure and
retry behavior remains accepted. A document may describe the active path as
server-based rendering, but it does not expose the configured server URL.

## Trace

- Proposal: [Diagram Failure Controls](../improvement-proposals.md#5-diagram-failure-controls)
- Requirement: `UC-01` in
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md)
- Security boundary: [`docs/supplementary-specification.md`](../supplementary-specification.md)
- Iteration: [`P5`](../iterations/p5-diagram-failure-controls.md)
- Partial successor: [ADR-017](adr-017-session-plantuml-server.md)
