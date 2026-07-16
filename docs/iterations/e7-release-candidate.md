# E7: Release Candidate

Status: completed

## Goal

Validate the shipped experience from archive extraction through browser
interaction and define the first release candidate criteria.

## Risks Addressed

- Browser shell tests not exercising user interactions
- Release archives not being executable after extraction
- Startup or shutdown regressions in packaged binaries
- PlantUML source toggles failing in the real browser
- Release candidate criteria being implicit

## Artifacts To Start

- [`scripts/browser-interaction.mjs`](../../scripts/browser-interaction.mjs) -
  Chrome DevTools Protocol interaction test.
- [`scripts/install-smoke.sh`](../../scripts/install-smoke.sh) - archive
  extraction, help, startup, and shutdown smoke test.
- [`tests/fixtures/browser-repo/README.md`](../../tests/fixtures/browser-repo/README.md)
  - deterministic browser interaction fixture.

## Artifacts To Refine

- CI workflow: [`../../.github/workflows/ci.yml`](../../.github/workflows/ci.yml)
  - runs browser interaction and per-target archive installation checks.
- Release checklist: [`docs/release.md`](../release.md) - adds interaction and
  installation gates.

## Artifacts Consulted

- E6 results: [`e6-release-validation.md`](e6-release-validation.md).
- `UC-01`, `UC-02`, `UC-03`, and the release candidate contract.

## Decisions To Record

- Use a deterministic repository fixture for browser interaction tests.
- Test the release binary through Chrome DevTools Protocol, not only static
  HTML output.
- Test each target archive after extraction before it is eligible for release.
- Keep the first release candidate on the `0.1.x` SemVer line.

## Trace

- `UC-01` -> extracted binary -> startup and graceful shutdown smoke test
- `UC-02` -> fixture navigation -> selected document interaction
- `UC-03` -> PlantUML card -> source/diagram toggle interaction

## Exit Criteria

- Chrome opens the fixture workspace and lists its Markdown document.
- Browser interaction opens the document, finds its PlantUML card, and toggles
  to source view.
- Every locally built archive extracts a binary and README successfully.
- An extracted binary responds to `--help`, starts a workspace, and shuts down
  cleanly.
- CI runs these checks for the supported release targets.
- Release candidate criteria are documented.

## Results

All exit criteria passed locally. Chrome DevTools Protocol verified document
selection and PlantUML source toggling, and the Linux release archive passed
extraction, checksum, help, startup, and shutdown checks. CI now applies the
same checks to its release matrix.

Verification:

- `node scripts/asset-check.mjs`: passed.
- Chrome shell and interaction smoke tests: passed.
- `bash scripts/install-smoke.sh dist/lens-0.1.0-x86_64-unknown-linux-gnu.tar.gz`:
  passed.
- Archive checksum and contents: passed.
- `cargo test --locked`: 14 passed.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: passed.

Residual risks are full visual regression snapshots, platform archive smoke
execution in CI, and external renderer service availability.

## Artifact Outcomes

- started: browser interaction, installation smoke, and browser fixture assets.
- refined: CI and release checklist with release-candidate gates.
- started: `E7: Release Candidate` - this file - closed with implementation and
  verification evidence.
