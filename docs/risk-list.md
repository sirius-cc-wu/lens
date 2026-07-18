# Risk List

| ID | Risk | Type | Likelihood | Impact | Mitigation and evidence needed |
|---|---|---|---|---|---|
| `R-01` | The public PlantUML server may be unavailable, rate-limit requests, or change response behavior. | Technical / adoption | Medium | High | E1 validated a live SVG response plus valid, invalid, unavailable, and delayed mocked responses. C1 adds a browser-level success path against a controlled renderer, so external availability cannot make that regression test fail. A 10-second timeout and 2 MiB response limit now bound failures; real-service availability and browser-level failure evidence remain residual risks. |
| `R-02` | Unsanitized Markdown or SVG can execute unsafe browser content. | Technical / security | Medium | High | E1 escapes raw Markdown HTML, serves PlantUML SVG only as an image, and sets a restrictive content security policy. Expand malicious-content fixtures and browser testing before construction broadens the parser surface. |
| `R-03` | File and directory resolution can expose files outside the requested repository. | Technical / security | Low | High | E2 canonicalizes the document root, excludes discovered symbolic links and hidden entries, and serves only known document identifiers. Traversal and symlink behavior are tested. Large-repository discovery limits remain a residual risk. |
| `R-04` | "Display the codebase's code and document" is too ambiguous to estimate or scope. | Product | Low | Medium | ADR-004 defines V1 as documentation-only. Any source-code browsing requires a later product decision and use case. |
| `R-06` | Browser launch and process lifetime vary across desktop, container, and headless environments. | Technical | Medium | Medium | V1 supports Linux `xdg-open` with a manual URL fallback. Validate the release behavior in the V1 readiness check. |
| `R-07` | A standalone distribution could be harder to install than an editor plugin. | Schedule / adoption | Medium | Medium | V1 uses `cargo install --path . --locked`; validate a local source installation before release. |

`R-01` through `R-04` are architectural and product risks that drive the first
elaboration iteration.
