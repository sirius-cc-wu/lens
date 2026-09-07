# Spec: Client-Side Mermaid Diagram Rendering

## Objective

Extend [Lens](file:///home/ccwu/lens/README.md) to render fenced ````mermaid code blocks inside repository Markdown documents as visual SVG diagrams directly in the browser.

Mermaid is widely used across GitHub-flavored Markdown documentation. Lens currently renders only PlantUML diagrams and displays Mermaid blocks as raw code blocks. This feature enables developers and technical writers to view Mermaid diagrams seamlessly alongside PlantUML and Markdown text, working completely offline with zero external server dependencies or configuration.

## Tech Stack

- **Language / Runtime:** Rust 1.75+ (2021 edition)
- **Markdown Parsing:** `pulldown-cmark` (v0.9.3)
- **HTTP / Loopback Server:** `axum` (v0.6.20)
- **Diagram Rendering:** Client-side [Mermaid](https://mermaid.js.org/) JavaScript library (vendored and embedded into the binary via `include_str!`)
- **Browser Runtime:** Modern browser (Chromium, Firefox, Safari)

## Commands

- **Build:** `cargo build --locked`
- **Test:** `cargo test --locked`
- **Lint:** `cargo clippy --locked --all-targets --all-features -- -D warnings`
- **Format:** `cargo fmt --check`
- **Dev Run:** `cargo run -- <path-to-markdown-file>`

## Project Structure

```text
src/
├── markdown.rs              # Recognizes ```mermaid fenced blocks, emits Mermaid diagram placeholders
├── viewer/
│   ├── assets/
│   │   ├── app.css          # Styles for Mermaid diagram containers and failure views
│   │   ├── app.js           # Client-side Mermaid initialization, rendering, and failure fallback
│   │   └── mermaid.min.js   # Vendored client-side Mermaid library (offline asset)
│   ├── page.rs              # Embeds mermaid.min.js, injects script tag, adjusts CSP style-src
│   └── routes.rs            # Serves /mermaid.js with session token authentication
docs/
└── features/
    └── markdown-viewing/
        └── mermaid-rendering-spec.md  # This specification
```

## Code Style

Follow repository guidelines in [AGENTS.md](file:///home/ccwu/lens/AGENTS.md):
- Preserve cohesive module boundaries; keep test code with the module owning the behavior.
- Use plain language on first use for domain terms.
- Tests must follow `<condition_or_action>_then_<observable_result>` naming and 3A (`// Arrange`, `// Act`, `// Assert`) structure.

Example test style:
```rust
#[test]
fn markdown_with_mermaid_code_block_then_emits_mermaid_diagram_placeholder() {
    // Arrange
    let markdown = "```mermaid\ngraph TD;\nA-->B;\n```";
    let resolver = SourceLinkResolver::new(Path::new("/repo"), Path::new("/repo"));
    let known = BTreeSet::new();

    // Act
    let rendered = render(markdown, 0, "doc.md", Path::new("doc.md"), &known, &resolver);

    // Assert
    assert!(rendered.html.contains(r#"class="diagram mermaid-diagram""#));
    assert!(rendered.html.contains(r#"data-mermaid-container"#));
}
```

## Testing Strategy

- **Unit Tests (`src/markdown.rs`):**
  - Verify fenced ````mermaid blocks produce the expected HTML container, error element, and raw source `<details>`.
  - Verify case-insensitivity of language identifier (e.g., ````MERMAID).
  - Verify documents with mixed PlantUML and Mermaid blocks parse each correctly.
  - Verify empty or whitespace-only Mermaid blocks behave predictably.
- **Route & Page Tests (`src/viewer/routes.rs`, `src/viewer/page.rs`):**
  - Verify `/mermaid.js` route requires valid session token and responds with HTTP 200 and `application/javascript`.
  - Verify document HTML contains `<script src="/mermaid.js?token=...">`.
  - Verify Content Security Policy header permits Mermaid script from `'self'` and required inline SVG styling (`style-src 'self' 'unsafe-inline'`).
- **End-to-End Manual Verification:**
  - Open a Markdown document containing valid flowchart, sequence, and class diagrams; verify browser renders SVG diagrams.
  - Test syntax error in Mermaid code block; verify graceful error banner and expander with source code.
  - Test automatic refresh on file change; verify Mermaid diagram updates when source file is modified and saved.

## Boundaries

- **Always do:**
  - Ensure zero external network calls at runtime when rendering Mermaid (fully offline).
  - Maintain the existing PlantUML diagram rendering pipeline without regression.
  - Format with `cargo fmt --check` and pass `cargo clippy --locked --all-targets --all-features -- -D warnings`.
  - Keep tests labeled with Arrange / Act / Assert and behavior-driven test names.
- **Ask first:**
  - Adding any new Rust crate dependencies to `Cargo.toml`.
  - Expanding scope to include standalone `.mmd` / `.mermaid` files.
- **Never do:**
  - Load Mermaid from a remote CDN (e.g., jsdelivr/unpkg) at runtime.
  - Break or weaken Content Security Policy script isolation (`script-src` must remain `'self'`).
  - Cause a malformed Mermaid diagram to break or blank the rest of the Markdown document.

## Acceptance Criteria

1. **Fenced Code Block Detection:** Markdown documents with ````mermaid fenced blocks render those blocks as Mermaid diagram containers instead of generic `<pre><code>` blocks.
2. **Visual Rendering:** In the browser, valid Mermaid code renders into clean SVG diagrams.
3. **Graceful Error Handling:** If Mermaid code has invalid syntax, an actionable error message appears and the raw diagram source remains readable via `<details class="diagram-source">`.
4. **Auto-Refresh Support:** Editing a Mermaid diagram in a local Markdown file refreshes the document view and re-renders the diagram without requiring manual restart.
5. **No External Network Requests:** The entire rendering process works without internet access.
6. **Security & CSP:** Script execution remains restricted to `'self'`; token-based capability authentication protects the `/mermaid.js` endpoint.
7. **Regression Safety:** All 129 existing unit and integration tests continue to pass.

## User Review Gate

Before proceeding to Phase 2 (Technical Implementation Plan), the specification must be reviewed and approved by the user.
