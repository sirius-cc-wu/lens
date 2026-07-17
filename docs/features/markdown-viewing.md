# FEAT-01: View Markdown with PlantUML

Status: inception baseline

## Actors

| Actor | Goal |
|---|---|
| Developer or technical writer | Read repository Markdown and its diagrams without opening an editor-specific plugin. |
| Diagram renderer | Converts PlantUML source to a displayable result. It may be local or remote; the contract is not yet selected. |
| Operating system browser | Displays the local Lens view after the CLI starts it. |

## Use-Case List

| ID | Use case | Priority |
|---|---|---|
| `UC-01` | View a Markdown file with PlantUML blocks | High |
| `UC-02` | View repository documentation from the current directory | High |
| `UC-03` | View documentation from a directory argument | High |
| `UC-04` | Navigate between discovered Markdown documents | Medium |
| `UC-05` | Receive a target-resolution or rendering failure | High |
| `UC-06` | View source code associated with documentation | Unknown; clarify before commitment |

One of six use cases is detailed in inception (16.7%) to validate the initial
scope without prematurely specifying the entire product.

## UC-01: View a Markdown File with PlantUML Blocks

Primary actor: Developer or technical writer

Goal: Read a selected Markdown file and its diagrams in a browser without
Obsidian.

Preconditions:

- The user can execute `lens`.
- The supplied target is a readable Markdown file.
- A supported browser is available.

Trigger: The user runs `lens <markdown-file>`.

Main success scenario:

1. Lens validates and resolves the file target.
2. Lens starts a local browser-facing session for that target.
3. Lens opens the session in the user's browser.
4. Lens parses the Markdown document.
5. Lens recognizes each fenced block labeled `plantuml`.
6. Lens renders the Markdown and replaces each valid PlantUML block with its
   rendered diagram.
7. The user reads the document and diagrams in the browser.

Extensions:

- 1a. If the target is missing, unreadable, or not a supported Markdown file,
  Lens exits with an actionable error and does not start a browser session.
- 3a. If a browser cannot be opened automatically, Lens reports the local URL
  for the user to open manually.
- 5a. Fenced blocks in other languages remain code blocks.
- 6a. If a PlantUML block is invalid or the renderer is unavailable, Lens keeps
  the source visible and shows an error associated with that block. One failed
  diagram does not prevent the rest of the document from rendering.
- 6b. If rendering would send source to a remote service, Lens follows the
  renderer privacy decision established in elaboration before making the
  request.

Postconditions:

- A local browser view exists for the selected Markdown document.
- The original file remains unchanged.
- The user can identify any diagrams that were not rendered and why.

## Open Questions

- Does "codebase code and document" require a code-file navigator in the first
  release, or only documentation navigation with links to repository files?
- Which Markdown extensions and filenames are in scope?
- Is network rendering opt-in, opt-out, or prohibited by default?
- Must the viewer work in a browser that is already running, headless
  environments, or remote development containers?
