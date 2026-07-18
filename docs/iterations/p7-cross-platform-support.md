# Iteration: P7 Cross-Platform Support

Status: completed

Phase Intent:

- Turn existing platform-specific browser launch branches into a tested and
  packageable supported-platform boundary.

Goal:

- Start Lens in a default browser and prepare native archives on Linux, macOS,
  and Windows without changing the loopback authorization model.

Risks Addressed:

- `R-06`: desktop browser launch and process behavior vary by operating system.
- `R-07`: platform-specific binary installation needs an unambiguous artifact.

Artifacts to Start:

- `ADR-013`, native browser launch and archives:
  [`docs/decisions/adr-013-cross-platform-support.md`](../decisions/adr-013-cross-platform-support.md) - fix platform commands and native-runner packaging expectations.

Artifacts to Refine:

- `ADR-004`, V1 release scope, and `ADR-010`, binary artifacts:
  [`docs/decisions/adr-004-v1-release-scope.md`](../decisions/adr-004-v1-release-scope.md) and [`docs/decisions/adr-010-linux-binary-release-artifacts.md`](../decisions/adr-010-linux-binary-release-artifacts.md) - replace the Linux-only platform assumption.
- Release readiness, supplementary specification, README, proposal list, and risk list:
  [`docs/release-readiness.md`](../release-readiness.md), [`docs/supplementary-specification.md`](../supplementary-specification.md), [`README.md`](../../README.md), [`docs/improvement-proposals.md`](../improvement-proposals.md), and [`docs/risk-list.md`](../risk-list.md) - state launcher, artifact, and fallback behavior.

Artifacts Consulted:

- `ADR-002`, loopback viewer scope:
  [`docs/decisions/adr-002-loopback-viewer-scope.md`](../decisions/adr-002-loopback-viewer-scope.md)
- `ADR-010`, target-specific artifact contract:
  [`docs/decisions/adr-010-linux-binary-release-artifacts.md`](../decisions/adr-010-linux-binary-release-artifacts.md)

Decisions to Record:

- `ADR-013`: test browser-launch command construction independently from the
  host operating system, and package native binaries through one target-aware
  release command.

Trace:

- Proposal 7 -> `ADR-013` -> browser-command unit test and native-target
  package command

Exit Criteria:

- Linux, macOS, and Windows command forms are covered by a deterministic unit
  test.
- Every supported target has an archive contract that preserves its binary name
  and checksum.
- A host-target archive passes checksum and content checks.
- Formatter, tests, Clippy, and package verification pass.

Results:

- Browser spawning now delegates to a platform-command value, making all three
  launch forms testable from the Linux test host. Unsupported systems return an
  error so Lens continues to print its loopback URL for manual opening.
- `scripts/package-release.sh` packages Linux, macOS, and Windows targets,
  including `lens.exe` for Windows. The prior Linux command is a compatibility
  wrapper. A fresh Linux artifact passed SHA-256 and archive-content checks.
- Tagged native-runner execution and publication are deferred to P8 release
  automation.

Artifact Outcomes:

- started: `ADR-013`, native browser launch and archives:
  [`docs/decisions/adr-013-cross-platform-support.md`](../decisions/adr-013-cross-platform-support.md) - defines the cross-platform boundary.
- refined: release scope, artifact format, readiness, user documentation, and
  risk records - replace the Linux-only assumption with executable evidence.
