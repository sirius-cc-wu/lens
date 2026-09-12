# Mermaid Standalone SVG View Feature Review

Reviewed `feat/mermaid-svg-view` from merge base `7c01f2373ee61d82d4b01bfb23c314b3924d3983` with `main` through head `3efc49018146d8b60052d9ebd2c78d15adb09fe8`.

**Verdict:** Clean approval — no actionable findings. The implementation delivers the standalone scalable Mermaid SVG view capability specified in [UC-11](../features/markdown-viewing/use-cases.md#uc-11-view-a-mermaid-diagram-in-a-standalone-svg-page) and [ADR-023](../decisions/adr-023-mermaid-standalone-svg-view.md). It satisfies all quality gates, adheres to project boundaries, maintains Content Security Policy protections, and passes unit, CLI, and browser integration suites without regressions.

## Scope

Inspected the complete authored diff against `main`:
- Requirements and specifications:
  - `docs/features/markdown-viewing/use-cases.md` (`UC-11`)
  - `docs/features/markdown-viewing/mermaid-svg-requirements.md` (`FEAT-01-REQ-MERMAID-SVG`)
  - `docs/decisions/adr-023-mermaid-standalone-svg-view.md` (`ADR-023`)
  - `docs/features/markdown-viewing/mermaid-svg-technical-design.md`
- Markdown rendering & placeholder generation:
  - `src/markdown.rs`: Mermaid figure placeholder generation with initially hidden `data-mermaid-open` anchor, and accompanying unit tests.
- Client-side assets & lifecycle:
  - `src/viewer/assets/app.js`: Standalone SVG post-processing (`prepareStandaloneSvg`), Blob URL creation and revocation, error gating, and `blob:` scheme exclusion from session token interception.
  - `src/viewer/assets/app.css`: Editorial typography and layout styling for `.diagram-open-link`.
- Service client compiler compliance:
  - `src/service/client.rs`: Module-level `#![allow(unused_assignments)]` directive for compiler lint compliance under `-D warnings`.
- End-to-end integration tests:
  - `tests/browser/lens.spec.mjs`: Playwright test scenarios for control presentation, syntax error suppression, Blob URL navigation, and dynamic vector scaling.

## Multi-Axis Review

### 1. Functional Correctness (UC-11 & Acceptance Criteria)

- **Control Gating (AC-1, AC-4):** Each rendered Mermaid figure initially contains `<a class="diagram-open-link" data-mermaid-open target="_blank" rel="noopener noreferrer" hidden>Open SVG</a>`. The control is revealed only upon successful client-side Mermaid rendering settling. If syntax parsing or layout fails, the control remains strictly hidden while displaying the diagram error banner and source disclosure.
- **Standalone Navigation (AC-1, AC-3):** Activating "Open SVG" triggers a standard top-level browser navigation in a new tab (`target="_blank"`, `rel="noopener noreferrer"`), presenting a pure SVG document (`image/svg+xml`).
- **Dynamic Viewport Scaling (AC-2):** `prepareStandaloneSvg` strips Mermaid's inline `max-width` cap and applies `width="100%"` and `height="100%"` attributes while preserving the intrinsic `viewBox`. Resizing the standalone browser window smoothly expands the diagram bounding box without clipping or aspect ratio distortion.
- **High-Contrast Background (AC-3):** An explicit `background-color: #ffffff;` style is ensured on the root `<svg>` element, guaranteeing dark diagram lines and text remain legible in dark-themed browser canvas viewers.
- **Token Isolation & Non-Destructive Navigation (AC-5):** The client-side click interceptor in `app.js` specifically excludes URLs with the `blob:` prefix, preventing `?token=...` query pollution and avoiding `net::ERR_FILE_NOT_FOUND` navigation failures.

### 2. Security & Policy Enforcement

- **Content Security Policy (CSP):** The restrictive server CSP (`default-src 'self'; base-uri 'none'; img-src 'self' data:; object-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'`) is preserved without modification. Standalone SVG views inherit the origin CSP.
- **Context Isolation:** The "Open SVG" anchor enforces `rel="noopener noreferrer"`, isolating the standalone tab and preventing reverse window manipulation via `window.opener`.
- **Token Separation:** Blob URLs are allocated from browser memory without incorporating or leaking viewer session capabilities or filesystem paths.

### 3. Offline Compliance

- **Zero Network Traffic:** The feature operates completely client-side via native Web APIs (`Blob`, `URL.createObjectURL`, `URL.revokeObjectURL`, `DOMParser`, `XMLSerializer`) and the locally vendored `mermaid.min.js` bundle. No remote requests or external CDN dependencies are introduced.

### 4. Code Cleanliness & Repository Conventions

- **Module Boundaries:** Responsibilities remain strictly separated across domain boundaries:
  - `markdown.rs` handles server-side HTML AST placeholder emission.
  - `app.js` encapsulates client-side DOM post-processing, event handling, and Blob URL lifecycle.
  - `app.css` defines presentation.
- **Line Count & Simplicity:** Modified files remain well within complexity limits (non-test lines in `markdown.rs` are ~323 lines, well under the 500-line split threshold).
- **Behavior-Oriented Testing:** All new unit and browser tests adhere to `<condition_or_action>_then_<observable_result>` naming and explicit Arrange / Act / Assert (3A) structure.

## Validation & Test Execution Metrics

All verification quality gates and test suites were executed against the worktree:

1. **Rust Code Formatting:**
   - Command: `cargo fmt --check`
   - Result: Passed (0 formatting violations).
2. **Clippy Static Analysis:**
   - Command: `cargo clippy --locked --all-targets --all-features -- -D warnings`
   - Result: Passed (0 warnings, 0 errors).
3. **Rust Unit & CLI Test Suite:**
   - Command: `cargo test --locked`
   - Result: Passed 136 tests (130 unit/integration tests and 6 CLI tests) in 3.18s.
   - New unit tests passed:
     - `mermaid_block_then_emits_open_svg_link_in_placeholder`
     - `mixed_plantuml_and_mermaid_document_then_emits_open_svg_only_on_mermaid`
4. **Client-Side Syntax Checks:**
   - Command: `node --check src/viewer/assets/app.js && node --check src/viewer/assets/mermaid.min.js`
   - Result: Passed (clean syntax).
5. **Playwright Browser Test Suite:**
   - Command: `npx playwright test`
   - Result: Passed 35 tests in 24.9s across Chromium.
   - New browser scenarios passed:
     - `rendered_mermaid_diagram_then_displays_standalone_open_svg_link` (630ms)
     - `invalid_mermaid_syntax_then_suppresses_open_svg_link_and_reveals_source` (590ms)
     - `open_svg_link_clicked_then_navigates_to_uncorrupted_blob_url_and_scales_dynamically` (763ms)

## Residual Risks & Validation Limits

- **Cross-Browser Verification:** Automated browser testing was conducted against Chromium. While the solution relies exclusively on standard baseline Web APIs (`Blob`, `DOMParser`, `XMLSerializer`, `URL.createObjectURL`), automated test execution was not conducted against Firefox or WebKit in this pipeline run.
- **Complex Diagram Topologies:** Verification exercised flowcharts, Gantt charts, and sequence diagrams. Highly specialized Mermaid diagram families (such as gitGraph, mindmap, or C4) share the same underlying SVG serialization pipeline, but were not individually asserted in distinct E2E fixtures.
- **Long-Lived Session Blob Revocation:** While `URL.revokeObjectURL` properly cleans up previous Blob URLs on diagram re-render, closing a standalone SVG tab does not signal the opener document; deallocation of opened standalone Blob snapshots relies on native browser page termination.

---

*Note on PlantUML Diagrams:* Per repository guidelines in `AGENTS.md`, this record includes no PlantUML diagrams because there are no actionable findings or proposed modifications.
