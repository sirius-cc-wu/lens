---
type: "Operation Contract"
title: "OC-05: Request a Diagram"
description: "Specifies the server-selection, bounded-response, and no-fallback guarantees for one authorized diagram request."
id: "OC-05"
operation: "request_diagram(diagram_id)"
traces: [UC-01, UC-10, SSD-01, ADR-017]
status: "active"
tags: [analysis, operation-contract, plantuml]
---

# OC-05: Request a Diagram

Operation: `request_diagram(diagram_id)`

Cross References: `UC-01`, `UC-10`,
[SSD-01](ssd-01-open-markdown-target.md), and
[ADR-017](../../decisions/adr-017-session-plantuml-server.md)

Scope: Lens

Preconditions:

- A viewing session exists.
- `diagram_id` identifies a diagram derived from a document in that session's
  authorized document set.
- The session fixed one normalized PlantUML server base URL when it started.

Postconditions on success:

- Exactly the source already associated with `diagram_id` was requested from
  the session-fixed PlantUML server.
- The response was no larger than 2 MiB and satisfied the SVG content checks.
- The diagram response was returned without changing its source, server
  destination, or viewing-session authorization.
- No other PlantUML server was contacted.

Postconditions on failure:

- The diagram source remained available to the browser as a visible fallback.
- The failure was associated with the requested diagram and remained eligible
  for a retry through the same Lens-owned route.
- No default-server request was made when the session used a configured server.
- No repository document, server selection, or viewing-session authorization
  changed.

Special Requirements:

- A server request times out after ten seconds.
- Browser input cannot provide PlantUML source, a server URL, or an executable
  command.
