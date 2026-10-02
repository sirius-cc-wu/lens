---
type: "Architecture Decision"
title: "ADR-026: Offline Mathematical Formula Rendering via Bundled KaTeX and Delimiter Preservation"
description: "Renders inline and display LaTeX mathematical formulas in Markdown documents using bundled client-side KaTeX with zero external network dependencies, preserving raw LaTeX syntax during Markdown parsing."
id: "ADR-026"
status: "accepted"
date: "2026-10-02"
tags: [architecture, decision, markdown, math, katex, rendering, offline, security]
---

# ADR-026: Offline Mathematical Formula Rendering via Bundled KaTeX and Delimiter Preservation

Status: accepted

Date: 2026-10-02

## Context

Lens serves as a focused human-review surface for repository specifications, architecture decision records, and technical documentation. Systems engineering, automotive firmware, network protocol design, and control systems specifications frequently contain mathematical formulations:
- Physical layer wire limits, frame durations, and timing budgets ($T_{\text{frame}}$, $t_{\text{bit}}$, $\mu\text{s}$).
- Control loop frequencies and headroom calculations (e.g. 4 pkts/ms $\to$ 4 kHz vs. 100–200 Hz).
- Protocol state transitions, cryptographic notations, matrices, and flow formulas ($\xrightarrow{\text{IPCF}}$).

Currently, Lens parses Markdown documents via the standard Markdown parser (`pulldown-cmark`, adhering to the CommonMark specification) and renders the output in the browser with support for PlantUML (ADR-001/017) and Mermaid (ADR-023). However, Lens provides no mathematical typesetting engine. When reviewers inspect technical specifications, formulas appear either as raw unrendered LaTeX markup (`$T_{\text{frame}}$`, `$$...$$`) or, worse, become corrupted when the Markdown parser interprets LaTeX characters (such as underscores `_` as italics `<em>`, asterisks `*` as bold, or backslashes `\` as escapes).

Lens requires a native mathematical formula rendering capability that satisfies these constraints:
1. **100% Offline & Loopback-Only**: Lens must never make external network calls or load remote scripts/fonts from CDNs at runtime.
2. **Instantaneous & Synchronous Rendering**: Document review in Lens relies on file-watch polling and seamless re-rendering on save. Rendering must not suffer from asynchronous typesetting delays, layout shifts, or font flicker.
3. **Ecosystem Parity**: Authors write Markdown targeting GitHub, VS Code, and GitLab, all of which use KaTeX for math typesetting (`$..$` and `$$..$$`).
4. **Minimal Binary Footprint**: All assets must be bundled into the compiled binary via `include_str!` without multi-megabyte asset bloat.

### Technology Evaluation: MathJax vs. KaTeX

- **MathJax (v3/v4)**:
  - *Strengths*: Near 100% coverage of esoteric LaTeX macros; MathML output.
  - *Weaknesses*: Heavyweight (multi-megabyte footprint); asynchronous typesetting pipeline (`MathJax.typesetPromise()`) that complicates live-refresh DOM replacement; complex font chunk loading that requires extensive routing or custom packaging when embedded offline.
- **KaTeX**:
  - *Strengths*: Extremely fast and synchronous (`katex.renderToString` / `renderMathInElement`); lightweight (~280 KB minified JS); standalone CSS with inlined WOFF2 fonts; industry standard for GitHub-Flavored Markdown and VS Code; covers 99.9% of engineering math syntax.
  - *Weaknesses*: Lacks support for obscure LaTeX packages, which are rarely or never found in Markdown engineering documentation.

KaTeX is the optimal architectural match for Lens.

---

## Decision

Lens adopts **KaTeX** (version 0.16.11) as its client-side mathematical typesetting engine, bundled entirely within the binary, served over authenticated loopback routes, and paired with a syntax-preserving pre-parser pipeline in `src/markdown.rs`.

### 1. Mathematical Delimiters & Syntax Support

Lens supports three standard Markdown mathematical formula formats:
1. **Inline Math (`$formula$`)**:
   - Delimited by single dollar signs.
   - Enforces strict CommonMark / GitHub delimiter boundaries:
     - The opening `$` must not be immediately followed by whitespace (e.g. `$ 100` is currency, `$x+y$` is math).
     - The closing `$` must not be immediately preceded by whitespace (e.g. `100 $` is currency).
     - Currency references with adjacent numbers (e.g. `$10 and $20`) and escaped dollar signs (`\$`) are preserved as literal characters and not parsed as math.
2. **Display Math (`$$formula$$`)**:
   - Delimited by double dollar signs, rendered on a distinct centered block line.
3. **Fenced Math Code Blocks (````math ... ````)**:
   - Supported natively in `src/markdown.rs` alongside `plantuml` and `mermaid` blocks.
   - Formatted as a block-level display equation.

### 2. Context-Aware Pre-Parser Protection (`src/markdown.rs`)

To prevent the Markdown parser (`pulldown-cmark`) from mangling LaTeX expressions (such as turning `T_{\text{frame}}` into `T<em>\text{frame}</em>` or swallowing backslashes), while avoiding false matches inside code blocks or code spans:

1. **Phase 1 (Code & Span Masking)**:
   - The document pre-processor identifies all protected regions: fenced code blocks (``` ````), indented code blocks, inline code spans (`` `...` ``), HTML comments, and Markdown link URLs.
   - Protected regions are temporarily replaced with inert alphanumeric sentinels so their contents (which may include shell variables `$VAR` or literal `$`) are never matched by math scanners.
2. **Phase 2 (Math Delimiter Extraction)**:
   - Within the remaining unprotected prose and table cells, math delimiters (`$$...$$` and `$..$`) are extracted.
   - Each extracted formula is replaced by an HTML placeholder tag with the raw LaTeX string safely HTML-escaped:
     ```html
     <span class="math-inline" data-math-inline="{escaped_tex}"></span>
     ```
     and
     ```html
     <div class="math-display" data-math-display="{escaped_tex}"></div>
     ```
3. **Phase 3 (Sentinel Restoration)**:
   - Code spans, code blocks, and links are restored from their sentinels.
4. **Phase 4 (Fenced ````math Code Blocks & AST Generation)**:
   - `pulldown-cmark` processes the document structure (Abstract Syntax Tree, or AST).
   - Fenced ````math blocks are intercepted in `src/markdown.rs` during AST traversal:
     `Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(language)))` matching `"math"`.
   - Emitted as a display container:
     ```html
     <div class="math-block" data-math-block>
       <div class="math-target"></div>
       <p class="math-error" hidden>Formula rendering failed.</p>
       <details class="math-source" hidden><summary>Formula source</summary><pre><code>{escaped_tex}</code></pre></details>
     </div>
     ```
   - Because `pulldown-cmark` passes HTML tags through without parsing their inner content as markdown, formulas in table cells (`| $T_{\text{frame}}$ | ... |`), inline prose, and multiline display blocks retain complete typographical fidelity.

### 3. Bundled Offline Assets & Self-Contained Font Delivery

Lens embeds KaTeX 0.16.11 assets into the executable at compile time:
- `src/viewer/assets/katex.min.js`: Vendored KaTeX script (~280 KB).
- `src/viewer/assets/katex.min.css`: Vendored KaTeX stylesheet with self-contained, base64-inlined WOFF2 fonts (~850 KB total). Inlining fonts as data URIs eliminates external font HTTP routes, eliminates MIME type configuration, and guarantees 100% offline font availability without network overhead.

Lens exposes authenticated loopback routes in `src/viewer/routes.rs`:
```text
GET /katex.js?token={session_token}
GET /katex.css?token={session_token}
```
All routes require a valid session token matching the viewer instance.

### 4. Content Security Policy (CSP) Update (`src/viewer/routes.rs`, `src/viewer/page.rs`)

To support inlined font data URIs while maintaining strict script and frame isolation, the server Content Security Policy is updated:
```text
default-src 'self'; base-uri 'none'; font-src 'self' data:; img-src 'self' data:; object-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'
```
- Adding `font-src 'self' data:` allows KaTeX's base64-inlined WOFF2 glyphs to load without browser violation reports.
- `script-src` strictly remains `'self'`, prohibiting third-party code.

`src/viewer/page.rs` injects the stylesheet and script tags into the document HTML shell:
```html
<head>
  ...
  <link rel="stylesheet" href="/katex.css?token={token}">
  <link rel="stylesheet" href="/app.css?token={token}">
</head>
<body>
  ...
  <script src="/katex.js?token={token}"></script>
  <script src="/mermaid.js?token={token}"></script>
  <script src="/app.js?token={token}"></script>
</body>
```

### 5. Client-Side Rendering, Security Limits & Error Resilience (`src/viewer/assets/app.js`)

In `src/viewer/assets/app.js`:
- On page load and after every document live-refresh update:
  - Scan all `[data-math-inline]` and `[data-math-display]` elements, as well as `[data-math-block]` containers.
  - Render each formula synchronously with strict security constraints:
    ```javascript
    katex.render(source, targetElement, {
      displayMode: isDisplay,
      throwOnError: false,
      trust: false,       // Strictly prohibit \href, \url, and arbitrary HTML injection
      maxSize: 500,       // Bound font expansion to prevent layout denial-of-service
      maxExpand: 1000,    // Bound recursive macro expansion to prevent main-thread hangs
      strict: "warn"
    });
    ```
- **Fault Tolerance**:
  - If a formula contains invalid LaTeX syntax, KaTeX's error output or a clean warning indicator is displayed, and the raw formula source is accessible in a collapsible `<details>` element.
  - A malformed formula must never crash the JavaScript engine, blank the document, or interrupt PlantUML/Mermaid diagram rendering.

---

## Consequences

- **Review Quality**: Technical, mathematical, and timing analysis documents render with publication-grade mathematical typography directly in Lens.
- **Offline Independence**: Retains 100% local operation without runtime internet access or third-party web services.
- **Markdown Parity**: Engineering specifications written for GitHub or VS Code render identically inside Lens.
- **Security Boundaries**: `trust: false`, resource caps, and CSP `font-src 'self' data:` preserve robust isolation against malicious markup.
- **Zero Cargo Dependency Bloat**: KaTeX is integrated client-side as an embedded static asset, requiring no native C/C++ libraries, Node runtimes, or additional Rust dependencies in `Cargo.toml`.
