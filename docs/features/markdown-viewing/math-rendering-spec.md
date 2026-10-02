# Spec: Client-Side Mathematical Formula Rendering (KaTeX)

## Objective

Extend [Lens](../../../README.md) to render inline and display LaTeX mathematical formulas inside repository Markdown documents directly in the browser using bundled, offline [KaTeX](https://katex.org/).

Technical, systems architecture, firmware, and protocol specifications (such as CAN-FD bus budgets, control loop hierarchies, cryptography specifications, and physical wire timings) rely on clear mathematical typography. Currently, Lens displays raw LaTeX strings (`$T_{\text{frame}}$`, `$$...$$`) or produces corrupted output when standard Markdown parsers misinterpret underscores and backslashes. This feature provides crisp, publication-grade math rendering across all Markdown documents while maintaining Lens's strict 100% offline, single-binary architecture.

## Tech Stack

- **Language / Runtime:** Rust 1.75+ (2021 edition)
- **Markdown Parsing:** `pulldown-cmark` (v0.9.3) with syntax-preserving, context-aware math tokenizer in `src/markdown.rs`
- **HTTP / Loopback Server:** `axum` (v0.6.20)
- **Math Rendering Engine:** Client-side KaTeX 0.16.11 (bundled `katex.min.js` and `katex.min.css` with inlined WOFF2 fonts embedded into the binary via `include_str!`)
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
├── markdown.rs              # Recognizes math delimiters ($, $$, ```math), masks code spans, preserves raw LaTeX
├── viewer/
│   ├── assets/
│   │   ├── app.css          # Styles for math blocks, inline expressions, and error views
│   │   ├── app.js           # Client-side KaTeX initialization, rendering, and failure fallback
│   │   ├── katex.min.css    # Vendored KaTeX stylesheet with inlined WOFF2 fonts (offline asset)
│   │   └── katex.min.js     # Vendored KaTeX library (offline asset)
│   ├── page.rs              # Embeds katex assets, injects stylesheet and script tags into page shell
│   └── routes.rs            # Serves /katex.js and /katex.css with token auth; updates CSP with font-src
docs/
├── decisions/
│   └── adr-026-mathematical-formula-rendering.md  # Governing ADR
└── features/
    └── markdown-viewing/
        └── math-rendering-spec.md                # This specification
```

## Delimiter Contracts & Preserving Logic

### 1. Supported Delimiters

1. **Inline Math (`$formula$`):**
   - Single dollar signs enclosing LaTeX notation.
   - Enforces strict whitespace and currency boundaries:
     - Opening `$` must not be immediately followed by whitespace (e.g. `$ 100` is currency, `$x+y$` is math).
     - Closing `$` must not be immediately preceded by whitespace (e.g. `100 $` is currency).
     - Currency references with adjacent numbers (e.g. `$10 and $20`) are not parsed as math.
     - Escaped dollar signs (`\$`) are preserved as literal characters and not treated as delimiters.
2. **Display Math (`$$formula$$`):**
   - Double dollar signs enclosing multiline or centered single-line equations.
   - May span multiple lines.
3. **Fenced Code Block (````math ... ````):**
   - GitHub-standard fenced code block with language identifier `math`.
   - Emits a block-level display equation container.

### 2. Context-Aware Pre-Parser Protection

In standard Markdown, underscores `_`, asterisks `*`, and backslashes `\` trigger formatting or escapes. To prevent corruption without inadvertently mangling code or literal variables:
1. **Code Span & Block Masking:** Before math extraction, identify all fenced code blocks (``` ````), indented code blocks, inline code spans (`` `...` ``), HTML comments, and Markdown link URLs. Temporarily substitute them with unique alphanumeric sentinels.
2. **Math Extraction:** Extract `$$...$$` and `$..$` from the remaining text. Replace each formula with an HTML-escaped placeholder `<span class="math-inline" data-math-inline="{escaped_tex}"></span>` or `<div class="math-display" data-math-display="{escaped_tex}"></div>`.
3. **Sentinel Restoration:** Restore masked code blocks and spans prior to Markdown AST parsing.
4. **Fenced ````math Interception:** Intercept fenced code blocks labeled `math` during AST event traversal, emitting display containers `<div class="math-block" data-math-block>...</div>`.

## Security & Content Security Policy (CSP)

1. **CSP Update:** The server Content Security Policy in `src/viewer/routes.rs` is updated to include `font-src 'self' data:`:
   ```text
   default-src 'self'; base-uri 'none'; font-src 'self' data:; img-src 'self' data:; object-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'
   ```
2. **KaTeX Execution Boundaries:** In `src/viewer/assets/app.js`, KaTeX must be initialized with:
   - `trust: false` (strictly preventing `\href` or script injection).
   - `maxSize: 500` (bounding font expansion against layout denial-of-service).
   - `maxExpand: 1000` (bounding macro recursion to prevent browser main-thread hangs).
   - `throwOnError: false` (preventing unhandled exceptions).

## Code Style

Follow repository guidelines in [AGENTS.md](../../../AGENTS.md):
- Preserve cohesive module boundaries; keep test code with the module owning the behavior.
- Tests must follow `<condition_or_action>_then_<observable_result>` naming and 3A (`// Arrange`, `// Act`, `// Assert`) structure.

Example test style:
```rust
#[test]
fn markdown_with_inline_math_then_emits_math_placeholder_with_preserved_tex() {
    // Arrange
    let markdown = "The duration is $T_{\\text{frame}}$ in microseconds.";
    let resolver = SourceLinkResolver::new(PathBuf::from("/repo"));
    let known = BTreeSet::new();

    // Act
    let rendered = render(markdown, 0, "doc.md", Path::new("doc.md"), &known, &resolver);

    // Assert
    assert!(rendered.html.contains(r#"class="math-inline""#));
    assert!(rendered.html.contains(r#"data-math-inline="T_{\text{frame}}""#));
    assert!(!rendered.html.contains("<em>")); // Verifies underscore was not mangled into italics
}
```

## Testing Strategy

- **Unit Tests (`src/markdown.rs`):**
  - Verify inline math `$x+y$` produces `<span class="math-inline" ...>`.
  - Verify underscores and asterisks inside math are not converted to `<em>` or `<strong>`.
  - Verify currency strings (e.g. `$10 and $20`) are not parsed as math.
  - Verify code spans containing dollar signs (e.g. `` `$x$` `` or `` `echo $PATH` ``) are not converted to math placeholders.
  - Verify escaped dollar signs `\$5` are treated as literal text.
  - Verify display math `$$...$$` produces `<div class="math-display" ...>`.
  - Verify fenced ````math code blocks produce `<div class="math-block" ...>`.
  - Verify math formulas inside Markdown table cells (`| Formula | $E=mc^2$ |`) parse cleanly.
  - Verify mixed documents with PlantUML, Mermaid, and KaTeX parse with zero cross-talk.
- **Route & Page Tests (`src/viewer/routes.rs`, `src/viewer/page.rs`):**
  - Verify `/katex.js` and `/katex.css` routes require valid session token and return HTTP 200 with correct Content-Type.
  - Verify unauthenticated requests return 401 Unauthorized.
  - Verify document HTML includes `<link rel="stylesheet" href="/katex.css?token=...">` and `<script src="/katex.js?token=...">`.
  - Verify Content Security Policy header permits KaTeX resources under `'self'` and fonts under `data:`.
- **Browser & End-to-End Verification:**
  - Verify browser renders inline equations, display equations, fractions, subscripts, and greek letters.
  - Verify malformed LaTeX (e.g. `$\invalid{macro$`) displays a non-fatal error without crashing the page or blocking document live-refresh.
  - Verify documents with both Mermaid diagrams and KaTeX formulas render both cleanly.

## Boundaries

- **Always do:**
  - Ensure zero external network calls at runtime when rendering KaTeX (fully offline).
  - Preserve all existing PlantUML, Mermaid, and source code viewer features without regression.
  - Format with `cargo fmt --check` and pass `cargo clippy --locked --all-targets --all-features -- -D warnings`.
  - Keep tests labeled with Arrange / Act / Assert and behavior-driven test names.
- **Ask first:**
  - Adding any new Rust crate dependencies to `Cargo.toml`.
- **Never do:**
  - Load KaTeX or fonts from a remote CDN (e.g. cdnjs, jsdelivr) at runtime.
  - Set `trust: true` in KaTeX configuration.
  - Break or weaken Content Security Policy script isolation (`script-src` must remain `'self'`).
  - Allow invalid LaTeX syntax to blank or break the rest of the Markdown document.

## Acceptance Criteria

1. **Inline Math:** Text enclosed in valid single dollar signs (e.g. `$T_{\text{frame}}$`) renders as a styled inline equation, preserving subscripts and LaTeX commands.
2. **Display Math:** Standalone `$$...$$` and ````math blocks render as centered, display-mode block equations.
3. **Code Span Immunity:** Dollar signs inside inline code spans and code blocks are not intercepted or corrupted.
4. **Table Compatibility:** Math formulas in Markdown table cells render properly without disrupting table formatting or alignment.
5. **Offline Self-Containment:** All JS, CSS, and font assets are compiled into the binary; no external network requests are made.
6. **Live Refresh Support:** Modifying and saving a Markdown file re-renders all math formulas instantaneously without page reload glitches.
7. **Security & Error Resilience:** `trust: false` is enforced; malformed LaTeX displays a gentle inline/block warning while keeping the rest of the document fully readable.
