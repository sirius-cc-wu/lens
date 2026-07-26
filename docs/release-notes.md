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
[ADR-021](decisions/adr-021-validated-vscode-source-links.md), and the
[C9 transition record](iterations/c9-accessible-source-link-handoff.md).

## Changed: Select Documents Before Starting Lens

Lens document pages now dedicate their width to the selected Markdown document
or PlantUML diagram. The discovered-document catalog, identifier search,
pagination, current-result marker, **Hide documents** / **Show documents**
control, and tab-local pane visibility state have been removed.

Coding agents and command-line tools now perform generic document discovery and
selection before Lens starts:

```bash
lens docs/features/markdown-viewing/use-cases.md
lens "$(fd --type file --full-path '<file-pattern>' .)"
```

The optional `fd` example uses POSIX-shell command substitution and must produce
exactly one path. Lens does not install, invoke, or require `fd`; literal paths,
`find`, `rg`, fuzzy finders, shell completion, and other selectors remain valid.

This is a user-interface compatibility change. Unlinked documents are no
longer visible through an in-browser catalog and require a new direct Lens
invocation or a known URL during the active session. Authored Markdown links,
browser history, direct Markdown and PlantUML targets, repository and target
scope, automatic refresh, and known document routes remain unchanged. Former
`query` and `page` parameters are ignored for known routes.

See [ADR-020](decisions/adr-020-focused-document-review.md),
[R2 construction evidence](iterations/r2-remove-document-navigation-pane.md),
[R3 transition evidence](iterations/r3-focused-document-review-transition.md),
and the [focused review checks](release-readiness.md#focused-review-checks).

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
