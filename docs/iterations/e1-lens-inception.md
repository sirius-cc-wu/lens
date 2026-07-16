# E1: Lens Inception

Status: completed

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

- Feature brief: [`docs/features/lens-viewer.md`](../features/lens-viewer.md) -
  refined with SSD summaries, operation contracts, and implementation evidence.

## Artifacts Consulted

- [`docs/user-prompts.md`](../user-prompts.md) - project intent and previously
  stated constraints.
- [`references/obsidian-puml-viewer/README.md`](../../references/obsidian-puml-viewer/README.md)
  - reference capabilities and renderer behavior; reference files are
  read-only.

## Decisions To Record

- Runtime and language choice: [ADR-001](../decisions/adr-001-rust-runtime.md) -
  Rust is recommended for the MVP runtime.
- PlantUML renderer deployment and configuration: keep the adapter replaceable;
  the current contract is a configurable POST endpoint returning SVG.
- Browser application asset strategy: the spike embeds a minimal HTML client;
  production assets remain an elaboration concern.

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

The vertical slice met the inception exit criteria:

- Rust CLI target resolution supports no argument, directory, and file forms.
- A local HTTP workspace serves a minimal browser UI plus tree and file APIs.
- Canonical path checks reject traversal and skip symlinks outside the target.
- Markdown PlantUML fences are extracted and rendered through an injected
  adapter; renderer failures return HTTP 502 without stopping the server.
- `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, and
  the CLI startup smoke test pass.

Residual risks are graceful signal handling, production-grade HTTP behavior,
renderer response validation, and a richer browser client.

## Artifact Outcomes

- started: `Lens Viewer` - [`docs/features/lens-viewer.md`](../features/lens-viewer.md)
  - refined with SSD summaries, contracts, and the validated MVP boundary.
- started: `ADR-001` - [`docs/decisions/adr-001-rust-runtime.md`](../decisions/adr-001-rust-runtime.md)
  - records the runtime recommendation and its evidence.
- started: `SSD-01` through `SSD-03` and `C-01` through `C-02` - canonical
  summaries in the feature brief - system operations are now stable enough for
  the next design iteration.
- started: `E1: Lens Inception` - this file - closed with implementation and
  verification results.
- deferred: detailed domain model, realizations, and design class diagram -
  create them when production workspace and renderer responsibilities are
  explored in elaboration.
