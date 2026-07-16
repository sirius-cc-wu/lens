# E4: Document Fidelity

Status: completed

## Goal

Keep Lens responsive and privacy-explicit for large or repository-specific
content while improving Markdown document fidelity.

## Risks Addressed

- Fixed-only generated-tree filtering
- Whole-file JSON responses for large files
- Markdown content being displayed as undifferentiated source
- Implicit upload of PlantUML source to a public renderer
- Missing packaging evidence for the extracted browser asset

## Artifacts To Start

- No new durable design artifact. The feature brief and domain model remain the
  canonical sources for this iteration.

## Artifacts To Refine

- Feature brief: [`docs/features/lens-viewer.md`](../features/lens-viewer.md) -
  refined with chunked files, `.lensignore`, Markdown fidelity, and privacy.
- Browser asset: [`assets/index.html`](../../assets/index.html) - refined with
  Markdown content rendering and large-file continuation.

## Artifacts Consulted

- E3 results: [`e3-workspace-usability.md`](e3-workspace-usability.md).
- `UC-02`, `UC-03`, `C-02`, and `DM-01` in the feature brief.

## Decisions To Record

- Load `.lensignore` from the workspace root and apply gitignore-style glob
  rules during listing without recursively scanning ignored directories.
- Allow explicit reads of ignored paths so user-selected targets remain
  accessible.
- Serve large files in bounded line chunks and let the browser request more.
- Render basic Markdown structure safely without injecting HTML.
- Make remote renderer configuration explicit; do not select a public endpoint
  by default.

## Trace

- `UC-02` -> `.lensignore` and chunked `readFile(path, range)`
- `UC-03` -> safe Markdown structure -> inline PlantUML cards
- `UC-03` -> explicit renderer configuration -> source privacy boundary

## Exit Criteria

- `.lensignore` patterns filter directory listings while explicit reads work.
- Large files do not produce one whole-file JSON response.
- The browser can request subsequent file chunks.
- Markdown headings, paragraphs, links, and non-PlantUML fences render safely.
- PlantUML source remains visible when rendering fails.
- No remote renderer is selected without explicit configuration.
- The project packages with the embedded browser asset.

## Results

All exit criteria passed. Lens now loads `.lensignore`, applies built-in and
project patterns lazily, supports explicit ignored-file reads, chunks large
files at 256 KiB or 1000 lines, and renders basic Markdown structure without
HTML injection. The renderer is unconfigured by default, eliminating implicit
remote source upload.

Verification:

- `cargo test --locked`: 13 passed.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: passed.
- `cargo fmt --check`: passed.
- `cargo package --allow-dirty --no-verify`: passed.

Residual risks are full Markdown CommonMark compatibility, `.gitignore`
import support, resumable line chunks beyond the current UI, and release
artifact testing across platforms.

## Artifact Outcomes

- refined: `Lens Viewer` - current source of truth for E4 behavior, policy, and
  privacy.
- refined: browser asset - safe Markdown rendering and chunk continuation.
- started: `E4: Document Fidelity` - this file - closed with implementation and
  verification evidence.
- deferred: CommonMark-complete rendering, `.gitignore` import, and release
  matrix packaging tests.
