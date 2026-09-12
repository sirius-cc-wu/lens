# Technical Design: Mermaid Diagram Standalone Scalable SVG View

## Objective

Extend [Lens](file:///home/ccwu/lens/README.md) to enable users to view rendered Mermaid diagrams in a dedicated browser tab or window as a scalable, standalone SVG document (`image/svg+xml`).

Lens constrains document prose to a maximum reading width of 920 pixels (`min(920px, calc(100% - 2rem))`). While optimal for text typography, complex Mermaid diagrams (such as large architectural flowcharts, extensive sequence diagrams, and class hierarchies) are compressed to fit within this column, rendering labels illegibly small. Additionally, Mermaid automatically injects inline `max-width: <calculated>px;` styles onto the root `<svg>`, capping diagram scaling even on large monitors.

This capability provides an accessible "Open SVG" link on every successfully rendered Mermaid diagram figure. Activating this control opens the diagram as a standalone SVG in another browser tab, freeing it from column and inline constraints so users can freely resize the browser window, zoom with native browser controls, inspect fine details, or save the vector asset locally—working completely offline with zero external network dependencies or server roundtrips.

## Tech Stack

- **Language / Runtime:** Rust 1.75+ (2021 edition)
- **Markdown Parsing:** `pulldown-cmark` (v0.9.3)
- **HTTP / Loopback Server:** `axum` (v0.6.20)
- **Diagram Engine:** Client-side [Mermaid](https://mermaid.js.org/) JavaScript library (vendored version 11.17.2 embedded via `include_str!`)
- **Browser APIs:** Standard Web APIs (`Blob`, `URL.createObjectURL`, `URL.revokeObjectURL`, `DOMParser`, `XMLSerializer`)
- **Supported Browsers:** Modern evergreen browsers (Chromium-based, Firefox, Safari)

## Commands

- **Build:** `cargo build --locked`
- **Test:** `cargo test --locked`
- **Lint:** `cargo clippy --locked --all-targets --all-features -- -D warnings`
- **Format:** `cargo fmt --check`
- **Dev Run:** `cargo run -- <path-to-markdown-file>`

## Project Structure

```text
src/
├── markdown.rs              # Emits Mermaid placeholder with hidden 'Open SVG' link
├── viewer/
│   ├── assets/
│   │   ├── app.css          # Styles .diagram-open-link matching Lens typography
│   │   ├── app.js           # Adapts SVG, creates/manages Blob URL, handles click routing
│   │   └── mermaid.min.js   # Vendored client-side Mermaid library (offline asset)
│   ├── page.rs              # Content Security Policy and capability isolation
│   └── routes.rs            # Loopback HTTP server routes
docs/
├── decisions/
│   └── adr-023-mermaid-standalone-svg-view.md  # Architecture decision record
└── features/
    └── markdown-viewing/
        ├── mermaid-rendering-spec.md          # Client-side Mermaid rendering spec
        ├── mermaid-svg-requirements.md        # Feature requirements and user story
        └── mermaid-svg-technical-design.md    # This technical design
```

## Module Contracts

### 1. Markdown Placeholder Generation (`src/markdown.rs`)

`mermaid_placeholder` emits a figure containing an accessible anchor with `data-mermaid-open` set to `hidden`:

```html
<figure class="diagram mermaid-diagram" data-mermaid-container>
  <div class="mermaid-target"></div>
  <a class="diagram-open-link" data-mermaid-open target="_blank" rel="noopener noreferrer" hidden>Open SVG</a>
  <p class="diagram-error" hidden>Mermaid rendering failed. The source is shown below.</p>
  <details class="diagram-source">
    <summary>Mermaid source</summary>
    <pre><code>...</code></pre>
  </details>
</figure>
```

#### Invariants:
- The `Open SVG` link is initially `hidden` and is revealed only after Mermaid rendering succeeds.
- The link sets `target="_blank"` and `rel="noopener noreferrer"`.
- Error notices (`.diagram-error`) and source disclosure (`.diagram-source`) remain structurally unchanged.

### 2. Client-Side SVG Adaptation & Lifecycle (`src/viewer/assets/app.js`)

#### A. SVG Post-Processing (`prepareStandaloneSvg`):
Before generating the Blob, sanitize the root `<svg>` element:
1. Strip inline `maxWidth` (`svgEl.style.maxWidth = ''`) to uncap window scaling.
2. Set attributes `width="100%"` and `height="100%"` while retaining the intrinsic `viewBox`.
3. Set an explicit solid background (`background-color: #ffffff;`) if unspecified, ensuring legibility on dark browser canvas backgrounds.

```javascript
function prepareStandaloneSvg(svgText) {
  const parser = new DOMParser();
  const doc = parser.parseFromString(svgText, 'image/svg+xml');
  const svgEl = doc.documentElement;
  if (!svgEl || svgEl.nodeName !== 'svg') {
    return svgText;
  }
  svgEl.style.maxWidth = '';
  svgEl.setAttribute('width', '100%');
  svgEl.setAttribute('height', '100%');
  if (!svgEl.style.backgroundColor) {
    svgEl.style.backgroundColor = '#ffffff';
  }
  return new XMLSerializer().serializeToString(svgEl);
}
```

#### B. Render Integration & Lifecycle:
- In the Mermaid rendering loop, locate `container.querySelector('[data-mermaid-open]')`.
- On render success:
  - If `openLink.dataset.blobUrl` exists, revoke it via `URL.revokeObjectURL()`.
  - Process SVG via `prepareStandaloneSvg(svg)`.
  - Create `new Blob([adaptedSvg], { type: 'image/svg+xml;charset=utf-8' })`.
  - Create URL via `URL.createObjectURL(blob)`, assign to `openLink.href` and `openLink.dataset.blobUrl`.
  - Reveal link (`openLink.hidden = false`).
- On render failure:
  - Ensure `openLink.hidden = true`.

#### C. Click Interceptor Exemption:
In `app.js`'s global link click interceptor, exempt `blob:` URLs from session token query appending:
```javascript
if (!href || href.startsWith('#') || href.startsWith('javascript:') || href.startsWith('vscode:') || href.startsWith('mailto:') || href.startsWith('blob:')) {
  return;
}
```
*Rationale:* Appending `?token=...` to a Blob URL corrupts the browser's in-memory registry key, triggering `net::ERR_FILE_NOT_FOUND`.

### 3. Visual Styling (`src/viewer/assets/app.css`)

Style `.diagram-open-link` consistent with Lens's warm editorial aesthetic:

```css
.diagram-open-link {
  display: inline-block;
  margin-top: .75rem;
  color: #8b3f21;
  font-family: system-ui, sans-serif;
  font-size: .85rem;
  font-weight: 600;
  text-decoration: underline;
  text-underline-offset: 2px;
}
.diagram-open-link:hover {
  color: #5a2712;
}
```

### 4. Page Security & Isolation (`src/viewer/page.rs`)

- Existing Content Security Policy (`default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'`) is preserved without modification.
- Standalone SVG views inherit the origin's CSP; `style-src 'self' 'unsafe-inline'` ensures Mermaid SVG inline `<style>` blocks apply correctly.
- Top-level navigation to `blob:` is permitted under standard CSP navigation rules.
- `inject_capability()` does not touch `blob:` URLs, maintaining clean token separation.

## Testing Strategy

All tests adhere to `<condition_or_action>_then_<observable_result>` naming and Arrange / Act / Assert (3A) structure.

### 1. Unit Tests (`src/markdown.rs`)

- **`mermaid_block_then_emits_open_svg_link_in_placeholder`:**
  - `// Arrange`: Markdown string containing a fenced ````mermaid block.
  - `// Act`: Render the document via `render()`.
  - `// Assert`: Verify HTML contains `<a class="diagram-open-link" data-mermaid-open target="_blank" rel="noopener noreferrer" hidden>Open SVG</a>`.
- **`mixed_plantuml_and_mermaid_document_then_emits_open_svg_only_on_mermaid`:**
  - `// Arrange`: Markdown string with both PlantUML and Mermaid fenced blocks.
  - `// Act`: Render the document.
  - `// Assert`: Verify Mermaid figure contains `data-mermaid-open` and PlantUML figure retains standard `data-diagram` without regression.

### 2. Browser Integration Tests

- **SVG Resizing & Uncapped Viewport:**
  - Verify standalone SVG opened from `blob:` URL has `width="100%"`, `height="100%"`, and no `max-width` inline style.
  - Verify resizing browser window expands SVG bounding client rect proportionally.
- **Background Contrast:**
  - Verify standalone SVG specifies `#ffffff` background, preventing illegible dark text against dark canvas viewers.
- **Failure State Gating:**
  - Given invalid Mermaid syntax, verify `data-mermaid-open` link remains hidden while error text and raw source `<details>` are displayed.
- **Click Interceptor Verification:**
  - Verify clicking `Open SVG` navigates to exact `blob:` URL without `?token=` parameter appended.

## Boundaries

- **Always do:**
  - Operate 100% offline with zero external network requests.
  - Ensure the "Open SVG" link opens in a new browsing context (`target="_blank"`) with `rel="noopener noreferrer"`.
  - Keep the link hidden until Mermaid rendering succeeds.
  - Strip `maxWidth` and set 100% width/height so standalone SVGs scale freely with the window.
  - Exclude `blob:` URLs from `app.js` token query appending.
  - Format with `cargo fmt --check` and pass `cargo clippy --locked --all-targets --all-features -- -D warnings`.
  - Structure all tests with `// Arrange`, `// Act`, and `// Assert`.
- **Ask first:**
  - Adding external dependencies to `Cargo.toml`.
  - Adding server routes for Mermaid SVG caching.
- **Never do:**
  - Use `data:` URLs for top-level navigation (blocked by modern Chromium security policies).
  - Weaken Content Security Policy headers in `page.rs`.
  - Cause malformed Mermaid blocks to break document rendering.
  - Write to or mutate files outside `/home/ccwu/Projects/lens.worktrees/feat-mermaid-svg-view`.

## Acceptance Criteria

1. **Standalone SVG View:** Every successfully rendered Mermaid diagram displays an "Open SVG" link. Clicking the link opens the diagram in a separate browser tab as a pure SVG document (`image/svg+xml`).
2. **Responsive Window Scaling:** Resizing the browser window in the standalone SVG view dynamically and proportionally scales the diagram without being capped by inline `max-width` styles.
3. **High-Contrast Readability:** The standalone SVG includes an explicit white background (`#ffffff`) to ensure readability on dark browser canvas viewers.
4. **Link Gating:** The "Open SVG" link remains hidden if Mermaid rendering fails or has not yet completed.
5. **No URL Parameter Corruption:** Clicking the "Open SVG" link navigates directly to the clean Blob URL without having `?token=...` appended.
6. **Zero Server State & 100% Offline:** The feature operates entirely client-side without server roundtrips or internet access.
7. **Regression Safety:** All 128 existing unit tests and 6 CLI integration tests continue to pass.

## User Review Gate

Before proceeding to Phase 2 (Technical Implementation Plan), this specification must be reviewed and approved by Sirius / Engineering Lead.
