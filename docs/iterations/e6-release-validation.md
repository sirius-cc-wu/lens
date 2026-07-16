# E6: Release Validation

Status: completed

## Goal

Validate the actual Lens browser experience and release artifacts, not only the
Rust crate build.

## Risks Addressed

- Browser asset regressions that unit tests cannot observe
- Renderer upload scope being broader than the selected PlantUML source
- Release archives lacking checksums or required runtime files
- Platform release builds diverging from the development target
- Unclear versioning and release criteria

## Artifacts To Start

- [`docs/release.md`](../release.md) - SemVer, archive, checksum, and release
  checklist.
- [`scripts/browser-smoke.mjs`](../../scripts/browser-smoke.mjs) - Chrome
  headless browser smoke test.
- [`scripts/package-release.sh`](../../scripts/package-release.sh) - native
  archive and checksum builder.

## Artifacts To Refine

- CI workflow: [`../../.github/workflows/ci.yml`](../../.github/workflows/ci.yml)
  - adds asset, browser, and release archive verification.
- Feature brief: [`docs/features/lens-viewer.md`](../features/lens-viewer.md) -
  refined with release and security evidence.

## Artifacts Consulted

- E5 results: [`e5-release-readiness.md`](e5-release-readiness.md).
- `UC-01`, `UC-02`, `UC-03`, and the file/privacy contracts.

## Decisions To Record

- Keep release versioning under SemVer, currently `0.1.x`.
- Package native binary plus README in a target-named archive and publish its
  SHA-256 sidecar.
- Use Chrome headless smoke coverage on Ubuntu and native release builds on
  Linux, macOS, and Windows CI runners.
- Treat renderer upload scope and browser URL safety as release checks.

## Trace

- `UC-01` -> release binary -> browser smoke test
- `UC-02` -> file boundary tests -> archive/runtime verification
- `UC-03` -> renderer scope test -> source-only upload assertion

## Exit Criteria

- Browser asset checks pass without `innerHTML` or unsafe URL schemes.
- Chrome opens the release workspace and exposes the expected DOM shell.
- Renderer tests prove only selected PlantUML source is uploaded.
- A release archive and SHA-256 sidecar are created and verified locally.
- Archive contents include the native binary and README.
- Cross-platform CI is defined for all supported release targets.
- Release criteria and SemVer policy are documented.

## Results

All exit criteria passed. The release binary was served locally and validated by
Chrome headless, the Linux archive checksum and contents were verified, and
security checks cover file boundaries, safe links, ignored paths, and renderer
source scope. CI now defines Linux GNU, macOS ARM64, and Windows MSVC archive
jobs, plus cross-platform verification.

Verification:

- `node scripts/asset-check.mjs`: passed.
- Chrome headless browser smoke test: passed.
- `cargo test --locked`: 14 passed.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: passed.
- `cargo build --locked --release`: passed.
- `bash scripts/package-release.sh x86_64-unknown-linux-gnu dist`: passed.
- Archive checksum and contents: passed.

Residual risks are full browser visual regression coverage, actual installation
testing of every platform archive, and external renderer service availability.

## Artifact Outcomes

- started: release checklist, browser smoke script, and package script.
- refined: CI workflow with browser and release jobs.
- refined: `Lens Viewer` with release and security evidence.
- started: `E6: Release Validation` - this file - closed with implementation and
  verification evidence.
