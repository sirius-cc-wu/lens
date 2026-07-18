# Iteration: C1 Browser End-to-End Harness

Status: completed

Goal:

- Establish a deterministic browser-test slice that starts the compiled `lens`
  command, renders a temporary documentation repository, and follows a
  discovered-document link.

Risks Addressed:

- `R-01`: public PlantUML service availability can make automated rendering
  evidence nondeterministic.

Artifacts to Start:

- `BTE-01`, browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - verify
  the compiled CLI, loopback viewer, browser, and a controlled renderer in one
  observable path.
- `ADR-005`, controlled renderer in browser tests:
  [`docs/decisions/adr-005-controlled-renderer-browser-tests.md`](../decisions/adr-005-controlled-renderer-browser-tests.md) -
  preserve the reason and scope of the renderer override.

Artifacts to Refine:

- Improvement proposals:
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - record that
  proposal 9 is selected and divide its browser paths into focused iterations.
- Supplementary specification:
  [`docs/supplementary-specification.md`](../supplementary-specification.md) -
  record the automated verification boundary for renderer behavior.
- V1 release readiness:
  [`docs/release-readiness.md`](../release-readiness.md) - add the executable
  browser-test command.
- Risk list: [`docs/risk-list.md`](../risk-list.md) - record the deterministic
  rendering evidence and the remaining browser coverage.

Artifacts Consulted:

- `FEAT-01`, primary feature and use cases:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md)
- `SSD-01`, open a Markdown target:
  [`docs/features/markdown-viewing/ssd-01-open-markdown-target.md`](../features/markdown-viewing/ssd-01-open-markdown-target.md)
- `SSD-02`, open and navigate a document root:
  [`docs/features/markdown-viewing/ssd-02-open-document-root.md`](../features/markdown-viewing/ssd-02-open-document-root.md)
- `ADR-001`, public PlantUML rendering:
  [`docs/decisions/adr-001-public-plantuml-rendering.md`](../decisions/adr-001-public-plantuml-rendering.md)
- `ADR-002`, loopback viewer scope:
  [`docs/decisions/adr-002-loopback-viewer-scope.md`](../decisions/adr-002-loopback-viewer-scope.md)

Decisions to Record:

- Controlled renderer test seam:
  [`docs/decisions/adr-005-controlled-renderer-browser-tests.md`](../decisions/adr-005-controlled-renderer-browser-tests.md)

Trace:

- `UC-01` -> `SSD-01` -> `BTE-01` rendered document and controlled SVG
- `UC-02` through `UC-04` -> `SSD-02` -> `BTE-01` discovered-document link

Exit Criteria:

- A headless browser test starts the compiled `lens` command against a
  temporary document root and discovers its printed loopback URL.
- A controlled renderer receives the diagram request and returns an SVG that
  the browser loads successfully.
- The browser verifies rendered Markdown and follows a link to a discovered
  document.
- The test does not open a desktop browser or contact the public renderer.

Results:

- Added Playwright with the installed Google Chrome channel and a browser-test
  command that builds the Rust binary before executing the suite.
- `BTE-01` creates a temporary document root, replaces `xdg-open` with a
  successful test stub, starts the compiled binary, and reads its loopback URL
  from standard output.
- `LENS_PLANTUML_SERVER` lets that child process use a controlled loopback
  renderer while the default remains the ADR-001 public server.
- The focused browser run first failed with the controlled renderer receiving
  zero requests, then passed after the override was implemented.
- Residual scope: C2 must cover the guidance page for an undiscovered path and
  the browser-visible PlantUML failure state.

Artifact Outcomes:

- started: `BTE-01`, browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - proves
  compiled-command startup, rendered Markdown, a controlled SVG, and document
  navigation.
- started: `ADR-005`, controlled renderer in browser tests:
  [`docs/decisions/adr-005-controlled-renderer-browser-tests.md`](../decisions/adr-005-controlled-renderer-browser-tests.md) -
  bounds the environment override to automated verification.
- refined: Improvement proposals:
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - proposal 9
  is selected for C1 and C2.
- refined: Supplementary specification:
  [`docs/supplementary-specification.md`](../supplementary-specification.md) -
  records deterministic browser verification.
- refined: V1 release readiness:
  [`docs/release-readiness.md`](../release-readiness.md) - includes the
  browser-test command.
- refined: Risk list: [`docs/risk-list.md`](../risk-list.md) - records C1
  coverage and C2 residual work.
