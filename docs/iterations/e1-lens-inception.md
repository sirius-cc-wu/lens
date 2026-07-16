# E1: Lens Inception

Status: active

## Goal

Reduce uncertainty about the smallest standalone Lens slice that proves the
core workflow: start from a target codebase, open a browser workspace, safely
browse content, and render a PlantUML block in Markdown.

## Risks Addressed

- Local CLI, server, and browser lifecycle
- Safe target resolution and workspace path containment
- Markdown PlantUML block extraction
- PlantUML renderer integration and failure handling
- Runtime and packaging suitability

## Artifacts To Start

- Feature brief: [`docs/features/lens-viewer.md`](../features/lens-viewer.md) -
  establishes the product boundary, actors, use cases, MVP boundary, and risk
  list.
- Use cases `UC-01` through `UC-03`: canonical source in the feature brief -
  expresses the first user goals before internal design.

## Artifacts To Refine

- None yet. Refine the feature brief when the stack spike or fixture behavior
  changes the scope or use-case extensions.

## Artifacts Consulted

- [`docs/user-prompts.md`](../user-prompts.md) - project intent and previously
  stated constraints.
- [`references/obsidian-puml-viewer/README.md`](../../references/obsidian-puml-viewer/README.md)
  - reference capabilities and renderer behavior; reference files are
  read-only.

## Decisions To Record

- Runtime and language choice: deferred until the launcher/file-serving spike.
- PlantUML renderer deployment and configuration: deferred until the renderer
  adapter spike.
- Browser application asset strategy: deferred until the launcher slice.

## Trace

- `UC-01 Start Workspace` -> target resolution, local server, browser launch
- `UC-02 Browse Workspace Content` -> safe file listing and reads
- `UC-03 Render PlantUML Block` -> Markdown extraction, renderer request,
  diagram response

## Exit Criteria

- A fixture repository can be selected by all three target forms: no argument,
  file path, and directory path.
- The local workspace serves only files within the resolved target.
- A Markdown fixture with a PlantUML block renders through a stubbed renderer.
- Renderer and file-access failures are visible without crashing the workspace.
- A stack recommendation is supported by the launcher slice rather than only
  language preference.

## Results

The inception baseline is documented, but implementation evidence is not yet
available. The iteration remains active until the exit criteria are tested.

## Artifact Outcomes

- started: `Lens Viewer` - [`docs/features/lens-viewer.md`](../features/lens-viewer.md)
  - current source of truth for inception requirements and risks.
- started: `E1: Lens Inception` - this file - historical record of the active
  objective and exit criteria.
- deferred: SSDs, operation contracts, domain model, realizations, and design
  class diagram - create them after the startup and rendering slice exposes
  stable system operations and architectural decisions.
