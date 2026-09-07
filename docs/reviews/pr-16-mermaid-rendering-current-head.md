# PR 16 review: current Mermaid implementation

Reviewed PR 16 on `feat/mermaid-diagram-rendering`, from merge base
`723937f6f32085ecc648248dec5dcb7504608dc2` through head
`a5307836a6c2442bda68173aeedffc5183eebd7c`.

No actionable findings. The current implementation resolves both findings in
[the earlier review](pr-16-mermaid-rendering.md): diagram configuration cannot
apply the reported document-wide CSS override, and a Gantt chart excluding every
weekday fails without blocking subsequent diagrams or browser interaction.
Both regression tests passed against the compiled server and shipped assets.

## Scope

Inspected the complete authored diff and the affected paths: Markdown fence
recognition and escaping, mixed PlantUML/Mermaid numbering, page construction and
capability insertion, asset routes and authentication middleware, Content
Security Policy, browser rendering and failure handling, link handling, document
revision polling and server refresh, and relevant tests. Also inspected the
incidental PlantUML buffer-capacity and Unix lock-file changes. Reviewed the
vendored bundle's configuration protection, sanitization, rendering limits, and
packaging; this was not a line-by-line audit of third-party library internals.

## Validation

- `cargo test --locked`: passed, 128 library tests and 6 CLI tests.
- `cargo fmt --check`: passed.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: passed.
- `node --check` for both `app.js` and `mermaid.min.js`: passed.
- `npm run test:browser`: all 32 Chromium scenarios passed, including both
  earlier finding regressions, route authentication, PlantUML retry, and refresh.
- One additional temporary Playwright scenario passed against the compiled Lens
  server. Flowchart, sequence, and class diagrams rendered; malformed and empty
  Mermaid fences revealed source and an error while a later valid diagram still
  rendered. Editing the Markdown file automatically refreshed the diagram.
  All non-session-origin requests were intercepted and blocked; none were
  attempted during this scenario. The temporary test was removed after review.

## Residual risks and validation limits

Firefox, Safari, and the full range of Mermaid diagram types and configuration
options were not exercised. Offline rendering was verified for the diagram
families above, not every optional renderer or asset feature. The vendored
library was not subjected to a comprehensive security audit. The repository
suite permanently covers the two prior defects; the broader Mermaid rendering
and refresh check remains temporary review evidence.

This record adds no PlantUML diagrams because there are no actionable findings;
no existing review diagrams were changed. No application source was modified.
