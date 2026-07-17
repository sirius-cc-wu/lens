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
- It opens a browser to display codebase code and documentation.
- Markdown files with PlantUML fenced blocks are rendered.
- Lens must not depend on Obsidian.

## Scope

### Initial Product Scope

- Resolve a file, directory, or current-directory target.
- Present Markdown documents in a browser served by the local Lens process.
- Detect and render fenced `plantuml` blocks in Markdown.
- Provide clear errors for unreadable targets, unsupported files, and failed
  diagram rendering.

### Deferred Scope

- Mermaid or other diagram languages.
- Standalone `.puml` file viewing.
- Markdown or diagram editing, export, zoom controls, and live file watching.
- Authentication, shared hosting, cloud synchronization, or telemetry.
- A full code browser beyond what is needed to navigate documentation. The
  phrase "display the codebase's code and document" needs elaboration before it
  becomes a release commitment.

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
| PlantUML rendering | Feasible through a configured renderer, as demonstrated by the reference plugin's PlantUML, Kroki, and local-server integrations. | Decide the renderer contract and default in `E1`; do not assume an external service is acceptable. |
| Markdown parsing | Feasible with an established CommonMark-compatible library. | Select a library after language and packaging decisions are known. |

No show-stopper has been found, but renderer privacy, packaging, and the exact
meaning of codebase browsing are material risks.

## Success Measures for a First Release

- A user can run `lens README.md` and see the document with its valid PlantUML
  blocks rendered in a supported browser.
- A user can run `lens` in a repository and reach Markdown documentation without
  supplying an explicit target.
- The user is told before diagram source leaves the machine, or can select a
  local renderer that keeps it local.
- No Obsidian installation, vault, or API is required at runtime.
