# Iteration: E3 V1 Release Readiness

Status: completed

Goal:

- Stabilize the V1 scope, Linux packaging path, and executable release checks.

Risks Addressed:

- `R-04`: code browsing has ambiguous scope.
- `R-06`: browser launch varies across desktop environments.
- `R-07`: a standalone distribution could be difficult to install.

Artifacts to Start:

- `ADR-004`, V1 release scope:
  [`docs/decisions/adr-004-v1-release-scope.md`](../decisions/adr-004-v1-release-scope.md)
- V1 release readiness:
  [`docs/release-readiness.md`](../release-readiness.md)

Artifacts to Refine:

- Vision and business case: [`docs/vision.md`](../vision.md)
- Supplementary specification: [`docs/supplementary-specification.md`](../supplementary-specification.md)
- `FEAT-01`, primary feature and use cases:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md)

Trace:

- `UC-01` through `UC-05` -> release readiness -> Linux Cargo package and
  browser-session verification

Exit Criteria:

- V1 scope and supported platform are explicit.
- A supported install command and Linux browser-launch behavior are documented.
- Automated and manual release checks cover the V1 user-visible behavior.
- `UC-06` is deferred rather than silently omitted.

Results:

- ADR-004 fixes V1 as a Linux documentation-only release and defers `UC-06`.
- Added a public README, MIT License, Cargo package metadata, and a release
  readiness checklist.
- Added process-level CLI acceptance tests for help, missing targets, and a
  current directory with no discoverable Markdown documents.
- `cargo package --allow-dirty` verified the source package. An isolated
  `cargo install --path . --locked --root /tmp/opencode/lens-v1-install`
  succeeded, and the installed binary reported the expected optional target.
- Public repository, homepage, and hosted documentation URLs remain deferred
  until canonical project URLs exist.

Artifact Outcomes:

- started: `ADR-004`, V1 release scope:
  [`docs/decisions/adr-004-v1-release-scope.md`](../decisions/adr-004-v1-release-scope.md) -
  documented the Linux, documentation-only, Cargo, and MIT decisions.
- started: V1 release readiness:
  [`docs/release-readiness.md`](../release-readiness.md) - defined automated,
  installation, target, and rendering checks.
- refined: Vision and business case: [`docs/vision.md`](../vision.md) -
  aligned V1 scope with the accepted release boundary.
- refined: Supplementary specification:
  [`docs/supplementary-specification.md`](../supplementary-specification.md) -
  records Linux support and source installation.
