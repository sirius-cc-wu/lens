---
type: "Use Case Model"
title: "FEAT-04: Open Documents Through a Background Lens Service"
description: "Defines how repeated Lens commands open isolated browser views without retaining control of the invoking terminal."
id: "FEAT-04"
status: "active"
scope: "Lens"
tags: [requirements, use-case, process-lifecycle]
---

# FEAT-04: Open Documents Through a Background Lens Service

Lens currently keeps each `lens <target>` process in the foreground because
that process owns the target's local HTTP server. A developer must therefore
leave one terminal occupied and use another terminal to open more documents.
This feature makes the ordinary command a short-lived request while one local
background Lens process keeps the resulting browser views available.

The important boundary is process reuse without authorization reuse. One
background process may host views for several repositories, but every viewing
session retains its own fixed document root, discovered document set, target
scope, and PlantUML server selection.

## System Boundary

Lens is the system under discussion. It includes the command invoked by the
developer and the background Lens process that keeps browser-facing sessions
available. The operating system supplies process and browser-launch facilities,
and the browser displays Lens responses outside the system boundary.

## Actors

| Actor | Goal |
|---|---|
| Developer or technical writer | Open several documentation targets from one shell without leaving a Lens command in the foreground. |
| Operating system browser | Display each requested Lens view while its owning viewing session remains available. |

## Use-Case List

| ID | Use case | Priority |
|---|---|---|
| `UC-11` | Open a target without occupying the terminal | High |

`UC-11` changes command and process lifetime around the target resolution and
viewing behavior already specified by `FEAT-01`. It does not broaden which
files a viewing session may read or serve.

## UC-11: Open a Target Without Occupying the Terminal

Primary actor: Developer or technical writer

Goal: Open a supported target in a browser alongside existing Lens documents,
then continue using the same terminal.

Trigger: The developer runs `lens`, `lens <target>`, or the corresponding
`--scope` form.

Main success scenario:

1. The developer asks Lens to open a target.
2. Lens validates and resolves the target under the existing target and scope
   rules.
3. Lens makes a background Lens process available without requiring a separate
   server-start command.
4. Lens establishes or selects an isolated viewing session for the request.
5. Lens acknowledges the request after the target's browser-facing URL is
   ready to serve.
6. Lens asks the operating system browser to open that view alongside existing
   Lens documents.
7. Lens returns control to the invoking terminal without waiting for the
   browser view to close.
8. The developer continues using the terminal while the requested view remains
   available.

Extensions:

- 2a. If the target is missing, unreadable, hidden, symbolic, unsupported, or
  has no discoverable documents, Lens reports the existing actionable target
  error and opens no browser view.
- 3a. If Lens cannot start or reach the background process within a bounded
  wait, the command reports the startup or communication failure and does not
  claim that the request succeeded.
- 3b. If a previous background process stopped or left stale discovery state,
  Lens recovers that state or replaces the process before accepting the
  request; the developer does not run a cleanup or start command.
- 4a. If the target belongs to another repository, scope, or session
  configuration, the same background process creates another isolated viewing
  session rather than merging authorized document sets.
- 4b. If the target belongs to a compatible existing viewing session, Lens may
  reuse that session, but the requested target remains the document opened by
  this command.
- 6a. If the operating system cannot open the browser automatically, Lens
  reports the local URL for manual opening while leaving the accepted viewing
  session available.
- 8a. If a diagram request or another browser-time operation fails after the
  command succeeds, the browser view reports that failure under the existing
  viewing-session rules.

## Special Requirements

- One ordinary `lens` command must not remain in the foreground merely to keep
  its browser view available.
- The command may wait for a bounded acknowledgment that the request was
  accepted and its browser view is available. It must not wait for that view to
  close.
- Concurrent commands must either reach the one background Lens process for
  the current operating-system user or receive an actionable failure; they must
  not silently create competing owners or lose an accepted request.
- A viewing session remains the authorization and configuration boundary. Its
  canonical document root, fixed document set, target scope, source-link rules,
  and normalized PlantUML server selection must not leak into another session.
- The background process and its command channel remain local to the current
  user. Merely reaching a loopback port does not authorize a different
  operating-system user. This feature does not add remote serving, shared
  hosting, or a non-loopback browser listener.
- The first command may take longer because it starts the background process.
  Later command acknowledgment should add no perceptible delay; construction
  must establish a measurable threshold rather than treating this phrase as a
  timing guarantee.
- The background process may remain idle or stop after its last viewing session
  is no longer needed. No specific idle shutdown policy is required for this
  feature.

## Trace

- Existing target and viewing behavior: [`FEAT-01`](../markdown-viewing/use-cases.md)
- Cross-cutting constraints: [supplementary specification](../../supplementary-specification.md)
- Selected risks: `R-06`, `R-11`, and `R-12` in the [risk list](../../risk-list.md)
- Inception boundary: [S1 background service scope](../../iterations/s1-background-viewer-service-scope.md)
- System interaction: [`SSD-07`](ssd-07-request-target-view.md)
- Operation contract: [`OC-07`](oc-07-request-target-view.md)
- Planned responsibility and Rust design: `RZ-04` and `DCD-04` in S3

## Open Questions for Elaboration

- Which local command-channel mechanism provides cross-platform per-user
  discovery, single-process startup, stale-state recovery, and request
  authentication?
- Which command-side and background-process responsibilities preserve manual
  browser-launch guidance and session-fixed `LENS_PLANTUML_SERVER` behavior?
- What measured acknowledgment threshold distinguishes normal reuse from a
  stalled background process?
