# FEAT-01: View Markdown with PlantUML

Status: refined in P5

## Actors

| Actor | Goal |
|---|---|
| Developer or technical writer | Read repository Markdown and its diagrams without opening an editor-specific plugin. |
| Diagram renderer | Depending on the selected session setting, converts PlantUML source through the public server or installed `plantuml` command, or leaves the source visible. |
| Operating system browser | Displays the local Lens view after the CLI starts it. |

## Use-Case List

| ID | Use case | Priority |
|---|---|---|
| `UC-01` | View a Markdown file with PlantUML blocks | High |
| `UC-02` | View repository documentation from the current directory | High |
| `UC-03` | View documentation from a directory argument | High |
| `UC-04` | Navigate between discovered Markdown documents | Medium |
| `UC-05` | Receive a target-resolution or rendering failure | High |
| `UC-06` | View source code associated with documentation | Deferred from V1 |

Inception detailed `UC-01` to validate initial scope without prematurely
specifying the entire product. E2 adds the document-root and navigation detail
for `UC-02` through `UC-04`; ADR-004 defers `UC-06` from V1.

## UC-01: View a Markdown File with PlantUML Blocks

Primary actor: Developer or technical writer

Goal: Read a selected Markdown file and its diagrams in a browser without
Obsidian.

Preconditions:

- The user can execute `lens`.
- The supplied target is a readable Markdown file.
- A supported browser is available.

Trigger: The user runs `lens [--renderer public|local|disabled] <markdown-file>`.

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
- 6b. With the default `--renderer public`, Lens sends PlantUML block source to
  the public PlantUML server.
- 6c. With `--renderer local`, Lens runs the installed `plantuml` command with
  the block source on standard input and keeps it off the renderer network.
- 6d. With `--renderer disabled`, Lens does not request a diagram image and
  instead displays the original PlantUML source with a disabled status.
- 6e. Each document identifies the active renderer. After one diagram fails,
  the user can retry only that diagram without changing its source or renderer
  selection.
- 6f. The user can disable diagram rendering for the active viewing session.
  Lens stops future renderer requests and keeps every diagram's source visible.

Postconditions:

- A local browser view exists for the selected Markdown document.
- The original file remains unchanged.
- The user can identify any diagrams that were not rendered and why.
- The user can determine whether the selected renderer is public, local, or
  disabled for the current viewing session.

E1 scope: The executable slice accepts a direct `.md` or `.markdown` file
target. Directory and current-directory targets remain work for `UC-02` and
`UC-03`.

## UC-02 and UC-03: Open a Document Root

Primary actor: Developer or technical writer

Goal: Open Markdown documentation from the current directory, a directory
argument, or a Markdown file argument.

Trigger: The user runs `lens`, `lens <directory>`, or `lens <markdown-file>`.

Main success scenario:

1. Lens resolves the document root from the supplied target or current
   directory.
2. Lens identifies Markdown documents within the document root.
3. Lens selects the explicitly named file, a root `README` document, a
   `docs/index` document, or the first discovered document as the initial
   document.
4. Lens opens a local viewing session for the document root.
5. The user reads the initial document in the browser.

Extensions:

- 1a. If the target is missing, unreadable, hidden, a symbolic link, or neither
  a directory nor a supported Markdown file, Lens reports an actionable error
  and starts no viewing session.
- 2a. If the document root has no Markdown documents, Lens reports an
  actionable error and starts no viewing session.

Special requirements:

- The document root and discovered documents are canonical paths.
- Symbolic links found during document discovery are excluded.
- Hidden files and directories found during document discovery are excluded.
- A direct hidden or symbolic-link target is rejected before document discovery.
- A direct file target remains the initial document but authorizes its canonical
  parent as the document root.

## UC-04: Navigate Between Discovered Markdown Documents

Primary actor: Developer or technical writer

Goal: Follow a Markdown link to another discovered document without allowing
the link to read files outside the document root.

Preconditions:

- Lens has an active viewing session with a discovered document set.

Main success scenario:

1. The user follows a link in the current document.
2. Lens identifies whether the link targets a discovered Markdown document.
3. Lens displays the target document and its diagrams.

Extensions:

- 2a. If the link target is not a discovered Markdown document, Lens returns a
  Lens-owned 404 guidance page. It does not resolve the requested URL into a
  filesystem path.
- 2b. External links and same-document fragment links retain their standard
  browser behavior.

Special requirements:

- Lens maps links only to the discovered document set. It must not use a link
  URL as a filesystem path or an arbitrary renderer URL.

## Open Questions

- Does "codebase code and document" require a code-file navigator in the first
  release, or only documentation navigation with links to repository files?
- Which Markdown extensions and filenames are in scope?
- What request, response-size, and timeout limits are needed for the public
  PlantUML server?
- Must the viewer work in a browser that is already running, headless
  environments, or remote development containers?

## UML Design Views

- [V1 component, realization, and Rust type diagrams](uml-design.md) (`CMP-01`,
  `RZ-01`, and `DCD-01`)
