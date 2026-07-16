# E3: Workspace Usability

Status: completed

## Goal

Make the browser workspace useful for realistic repositories without adding
startup indexing or Markdown editing.

## Risks Addressed

- Large repository responsiveness
- Generated and vendor tree noise
- Browser asset maintainability
- Weak nested-directory navigation
- Diagram errors obscuring Markdown source
- Unclear renderer source-data behavior

## Artifacts To Start

- Browser asset baseline: [`assets/index.html`](../../assets/index.html) -
  separates the client from the Rust server source.

## Artifacts To Refine

- Feature brief: [`docs/features/lens-viewer.md`](../features/lens-viewer.md) -
  refined with E3 workspace, Markdown, and renderer decisions.
- Domain model `DM-01`: canonical notes in the feature brief - clarified the
  document/block/diagram relationships used by the client.

## Artifacts Consulted

- E2 results: [`e2-runtime-hardening.md`](e2-runtime-hardening.md).
- `UC-02` and `UC-03`, including their failure extensions.

## Decisions To Record

- Keep directory reads lazy and skip the default generated/vendor directory
  names without recursively scanning them.
- Compile the static browser asset into the binary with `include_str!`.
- Render Markdown PlantUML blocks in document order with source toggles and
  source-preserving errors.
- Document the default public renderer endpoint and provide CLI/environment
  configuration for private or local endpoints.

## Trace

- `UC-02` -> lazy `listDirectory(path)` -> filtered tree and breadcrumbs
- `UC-03` -> `fileContent(plantumlBlockMetadata)` -> inline diagram cards
- `UC-03` -> renderer failure -> visible error with source preserved

## Exit Criteria

- Directory enumeration remains request-lazy and skips common generated/vendor
  directories.
- The browser supports root, nested, parent, and breadcrumb navigation.
- Markdown PlantUML blocks render in source order with source/diagram toggles.
- Incomplete blocks remain visible as source and are marked incomplete.
- Renderer data flow is documented with a private-endpoint alternative.
- Static asset delivery and all prior runtime tests pass.

## Results

All exit criteria passed. The browser client is now an embedded static asset
with responsive navigation, selected-file state, breadcrumbs, parent navigation,
and inline Markdown PlantUML cards. Directory reads remain lazy, and default
generated/vendor trees are filtered before their contents are examined.

Verification:

- `cargo test --locked`: 11 passed.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: passed.
- `cargo fmt --check`: passed.
- Static asset delivery and PlantUML block metadata are covered by server tests.

Residual risks are richer Markdown rendering, configurable ignore files,
large-file performance beyond the response cap, and production frontend asset
packaging.

## Artifact Outcomes

- started: browser asset baseline - [`assets/index.html`](../../assets/index.html)
  - compiled into the binary and covered by the root route test.
- refined: `Lens Viewer` - current source of truth for E3 behavior and privacy.
- refined: `DM-01` - document and PlantUML block behavior clarified.
- started: `E3: Workspace Usability` - this file - closed with implementation and
  verification evidence.
- deferred: `.gitignore`-compatible custom ignore rules, richer Markdown AST
  rendering, and distributable frontend asset packaging.
