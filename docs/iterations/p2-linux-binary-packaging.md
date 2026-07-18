# Iteration: P2 Linux Binary Packaging

Status: completed

Phase Intent:

- Construct a repeatable Linux binary artifact before connecting it to external
  release publication.

Goal:

- Give a release manager a verified, target-specific Lens archive that users
  can install without a Rust toolchain.

Risks Addressed:

- `R-07`: source-only installation can make the standalone viewer difficult to
  adopt.

Artifacts to Start:

- `ADR-009`, Linux binary release artifacts:
  [`docs/decisions/adr-009-linux-binary-release-artifacts.md`](../decisions/adr-009-linux-binary-release-artifacts.md) - fix the archive, checksum, and overwrite policy.

Artifacts to Refine:

- Release readiness:
  [`docs/release-readiness.md`](../release-readiness.md) - provide executable archive verification.
- Improvement proposals, README, and risk list:
  [`docs/improvement-proposals.md`](../improvement-proposals.md), [`README.md`](../../README.md), and [`docs/risk-list.md`](../risk-list.md) - state the delivered installation path and residual publication work.

Artifacts Consulted:

- `ADR-004`, Linux release scope:
  [`docs/decisions/adr-004-v1-release-scope.md`](../decisions/adr-004-v1-release-scope.md)
- V1 release readiness:
  [`docs/release-readiness.md`](../release-readiness.md)

Decisions to Record:

- `ADR-009`: create one explicit-target archive and checksum command, then
  make later tagged automation invoke it.

Trace:

- Proposal 2 -> `ADR-009` -> package command -> archive listing and checksum
  verification

Exit Criteria:

- Packaging builds the selected locked Linux target in release mode.
- The archive includes only the binary and user-facing license and README.
- A checksum validates the archive, and reruns cannot replace a prior artifact.
- Formatter, tests, Clippy, package verification, and an actual archive build
  pass.

Results:

- Added `scripts/package-linux-release.sh`, which defaults to the Rust host
  target and accepts a Linux `--target` plus a separate `--output` directory.
  It creates the archive only after confirming neither output already exists.
- A fresh host-target artifact was checked with `sha256sum --check` and
  `tar -tzf`; it contained only the target directory, executable, README, and
  license.
- GitHub upload remains intentionally deferred to proposal 8, which will reuse
  the package command for tagged releases.

Artifact Outcomes:

- started: `ADR-009`, Linux binary release artifacts:
  [`docs/decisions/adr-009-linux-binary-release-artifacts.md`](../decisions/adr-009-linux-binary-release-artifacts.md) - defines the durable artifact contract.
- refined: release readiness, proposal 2, README, and `R-07` - record command
  use, verification, and the pending tagged-publication step.
