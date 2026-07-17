# Risk List

| ID | Risk | Type | Likelihood | Impact | Mitigation and evidence needed |
|---|---|---|---|---|---|
| `R-01` | The public PlantUML server may be unavailable, rate-limit requests, or change response behavior. | Technical / adoption | Medium | High | In `E1`, validate successful, invalid, unavailable, and delayed responses; define user-visible failure behavior. |
| `R-02` | Unsanitized Markdown or SVG can execute unsafe browser content. | Technical / security | Medium | High | Define a sanitization boundary and verify it with malicious Markdown and SVG fixtures before expanding scope. |
| `R-03` | File and directory resolution can expose files outside the requested repository. | Technical / security | Medium | High | Prove a scoped target resolver with traversal and symlink test cases in `E1`. |
| `R-04` | "Display the codebase's code and document" is too ambiguous to estimate or scope. | Product | High | High | Interview the product owner and turn the desired browsing workflow into prioritized use cases before `E2`. |
| `R-06` | Browser launch and process lifetime vary across desktop, container, and headless environments. | Technical | Medium | Medium | Prototype browser launch plus manual URL fallback on the first target platform. |
| `R-07` | A standalone distribution could be harder to install than an editor plugin. | Schedule / adoption | Medium | Medium | Select the implementation language and packaging approach during elaboration; test a clean-machine installation. |

`R-01` through `R-04` are architectural and product risks that drive the first
elaboration iteration.
