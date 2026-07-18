# Iteration: C2 Browser Guidance and Renderer Failure

Status: completed

Goal:

- Complete the browser end-to-end evidence for an undiscovered document path
  and a failed PlantUML request without weakening the existing success path.

Risks Addressed:

- `R-01`: external renderer behavior can make failure handling hard to verify.
- `R-03`: a browser request might disclose a document that was not authorized
  for the viewing session.

Artifacts to Start:

- None. C1 established the canonical browser suite and controlled-renderer
  decision required for this work.

Artifacts to Refine:

- `BTE-01`, browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - add the
  undiscovered-path guidance and renderer-failure scenarios.
- Improvement proposals:
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - close
  proposal 9 with its executable evidence.
- V1 release readiness:
  [`docs/release-readiness.md`](../release-readiness.md) - state the browser
  outcomes covered by the required command.
- Risk list: [`docs/risk-list.md`](../risk-list.md) - update the renderer and
  document-authorization evidence.

Artifacts Consulted:

- `FEAT-01`, primary feature and use cases:
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md)
- `SSD-01`, open a Markdown target:
  [`docs/features/markdown-viewing/ssd-01-open-markdown-target.md`](../features/markdown-viewing/ssd-01-open-markdown-target.md)
- `SSD-02`, open and navigate a document root:
  [`docs/features/markdown-viewing/ssd-02-open-document-root.md`](../features/markdown-viewing/ssd-02-open-document-root.md)
- `ADR-002`, loopback viewer scope:
  [`docs/decisions/adr-002-loopback-viewer-scope.md`](../decisions/adr-002-loopback-viewer-scope.md)
- `ADR-005`, controlled renderer in browser tests:
  [`docs/decisions/adr-005-controlled-renderer-browser-tests.md`](../decisions/adr-005-controlled-renderer-browser-tests.md)

Decisions to Record:

- None. C2 uses the controlled-renderer boundary already accepted by ADR-005.

Trace:

- `UC-04` -> `SSD-02` -> `BTE-01` undiscovered document -> Lens guidance page
- `UC-05` -> `SSD-01` -> `BTE-01` controlled renderer failure -> visible error
  and source

Exit Criteria:

- A browser request for an existing but hidden Markdown file that is outside the
  discovered document set receives the Lens guidance page and not that file's
  content.
- A controlled renderer failure makes the diagram error visible, opens the
  source, and leaves the surrounding Markdown readable.
- The new checks demonstrate discrimination against an incorrect browser
  failure presentation.

Results:

- `BTE-01` now creates a hidden Markdown file with confidential text, requests
  its otherwise plausible document route, and verifies that the guidance page
  omits the text while retaining the return link.
- The controlled renderer can return HTTP 503. The browser test verifies that
  Lens makes its diagram error visible, opens the PlantUML source, and keeps
  the Markdown body readable.
- The new tests passed against the existing implementation. As a negative
  control, removing the browser script line that reveals the error made the
  renderer-failure test fail because the error remained hidden; the line was
  restored before this iteration closed.

Artifact Outcomes:

- refined: `BTE-01`, browser end-to-end suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - covers
  rendered Markdown, document navigation, an undiscovered document, and
  renderer success and failure with a controlled renderer.
- refined: Improvement proposals:
  [`docs/improvement-proposals.md`](../improvement-proposals.md) - proposal 9
  is implemented by the C1 and C2 browser suite.
- refined: V1 release readiness:
  [`docs/release-readiness.md`](../release-readiness.md) - identifies the
  browser-visible outcomes checked by `npm run test:browser`.
- refined: Risk list: [`docs/risk-list.md`](../risk-list.md) - records browser
  evidence for renderer failure and undiscovered documents.
