---
type: "Release Notes"
title: "Pending Lens Release Notes"
description: "Records user-visible changes awaiting the next Lens release."
status: "pending"
tags: [release]
---

# Pending Lens Release Notes

## Open Repository File Links in VS Code

Relative links to existing visible regular files inside the viewing session's
fixed document root now open their canonical local path through VS Code's
stable `vscode:` URL handler. Generated links visibly state
**(opens in VS Code)** and correctly encode spaces and non-ASCII path text.

Known Markdown and PlantUML documents continue to open inside Lens. Hidden,
symbolic, missing, directory, absolute, and out-of-root targets receive no
generated editor URL. Lens adds no source-content route and launches no editor
process.

VS Code remains optional, and a browser may request confirmation before opening
it. The separate `vscode-insiders:` scheme is not selected automatically. See
the [README source-link guidance](../README.md#vs-code-source-links),
[ADR-020](decisions/adr-020-validated-vscode-source-links.md), and the
[C9 transition record](iterations/c9-accessible-source-link-handoff.md).

## Breaking: One PlantUML Server per Viewing Session

Lens now has one server-based PlantUML rendering path. The command-line
`--renderer public|local|disabled` option has been removed, and passing any
`--renderer` form is an unknown-argument error.

The equivalent startup commands are:

```text
lens --renderer public docs    -> lens docs
lens --renderer local docs     -> LENS_PLANTUML_SERVER=<server> lens docs
lens --renderer disabled docs  -> no startup-time equivalent
```

Users who previously selected `local` should run or choose a private PlantUML
server and set `LENS_PLANTUML_SERVER` to its base URL. Lens uses
`https://www.plantuml.com/plantuml` when that variable is missing, blank, or
whitespace-only. A configured server failure is shown in the document and
never falls back to the public server.

The exported Rust `RendererMode` type has also been removed. Library callers
must change `serve(target, renderer_mode)` to `serve(target)`.

Per-diagram source visibility and retry remain available after a server
failure. The in-page rendering-disable control and `/renderer/disable` route
have been removed.

See the [README migration guidance](../README.md#plantuml), the
[accepted server decision](decisions/adr-017-session-plantuml-server.md), and the
[C7 construction record](iterations/c7-server-only-plantuml-rendering.md).
