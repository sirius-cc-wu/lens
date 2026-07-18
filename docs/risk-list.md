# Risk List

| ID | Risk | Type | Likelihood | Impact | Mitigation and evidence needed |
|---|---|---|---|---|---|
| `R-01` | The public PlantUML server may be unavailable, rate-limit requests, or change response behavior. | Technical / adoption | Medium | High | E1 validated a live SVG response plus valid, invalid, unavailable, and delayed mocked responses. A 10-second timeout and 2 MiB response limit now bound failures; real-service availability remains a residual risk. |
| `R-02` | Unsanitized Markdown or SVG can execute unsafe browser content. | Technical / security | Medium | High | E1 escapes raw Markdown HTML, serves PlantUML SVG only as an image, and sets a restrictive content security policy. Expand malicious-content fixtures and browser testing before construction broadens the parser surface. |
| `R-03` | File and directory resolution can expose files outside the requested repository. | Technical / security | Low | High | E2 canonicalizes the document root, excludes discovered symbolic links and hidden entries, and serves only known document identifiers. Traversal and symlink behavior are tested. Large-repository discovery limits remain a residual risk. |
| `R-04` | "Display the codebase's code and document" is too ambiguous to estimate or scope. | Product | High | High | E2 explicitly deferred code-file viewing. Obtain product-owner clarification and prioritized use cases before E3 or construction adds it. |
| `R-06` | Browser launch and process lifetime vary across desktop, container, and headless environments. | Technical | Medium | Medium | Prototype browser launch plus manual URL fallback on the first target platform. |
| `R-07` | A standalone distribution could be harder to install than an editor plugin. | Schedule / adoption | Medium | Medium | Select the implementation language and packaging approach during elaboration; test a clean-machine installation. |

`R-01` through `R-04` are architectural and product risks that drive the first
elaboration iteration.
