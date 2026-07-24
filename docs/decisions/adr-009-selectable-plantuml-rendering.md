---
type: "Architecture Decision"
title: "ADR-009: Select a PlantUML Renderer per Viewing Session"
description: "Defines public, local, and disabled PlantUML renderer choices fixed for each viewing session."
id: "ADR-009"
status: "superseded"
superseded_by: "ADR-017"
date: "2026-07-19"
tags: [architecture, decision, plantuml]
---

# ADR-009: Select a PlantUML Renderer per Viewing Session

Status: superseded by ADR-017

Date: 2026-07-19

## Context

The public PlantUML service keeps the initial Lens path simple but sends diagram
source off the user's machine and makes rendering depend on network service
availability. A user needs an explicit privacy-preserving option without
allowing a browser request to choose an arbitrary command or renderer URL.

## Decision

Lens accepts `--renderer public|local|disabled` when it starts a viewing
session. `public` remains the default and uses ADR-001's server behavior.
`local` runs the installed `plantuml` command with `-tsvg -pipe`, providing the
already authorized diagram source through standard input. `disabled` creates no
diagram image request and leaves the source visible with a session-status
message.

The renderer is fixed in `ViewerState` for the session. Browser routes still
identify only a pre-rendered diagram belonging to an authorized document; they
cannot supply a renderer choice, executable path, source, or URL. Public and
local results retain the ten-second timeout and 2 MiB output limit. A missing,
failed, or invalid local command follows the existing per-diagram failure path.

## Consequences

- Users can keep PlantUML source local when they install PlantUML themselves.
- Public rendering remains zero-configuration for existing users and the
  controlled renderer test environment applies only to that mode.
- Local rendering adds a user-managed executable and runtime dependency.
- Disabled rendering is a predictable no-network fallback, but it does not
  produce a visual diagram.

## Trace

- Proposal: [Local PlantUML Rendering](../improvement-proposals.md#1-local-plantuml-rendering)
- Requirement: `UC-01` in
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md)
- Risk: `R-01` in [`docs/risk-list.md`](../risk-list.md)
- Iteration: [`P1`](../iterations/p1-local-plantuml-rendering.md)
- Successor: [ADR-017](adr-017-session-plantuml-server.md)
