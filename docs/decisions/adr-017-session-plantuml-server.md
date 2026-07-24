---
type: "Architecture Decision"
title: "ADR-017: Use One Session-Fixed PlantUML Server"
description: "Removes renderer modes and fixes one environment-configured PlantUML server destination for each viewing session."
id: "ADR-017"
status: "accepted"
date: "2026-07-24"
supersedes: [ADR-009]
supersedes_in_part: [ADR-005, ADR-011]
tags: [architecture, decision, plantuml]
---

# ADR-017: Use One Session-Fixed PlantUML Server

Status: accepted

Date: 2026-07-24

## Context

Lens currently selects public-server, local-command, or disabled rendering
through `--renderer`. These modes make the CLI, exported Rust API, viewing
state, document markup, diagram requests, and failure tests vary even though
their main distinction is the last rendering step.

The installed `plantuml` command is an additional runtime dependency whose
version commonly differs across user machines. A server destination can be
self-hosted when source must stay within a controlled environment while
centralizing the PlantUML version used by all Lens sessions pointed at it.

The in-page disable control cannot prevent initial disclosure because diagram
requests begin while the document page loads. Keeping it as a privacy control
would overstate the protection it provides.

## Decision

Lens has one server-based PlantUML rendering path. It removes
`--renderer public|local|disabled`, the exported `RendererMode`, the local
`plantuml -tsvg -pipe` process path, and the browser's session-disable control
and route. The public Rust entry point becomes `serve(target)`.

At viewing-session startup, Lens reads `LENS_PLANTUML_SERVER`, trims surrounding
whitespace and trailing `/` characters, and fixes the result as the session's
server base URL. An absent or normalized-empty value selects
`https://www.plantuml.com/plantuml`. A non-empty value selects only that server.
The browser can neither read nor replace the configured URL.

Each diagram URL continues to append `/svg/<encoded-source>` to the fixed base
URL. A configured-server failure follows the existing visible-source and
per-diagram retry path. It never falls back to the public default. Existing
ten-second request timeouts, 2 MiB response limits, and SVG checks remain.

## Consequences

- Zero-configuration sessions continue to send PlantUML source to the public
  PlantUML server.
- Users who require a controlled destination can set
  `LENS_PLANTUML_SERVER` to a self-hosted or private server for the entire
  viewing session.
- Lens no longer provides an offline local-command renderer or a no-rendering
  startup mode. Users must configure an appropriate server before opening
  source that cannot be sent to the public service.
- CLI users must remove `--renderer`; library callers must remove
  `RendererMode` and the corresponding `serve` argument.
- Browser routes retain fixed document and diagram identifiers and gain no
  source, command, or URL input.
- `LENS_PLANTUML_SERVER` becomes supported product configuration while
  remaining the deterministic controlled-server seam for browser tests.

## Alternatives Considered

- Keep public and disabled renderer modes: rejected because it preserves the
  mode-bearing CLI and API for a path that produces no diagram.
- Keep the in-page disable control: rejected because it cannot stop requests
  that begin before the control is usable.
- Add `--plantuml-server`: rejected because it duplicates the environment
  configuration and requires precedence rules.
- Fall back to the public server: rejected because it would send source to a
  destination the user did not select after a configured-server failure.

## Supersession

- This decision supersedes [ADR-009](adr-009-selectable-plantuml-rendering.md)
  in full.
- It supersedes ADR-005's characterization of `LENS_PLANTUML_SERVER` as a
  test-only hook, while retaining
  [controlled-server browser testing](adr-005-controlled-renderer-browser-tests.md).
- It supersedes ADR-011's renderer-mode status and session-disable behavior,
  while retaining the [per-diagram failure and retry
  behavior](adr-011-diagram-failure-controls.md).

## Trace

- Proposal:
  [`PROP-REMOVE-RENDERER`](../proposals/remove-renderer.md)
- Requirements: `UC-01` and `UC-10` in
  [`FEAT-01`](../features/markdown-viewing/use-cases.md)
- Operation contract:
  [`OC-05`](../features/markdown-viewing/oc-05-request-diagram.md)
- Design:
  [`RZ-05` and `DCD-04`](../features/markdown-viewing/server-rendering-design.md)
- Risks: `R-01` and `R-10` in
  [`docs/risk-list.md`](../risk-list.md)
- Iteration:
  [`E4`](../iterations/e4-server-only-plantuml-rendering.md)
- Implementation:
  [`C7`](../iterations/c7-server-only-plantuml-rendering.md)
