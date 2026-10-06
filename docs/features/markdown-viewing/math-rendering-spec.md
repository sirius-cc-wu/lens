# Spec: Client-Side Mathematical Formula Rendering (KaTeX)

## Objective

Render inline and display LaTeX formulas in [Lens](../../../README.md) Markdown documents, offline, with bundled [KaTeX](https://katex.org/). Today, formulas such as `$T_{\text{frame}}$` appear as raw LaTeX, or come out corrupted when underscores turn into italics.

Governing decisions:

- [ADR-027](../../decisions/adr-027-native-parser-math-pulldown-cmark-013.md): parsing, the parser upgrade, and security invariants I1–I4.
- [ADR-026](../../decisions/adr-026-mathematical-formula-rendering.md) §3–§5: bundled assets, routes, CSP and KaTeX options.

## Tech Stack

- Rust 1.75+ (2021 edition)
- `pulldown-cmark` `=0.13.4` (`default-features = false`, `features = ["html"]`) with `ENABLE_MATH`
- `axum` 0.6.20
- KaTeX **0.16.22**: `katex.min.js`, plus `katex.min.css` with inlined WOFF2 fonts, embedded with `include_str!`

## Commands

- Build: `cargo build --locked`
- Test: `cargo test --locked`
- Lint: `cargo clippy --locked --all-targets --all-features -- -D warnings`
- Format: `cargo fmt --check`
- Browser: `npm run test:browser` (Playwright, Chromium)

## Project Structure

```text
src/
├── markdown.rs           # LENS_OPTIONS, event mapping (math spans, ```math, alt passthrough, I1, I4)
└── viewer/
    ├── assets/
    │   ├── app.css       # .math-inline / .math-display / .math-block / error styles
    │   ├── app.js        # renderMath(): KaTeX render loop and error fallbacks
    │   ├── katex.min.css # KaTeX 0.16.22 stylesheet, WOFF2 fonts inlined
    │   └── katex.min.js  # KaTeX 0.16.22
    ├── page.rs           # Asset embedding, page tags, CSP, tag-aware inject_capability (I2)
    └── routes.rs         # /katex.js, /katex.css (token-guarded)
tests/
├── browser/lens.spec.mjs
└── fixtures/math-specification.md
```

There is no `src/markdown/math.rs`. No Lens code scans Markdown source for `$`.

## Rules and Examples

Rules use stable IDs. Tests and the task board cite these IDs and do not restate them. `T` stands for the session token.

### R1: The parser decides what is math

Lens treats exactly the `Event::InlineMath` and `Event::DisplayMath` events from `pulldown-cmark` 0.13.4 as math. It adds no delimiter rules and filters none out.

| Example | Expected |
| :--- | :--- |
| `The frame takes $T_{\text{frame}}$.` | One `math-inline` span whose text is `T_{\text{frame}}`; no `<em>` |
| `$a*b*c$ and $x_1_2$` | TeX kept verbatim; no `<em>` or `<strong>` |
| `$$` + newline + `\begin{aligned}…\end{aligned}` + newline + `$$` | One `math-display` span whose text keeps its line breaks |
| `$ 100 and 200 $` | Literal text; no span |
| `It costs $10 and $20.` | Literal text; no span |
| `\$50 and \$100` | Literal `$50 and $100` |
| `` `echo $PATH` `` and `` `$x$` `` | Code spans; no math span |
| A fenced or indented code block containing `$x$`, including inside a list item or blockquote | Code block; no math span |
| `[l](https://a.example/$x$)` and `<https://a.example/$x$>` | The URL keeps `$x$` exactly; no span |
| `<!-- $x$ -->` | Escaped literal text (R5); no span |
| Math in a heading, link text, footnote definition, blockquote, list item or table cell | Span emitted in place |
| `> $x$` (blockquote) and `- $x$` (list item) | Span emitted in place inside `<blockquote>` or `<li>` |
| Mermaid or PlantUML fence containing `$a$` | Diagram source keeps `$a$`; no span |
| `Range $5-$10` (D1) | Span with text `5-`. **Accepted** |
| `$y$2` (D2) | Span `y`, then text `2`. **Accepted** |
| `\| $|x|$ \|` in a table (D3) | Cell splits at unescaped `\|`. **Accepted.** Authors write `\lvert x \rvert` or `\Vert` |
| `$(CC)$(FLAGS)` in prose (D6) | Span `(CC)`, then text `(FLAGS)`. **Accepted** |
| Block starter line inside `$$...$$` (D7) | Paragraph ends; no display math. **Accepted.** Authors use `aligned` or indent 4+ spaces |
| Footnotes under 0.13 `ENABLE_FOOTNOTES` (D9) | GitHub-compatible footnote semantics. **Accepted** |

### R2: Math span markup

- Inline: `<span class="math-inline" data-math-inline>{escape_html(tex)}</span>`
- Display: `<span class="math-display" data-math-display>{escape_html(tex)}</span>`
- `escape_html` is Lens's function in `src/markdown.rs`; it encodes `& < > " '`. Lens adds no other encoding.
- Lens emits these spans only as `Event::Html` and never constructs `Event::InlineHtml`.

| Example | Expected |
| :--- | :--- |
| `$a<b \text{"q"}$` | `<span class="math-inline" data-math-inline>a&lt;b \text{&quot;q&quot;}</span>` |
| `$$x$$` inside a table cell | `<td>` contains `span.math-display`; the table keeps its column count |
| `$$x$$` alone in a paragraph | `<p><span class="math-display" data-math-display>x</span></p>`; no `<div>` inside `<p>` |

### R3: Math in image alt text stays literal

| Example | Expected |
| :--- | :--- |
| `![speed $v$](plot.png)` | `alt="speed $v$"`; output has no `math-inline` |
| `![area $$r^2$$](a.png)` | `alt="area $$r^2$$"` |
| `![a <b> $x$](i.png)` | `alt="a &lt;b&gt; $x$"` |

### R4: Fenced `math` blocks

A fenced block whose info string trims to `math` (case-insensitive) emits:

```html
<div class="math-block" data-math-block><div class="math-target"></div><p class="math-error" hidden>Formula rendering failed. The source is shown below.</p><details class="math-source"><summary>Formula source</summary><pre><code>{escape_html(source)}</code></pre></details></div>
```

| Example | Expected |
| :--- | :--- |
| ```` ```math ```` / ```` ```Math ```` | `math-block` with the escaped source; no `language-math` code block |
| ```` ```math ```` inside a list item or blockquote | `math-block` inside that `<li>` or `<blockquote>` |
| Source containing `<script>` | `&lt;script&gt;` inside `<code>` |
| ```` ```mathematica ```` or ```` ```math extra ```` | Ordinary code block |

### R5: Parser raw HTML is always escaped (I1, security)

| Example | Expected |
| :--- | :--- |
| Block `<div onclick="x">hi</div>` | Escaped text; no `<div onclick` in output |
| Inline `text <img src=x onerror=alert(1)> text` | `&lt;img src=x onerror=alert(1)&gt;`; no `<img src=x` |
| Inline `<span data-math-inline>x</span>` | Escaped text; no `data-math-inline` element |
| `<script>alert('unsafe')</script>` | `&lt;script&gt;`; no `<script>` |

### R6: Capability injection touches tag attributes only (I2, security and integrity)

`inject_capability` tokenizes start tags. Inside matching attributes (`href` on `a` starting with `/documents/`, `src` on `img` starting with `/diagrams/`), `push_capability_url` splits the URL at the actual fragment `#` and query `?` delimiters while correctly distinguishing HTML character references (e.g. `&#x27;` in `/documents/O&#x27;Reilly.md#intro`).

| Example (HTML passed to `inject_capability`) | Expected |
| :--- | :--- |
| `<a href="/documents/a.md#x">` | `href="/documents/a.md?token=T#x"` |
| `<a href="/documents/a.md?v=1">` | `href="/documents/a.md?v=1&token=T"` |
| `<a href="/documents/O&#x27;Reilly.md#intro">` | `href="/documents/O&#x27;Reilly.md?token=T#intro"` (entity not broken) |
| `<img src="/diagrams/0/0" alt="d" data-diagram>` | `src="/diagrams/0/0?token=T"` |
| `<p>href="/documents/a.md"</p>` | Byte-for-byte unchanged |
| `<pre><code>&lt;a href="/documents/a.md"&gt;</code></pre>` | Byte-for-byte unchanged |
| `<a title="x" data-href="/documents/a.md" href="https://e.x">` | Unchanged |
| `<link href="/documents/a.md">`, `<h1 src="/diagrams/0/0">` | Unchanged (wrong element) |
| Markdown `` `<a href="/documents/a.md">` `` and raw `<a href="/documents/a.md">x</a>`, through `render` and then `page` | No `token=` inside the escaped text |

### R7: Pinned parser options (I3)

| Example | Expected |
| :--- | :--- |
| `H~2~O`, `x^2^` | No `<sub>` or `<sup>` |
| `[[Page]]` | No link |
| `Term` + newline + `: definition` | No `<dl>` |
| `> [!NOTE]` + newline + `> text` | Plain `<blockquote>`, no `markdown-alert` class |
| Tables, defined footnotes, `~~strike~~`, task lists, smart quotes, `# H {#id .c}` | Same structure as before the upgrade |

### R8: Client rendering and resilience

- `renderMath()` runs once on load, in its own `try` block, before Mermaid initialization. Live refresh reloads the whole page, so no other hook is needed.
- It renders every `[data-math-inline]`, `[data-math-display]` and `[data-math-block]`:
  - The source is read from text content before rendering: the span's own text, or the block's `.math-source code`.
  - Each call gets a fresh options object: `{displayMode, throwOnError: false, trust: false, maxSize: 500, maxExpand: 1000, strict: "warn"}`, with no `macros` option.
- When KaTeX throws, or its output contains `.katex-error`:
  - **Spans** are replaced with a strictly phrasing-safe fallback: `<span class="math-error" title="Formula rendering failed">Formula error: <code>{source}</code></span>`. CSS controls styling (`display: inline-block` or `display: block`). Spans never insert block elements (`<div>`, `<p>`, `<details>`) that violate phrasing content in `<p>`, `<td>`, or headings.
  - **Blocks** hide `.math-target`, un-hide `.math-error`, and set `.math-source` `open = true`.
- A malformed formula never blanks the document or blocks Mermaid or PlantUML.

### R9: Offline assets, routes and CSP

- `GET /katex.js` returns `text/javascript; charset=utf-8` and `GET /katex.css` returns `text/css; charset=utf-8`. Both need the session token; a missing or wrong token returns 401.
- The page links `/katex.css` before `/app.css`, and loads `/katex.js` before `/mermaid.js` and `/app.js`.
- The CSP is exactly `default-src 'self'; base-uri 'none'; font-src 'self' data:; img-src 'self' data:; object-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'`.
- `katex.min.css` contains no `url(` whose target is not a `data:` URI.
- Rendering makes zero requests to anything but the Lens origin.

### R10: Heading attribute blocks keep only id and classes (I4, security)

| Example | Expected |
| :--- | :--- |
| `# Title {#intro .lead}` | `<h1 id="intro" class="lead">Title</h1>` |
| `# Title {onclick=alert(1) style=position:fixed data-diagram href=/documents/a.md}` | `<h1>Title</h1>`; none of those attributes in output |

### R11: Graceful degradation

| Condition | Expected |
| :--- | :--- |
| **Mode A: KaTeX script unavailable, JavaScript enabled** | Spans show raw TeX; `math-block` shows collapsed "Formula source"; Mermaid interactive SVGs and live refresh work completely |
| **Mode B: JavaScript disabled** | Spans show raw TeX; `math-block` shows collapsed "Formula source"; PlantUML images load; Mermaid SVG and dynamic refresh are inactive (Mermaid source readable in `<details>`) |
| **Mode C: `id="katex"` element without KaTeX engine** | `renderMath()` returns cleanly without throwing |

## Security Boundaries

- **Always:** escape parser `Html` and `InlineHtml` (R5). Strip heading custom attributes (R10). Keep `inject_capability` tag-aware and entity-safe (R6). Keep KaTeX at `trust: false`. Read sources through `textContent`. Keep span error fallbacks phrasing-safe (R8).
- **Ask first:** any other Rust dependency change; any change to the R7 option set; any KaTeX version change; adding `CAPABILITY_ATTRIBUTES` entries outside an accepted ADR.
- **Never:** load KaTeX or fonts from a CDN; set `trust: true`; pass a shared `macros` object; weaken `script-src`; scan Markdown source for `$` in Lens code; use `Options::all()`; construct `Event::InlineHtml` in Lens; use `innerHTML` for fallbacks; insert block elements into math spans.

## Testing Strategy

- **Rust unit tests in `src/markdown.rs`:** at least one per example row in R1–R5, R7 and R10. Name them `<condition_or_action>_then_<observable_result>` and use 3A structure (Arrange, Act, Assert). Existing assertions may change only for the `Tag`/`TagEnd` API migration or `&quot;` → `"` in body text, and each change is listed in the Builder report.
- **Rust unit tests in `src/viewer/page.rs`:** every R6 row (including entity-aware URL handling), the R9 page-tag order, and the R9 `url(` check.
- **Route tests in `src/viewer/routes.rs`:** R9 routes, 401s and the exact CSP.
- **Browser tests in `tests/browser/lens.spec.mjs`:**
  - **In Slice 4 (core client qualification):** math rendering, `{trust: false, maxSize: 500, maxExpand: 1000}`, phrasing-safe error fallbacks (R8), missing KaTeX (R11 Mode A), `id="katex"` guard (R11 Mode C), zero off-origin requests.
  - **In Slice 5 (fixture and integration):** the full fixture's inline, display, aligned, block and table formulas; `\lvert x \rvert` table cell (D3); currency ranges (D1 guidance); live refresh and Mermaid coexistence; JavaScript-disabled degradation (R11 Mode B).

## Acceptance Criteria

1. R1–R11 hold, each with at least one passing test that names the rule in the Builder report.
2. `cargo fmt --check`, `cargo clippy --locked --all-targets --all-features -- -D warnings`, `cargo test --locked` and `npm run test:browser` all pass. The Builder report attaches the logs.
3. PlantUML, Mermaid, links, source links and frontmatter work as before. Pre-existing tests change only as allowed above.
4. The Builder report records SHA-256 of both KaTeX assets matching ADR-027 and the font-inlining check.
