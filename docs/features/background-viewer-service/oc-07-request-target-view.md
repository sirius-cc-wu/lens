---
type: "Operation Contract"
title: "OC-07: Request a Target View"
description: "Specifies command completion, viewing-session creation, isolation, and failure effects for one non-blocking Lens invocation."
id: "OC-07"
operation: "request_target_view(target?, invocation_directory, scope, plantuml_server?)"
traces: [UC-11, SSD-07, OC-02, ADR-002, ADR-017]
status: "active"
tags: [analysis, operation-contract, process-lifecycle]
---

# OC-07: Request a Target View

Operation:
`request_target_view(target?, invocation_directory, scope, plantuml_server?)`

Cross References: `UC-11`, [`SSD-07`](ssd-07-request-target-view.md),
[`OC-02`](../markdown-viewing/oc-02-open-document-root.md),
[`ADR-002`](../../decisions/adr-002-loopback-viewer-scope.md), and
[`ADR-017`](../../decisions/adr-017-session-plantuml-server.md)

Scope: Lens

## Effect in Plain Language

Each successful command obtains a new isolated browser-facing viewing session
from one reusable background Lens process. The target keeps the meaning it had
in the invoking shell, the resulting URL is ready before Lens acknowledges the
request, and existing browser views remain unchanged. Reusing the process does
not merge document authority or PlantUML configuration between commands.

## Preconditions

- None. Lens validates the invocation directory, optional target, target scope,
  and optional PlantUML server value as part of the operation.

## Postconditions on Success

- The optional target was interpreted relative to the canonical form of
  `invocation_directory`; an omitted target selected that directory.
- The target and scope rules from `OC-02` produced a canonical document root,
  a fixed discovered document set, and the explicitly or implicitly selected
  initial document.
- The supplied `plantuml_server?` value was normalized under ADR-017. An absent
  or normalized-empty value selected the public default, and a non-empty value
  selected only that server for the new viewing session.
- One new viewing session was created for this accepted request and associated
  with the background Lens process for the current operating-system user.
- The new viewing session owned its canonical document root, fixed document
  set, target scope, source-link authorization, selected initial document, and
  normalized PlantUML server independently of every existing viewing session.
- A loopback browser listener was ready to serve the new viewing session before
  Lens produced `view_ready(view_url)` or
  `manual_open_required(view_url)`.
- `view_url` addressed the selected initial document in the new viewing
  session. It did not contain an actor-supplied filesystem path that a browser
  route would later interpret.
- Lens attempted the operating-system browser handoff exactly once for this
  accepted request.
- The invoking command completed after the handoff attempt and did not wait for
  the browser to request the document or close the resulting view.
- Every viewing session that existed before the operation retained the same
  listener, root, document set, configuration, and browser-visible state.

## Postconditions on Target or Scope Failure

- No viewing session was created for the rejected request.
- No browser handoff was attempted.
- The command completed with the actionable target or scope error already
  defined by `FEAT-01` and `OC-02`.
- Existing viewing sessions remained available and unchanged.

## Postconditions on Startup, Delivery, or Acknowledgment Failure

- The command did not report `view_ready` or otherwise claim that the target
  was accepted.
- No browser handoff was attempted without a returned `view_url`.
- The command completed within a bounded failure interval with an error that
  distinguishes process startup, service discovery, request delivery, or
  acknowledgment timeout where the available evidence permits.
- Stale discovery state did not require a separate user cleanup command.
- Any incomplete internal session state from the failed operation was not made
  reachable through a browser URL and did not change an existing viewing
  session.

## Postconditions on Browser-Handoff Failure

- The accepted viewing session and its `view_url` remained available.
- The command reported `manual_open_required(view_url)` with the same manual
  opening guidance as the current foreground Lens behavior.
- The failure did not stop the background Lens process or another viewing
  session.

## State Rules for Design

- A successful command creates a new viewing session even when another session
  has the same canonical root, scope, and PlantUML server. This preserves the
  current per-invocation document-set snapshot and avoids silently admitting a
  newly created document to an older fixed session.
- A transport retry belonging to one command must not create more than one
  reachable viewing session or more than one browser handoff. A separately
  invoked `lens` command is a new request and intentionally opens another
  browser view.
- The exact idle shutdown policy remains unspecified. If the background
  process is unavailable when a later command begins, the later operation
  follows the same automatic-start behavior as the first command.

## Concept Ownership

The [glossary](../../glossary.md) owns the shared meanings of Lens client,
background Lens service, and viewing session. These technical lifecycle
concepts do not require a standalone domain model; `RZ-04` and `DCD-04` will
assign their software responsibilities in S3.

## Open Issues for Design

- Select a cross-platform, per-user command channel and discovery record that
  can establish one background-process owner, authenticate requests, recover
  stale state, and support bounded startup and acknowledgment.
- Define the request identity and outcome retention needed to make an internal
  transport retry idempotent without suppressing a separately invoked command.
- Assign browser launch to the short-lived client so it retains the invoking
  desktop environment and can report the manual URL, unless S3 finds contrary
  platform evidence.
- Set concrete startup, acknowledgment, and retained-outcome intervals during
  construction from executable failure and timing evidence.
