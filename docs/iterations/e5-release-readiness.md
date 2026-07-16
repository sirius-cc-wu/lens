# E5: Release Readiness

Status: completed

## Goal

Make Lens predictable to build, package, and use across supported platforms
while improving CommonMark and large-file compatibility.

## Risks Addressed

- Lightweight Markdown parsing diverging from CommonMark behavior
- Repository rules differing between `.gitignore` and `.lensignore`
- Large-file navigation stopping after the first chunk
- Missing release-build and packaging verification
- Browser behavior regressing without asset-level checks

## Artifacts To Start

- [`README.md`](../../README.md) - build, run, renderer privacy, ignore-rule,
  and verification instructions.
- [CI workflow](../../.github/workflows/ci.yml) - cross-platform verification
  contract.

## Artifacts To Refine

- Feature brief: [`docs/features/lens-viewer.md`](../features/lens-viewer.md) -
  refined with CommonMark, ignore imports, and release evidence.
- Browser asset: [`assets/index.html`](../../assets/index.html) - refined with
  line seeking and previous/next chunk navigation.

## Artifacts Consulted

- E4 results: [`e4-document-fidelity.md`](e4-document-fidelity.md).
- `UC-02`, `UC-03`, and the E4 file/privacy contracts.

## Decisions To Record

- Use pinned `pulldown-cmark` for CommonMark structural analysis while keeping
  untrusted HTML out of the browser DOM.
- Import `.gitignore` first and `.lensignore` second so project-specific rules
  can override repository defaults.
- Support previous/next chunk navigation and direct line seeking for large
  files.
- Verify formatting, tests, Clippy, release builds, and package contents on
  Linux, macOS, and Windows CI runners.

## Trace

- `UC-02` -> `readFile(path, range)` -> chunk continuation and line seeking
- `UC-02` -> ignore policy -> `.gitignore` plus `.lensignore`
- `UC-03` -> CommonMark analysis -> safe Markdown presentation
- `UC-01` -> release build/package -> distributable CLI asset

## Exit Criteria

- CommonMark structure and fenced PlantUML analysis are covered by tests.
- Both ignore files are imported with explicit ignored reads preserved.
- The browser supports previous/next chunk navigation and line seeking.
- README documents build, run, renderer privacy, and verification behavior.
- CI verifies format, tests, Clippy, release builds, and package contents on
  three operating systems.
- A locked release build and package complete locally.

## Results

All exit criteria passed. Lens uses pinned `pulldown-cmark` analysis, imports
`.gitignore` and `.lensignore`, supports chunk continuation and line seeking,
and now has README and cross-platform CI release checks. The browser continues
to build DOM nodes directly, so CommonMark structure does not reintroduce raw
HTML injection.

Verification:

- `cargo test --locked`: 14 passed.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: passed.
- `cargo fmt --check`: passed.
- `cargo build --locked --release`: passed.
- `cargo package --locked --allow-dirty --no-verify`: passed.

Residual risks are full CommonMark visual parity, platform-specific browser
launch behavior, and CI coverage of actual release archive installation.

## Artifact Outcomes

- started: [`README.md`](../../README.md) - user-facing release and privacy
  instructions.
- started: [CI workflow](../../.github/workflows/ci.yml) - cross-platform
  verification automation.
- refined: `Lens Viewer` - current source of truth for E5 compatibility and
  release behavior.
- refined: browser asset - chunk navigation and line seeking.
- started: `E5: Release Readiness` - this file - closed with implementation and
  verification evidence.
