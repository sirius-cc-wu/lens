# Lens Vision and Business Case

## Problem

Developers and technical writers need a quick way to inspect a repository's
Markdown documentation and its PlantUML diagrams without opening the project
in Obsidian or manually invoking a diagram renderer. Existing editor plugins
solve part of this problem but tie viewing to a particular host application.

## Product Vision

Lens is a standalone CLI that opens a browser-based view of a target Markdown
file or a codebase's documentation. It renders Markdown and its PlantUML fenced
blocks so readers can move between source documentation and its diagrams in one
local view.

## Stakeholder Outcome

The primary user can run one command from a repository, see its Markdown
documents in a browser, and understand diagrams without installing or using
Obsidian. The product should be useful for code reviews, architecture discovery,
and locally maintained technical documentation.

## Confirmed Requirements

- The CLI command is named `lens`.
- It accepts a file target, a directory target, or no target; no target means
  the current repository.
- It opens a browser to display repository documentation.
- Markdown files with PlantUML fenced blocks are rendered.
- Lens must not depend on Obsidian.

## Scope

### Initial Product Scope

- Resolve a file, directory, or current-directory target.
- Present Markdown documents in a browser served by the local Lens process.
- Detect and render fenced `plantuml` blocks in Markdown.
- Send PlantUML block source to the public PlantUML server at
  `https://www.plantuml.com/plantuml` for rendering.
- Provide clear errors for unreadable targets, unsupported files, and failed
  diagram rendering.
- Support Linux browser launch through `xdg-open`.

### Deferred Scope

- Mermaid or other diagram languages.
- Standalone `.puml` file viewing.
- Markdown or diagram editing, export, zoom controls, and live file watching.
- Authentication, shared hosting, cloud synchronization, or telemetry.
- Repository source-code browsing. V1 is documentation-only; a later release
  needs a separate use case and authorization model for source files.
- macOS and Windows releases.

## Business Case

Lens replaces a multi-step, host-application-specific workflow with a local CLI
command. Its value is faster documentation review and lower friction for teams
whose repositories contain Markdown and PlantUML. The first release is justified
if it can render common project documentation safely, reliably, and with a setup
burden lower than installing an editor plugin.

## Feasibility and Alternatives

| Question | Inception assessment | Follow-up |
|---|---|---|
| Build or buy | Build a thin standalone CLI and browser viewer. The reference Obsidian plugin is not reusable as a product dependency because Lens must not depend on Obsidian. | Reuse only portable ideas or permissively licensed libraries after license review. |
| Browser presentation | Feasible with a loopback HTTP server and the platform browser-launch facility. | Prove target resolution, server lifetime, and browser launch in `E1`. |
| PlantUML rendering | V1 uses the public PlantUML server at `https://www.plantuml.com/plantuml`. This accepts that PlantUML source leaves the machine. | Validate request construction, response handling, and failure behavior in `E1`. See [ADR-001](decisions/adr-001-public-plantuml-rendering.md). |
| Markdown parsing | Feasible with an established CommonMark-compatible library. | Select a library after language and packaging decisions are known. |

No show-stopper has been found, but public-renderer availability and Linux
packaging verification remain material risks.

## Success Measures for a First Release

- A user can run `lens README.md` and see the document with its valid PlantUML
  blocks rendered in a supported browser.
- A user can run `lens` in a repository and reach Markdown documentation without
  supplying an explicit target.
- A failed PlantUML request leaves its source visible with an actionable error
  while the rest of the document remains readable.
- No Obsidian installation, vault, or API is required at runtime.
- The Linux package installs with Cargo and opens the viewer through `xdg-open`
  or reports a manual loopback URL.
