# ADR-005: Use a Controlled PlantUML Renderer in Browser Tests

Status: accepted

Date: 2026-07-18

## Context

The browser end-to-end suite must start the compiled `lens` command and verify
PlantUML rendering without depending on the availability or response behavior
of the public PlantUML service. A local server whose responses are specified by
the test is called a controlled renderer.

The production renderer is intentionally selected by ADR-001. The test suite
needs a narrow way to substitute only that external dependency while preserving
the CLI, loopback server, browser, and diagram-route path.

## Decision

Lens reads `LENS_PLANTUML_SERVER` as a renderer base URL when it is present and
non-empty. Browser tests set it on the child `lens` process to point at a
controlled renderer on loopback. When it is absent, Lens uses the public
PlantUML server specified by ADR-001.

This is an automated-test hook, not a user-facing renderer-selection feature.
Lens does not add a CLI option, persist the override, or expose the configured
renderer URL through a browser route.

## Consequences

- Browser tests can verify SVG success and rendering failure deterministically
  without contacting the public service.
- The browser continues to request only Lens-owned diagram identifiers; the
  environment variable changes the server-side destination for those known
  diagrams, not the viewer's request surface.
- A future user-facing renderer setting must be designed separately; this hook
  does not decide the local-renderer product proposal.

## Trace

- Proposal: [Automated Browser End-to-End Testing](../improvement-proposals.md#9-automated-browser-end-to-end-testing)
- Requirements: `UC-01` through `UC-05` in
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md)
- Security boundary: [ADR-002](adr-002-loopback-viewer-scope.md)
- Verification evidence: `BTE-01`,
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs)
