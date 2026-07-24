---
type: "Release Notes"
title: "Pending Lens Release Notes"
description: "Records user-visible changes awaiting the next Lens release."
status: "pending"
tags: [release]
---

# Pending Lens Release Notes

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
[implemented proposal](proposals/remove-renderer.md), and the
[C7 construction record](iterations/c7-server-only-plantuml-rendering.md).
