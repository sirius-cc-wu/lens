# Iteration: P8 Release Automation

Status: completed

Phase Intent:

- Make the checked source and native release archives reproducible through
  repository-owned automation.

Goal:

- Verify every change before merge and publish a complete, native artifact set
  when a version-matching tag is pushed.

Risks Addressed:

- `R-07`: a distribution without repeatable native archives is harder to
  install and trust.

Artifacts to Start:

- `ADR-013`, release automation:
  [`docs/decisions/adr-013-release-automation.md`](../decisions/adr-013-release-automation.md) - defines verification, tag, matrix, and publish boundaries.

Artifacts to Refine:

- Release readiness, risk list, proposal list, and documentation index:
  [`docs/release-readiness.md`](../release-readiness.md),
  [`docs/risk-list.md`](../risk-list.md),
  [`docs/improvement-proposals.md`](../improvement-proposals.md), and
  [`docs/index.md`](../index.md) - record the release trigger and remaining manual evidence.
- Target package command:
  [`scripts/package-release.sh`](../../scripts/package-release.sh) - support the SHA-256 utility available on each native release runner.

Artifacts Consulted:

- `ADR-009`, target-specific release artifacts:
  [`docs/decisions/adr-009-linux-binary-release-artifacts.md`](../decisions/adr-009-linux-binary-release-artifacts.md)
- `ADR-012`, native browser launch and archives:
  [`docs/decisions/adr-012-cross-platform-support.md`](../decisions/adr-012-cross-platform-support.md)

Decisions Recorded:

- `ADR-013`: preserve one package command, verify the manifest-tag match, and
  publish only the complete set of native artifacts.

Trace:

- Proposal 8 -> `ADR-013` -> verification workflow and native tag-release workflow

Exit Criteria:

- Pull requests and `main` pushes run formatting, tests, Clippy, package, and
  browser verification.
- A version-matching tag invokes the package command once per native target.
- Publication waits for all matrix artifacts and uploads both archives and
  checksums to one GitHub Release.
- Local formatter, tests, Clippy, package verification, browser tests, and
  host archive checks pass.

Results:

- Added a verification workflow for pull requests and `main`, plus a tagged
  native-release matrix and a dependent publish job.
- The tag must equal the Cargo version prefixed with `v`; package jobs use the
  same target-aware script as local release readiness checks.
- Added the macOS `shasum -a 256` fallback to preserve SHA-256 artifact output
  where GNU `sha256sum` is unavailable.

Artifact Outcomes:

- started: `ADR-013`, release automation - records the durable workflow policy.
- refined: packaging portability and release-readiness evidence - connect the
  local package contract to tag publication.
