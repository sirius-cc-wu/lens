---
type: "Architecture Decision"
title: "ADR-020: Emit Validated VS Code Source Links"
description: "Hands qualifying repository files to VS Code with generated platform URLs while keeping filesystem paths out of Lens browser routes."
id: "ADR-020"
status: "accepted"
date: "2026-07-26"
tags: [architecture, decision, navigation, security, vscode]
---

# ADR-020: Emit Validated VS Code Source Links

Status: accepted

Date: 2026-07-26

Refines the post-V1 source-viewing scope deferred by
[ADR-004](adr-004-v1-release-scope.md) and uses the active session root selected
by [ADR-019](adr-019-repository-scoped-target-sessions.md).

## Context

Repository documentation frequently links to manifests, fixtures, scripts, and
implementation files. Lens currently rewrites only discovered Markdown and
PlantUML targets. Selecting another relative file eventually reaches Lens's
unavailable-document guidance, interrupting the documentation-to-code workflow.

Serving source through Lens would broaden its content and authorization model.
Accepting a path in a browser route would let browser input participate in
filesystem resolution and, if paired with process execution, local editor
launch. Lens instead needs a narrow user-selected handoff that preserves its
fixed-root security boundary and documentation-only HTTP surface.

## Decision

While rendering a known Markdown document, Lens may replace a qualifying
relative file destination with:

```text
vscode://file/{percent-encoded canonical absolute path}
```

Lens derives and validates the target before emitting the URL. It separates
query and fragment suffixes from the filesystem path; decodes valid path
escapes; rejects absolute, malformed, hidden, symbolic, missing, unreadable,
directory, and root-crossing candidates; requires a regular file; canonicalizes
the result; and proves that it remains beneath the session's canonical document
root.

Known Markdown and PlantUML identifiers retain their Lens routes and take
precedence over editor handoff. External URLs, authored `vscode:` destinations,
and same-document fragments retain their authored destinations. Generated
editor links omit authored suffixes because line-and-column translation is not
part of this decision.

Canonical path serialization uses forward slashes in the URL, preserves the
colon following a Windows drive letter, and percent-encodes spaces, non-ASCII
text, URL delimiters, and other bytes that are not safe inside path segments.
The stable VS Code scheme is `vscode:`; Lens does not automatically select the
distinct `vscode-insiders:` scheme.

Every generated editor link contains visible text stating that it opens in VS
Code. The text remains inside the link so assistive technology receives the
same indication.

Lens adds no HTTP route for source paths or source contents and starts no editor
process. Rendering, refresh, hover, and prefetching only produce markup.
Selecting the ordinary link is the sole trigger, and the browser and operating
system retain confirmation and application-launch control.

## Consequences

- Users can move directly from repository documentation to a qualifying local
  file in VS Code.
- The absolute local path is present in page markup and visible to the browser.
  Lens does not send it to a remote service.
- VS Code is optional. A missing or unregistered handler causes a browser-level
  failure without affecting the Lens session.
- The source resolver needs filesystem checks during initial rendering and
  after a known Markdown document changes.
- Repositories that wanted a relative regular-file link to retain browser
  handling must use an explicit external URL.
- Source position fragments, VS Code Insiders, configurable editors, source
  serving, and folder or workspace opening remain outside this decision.

## Alternatives Considered

### Add a loopback source-opening route

This makes a browser-supplied path part of filesystem authorization and editor
launch. It would require a larger request-authentication, origin, replay, and
command-construction design.

### Spawn the `code` command

Executable names and locations vary, and a browser request would need to launch
a local process. The registered URL handler keeps this authority with the
browser and operating system.

### Render source files in Lens

Source serving introduces media types, encodings, size limits, syntax
presentation, refresh rules, and a broader content authorization model that
the editor-handoff goal does not require.

### Use a file-extension allowlist

Extensions do not establish authorization and would omit extensionless scripts,
manifests, fixtures, and future languages. Location, filesystem type, and
visibility are the relevant rules.

## Trace

- Proposal:
  [`PROP-OPEN-SOURCE-LINKS-IN-VSCODE`](../proposals/open-source-links-in-vscode.md)
- Use case: [`UC-06`](../features/markdown-viewing/use-cases.md)
- System sequence:
  [`SSD-06`](../features/markdown-viewing/ssd-06-open-source-link.md)
- Contract:
  [`OC-06`](../features/markdown-viewing/oc-06-request-document-source-links.md)
- Design:
  [`RZ-06` and `DCD-05`](../features/markdown-viewing/source-link-design.md)
- Risks: [`R-02`, `R-03`, and `R-04`](../risk-list.md)
- Design iteration:
  [`D5`](../iterations/d5-validated-vscode-source-link-design.md)
