# Spike Report: Native Math via pulldown-cmark 0.13.4 (PB-6)

**Question:** Can Lens replace the hand-written math pre-parser (`src/markdown/math.rs`, 1,106 lines, 7 Gate 2 rounds) with pulldown-cmark's native `ENABLE_MATH`? What else changes from upgrading 0.9.3 → 0.13.4?

**Method:** A side-by-side probe crate ran both versions over 47 edge cases and 147 real documents (Lens docs, README, fdc-vs docs). The Builder ran it and I spot-checked the raw output. Zero changes to Lens or the frozen branch.

## Verdict

**Feasible, and recommended.** All 16 code, link and container cases that broke the pre-parser across Gate 2 now come out right, because the parser that renders the page is also the one that decides what is math. The remaining differences are delimiter-policy questions, plus three migration hazards that are known and can be contained in a single module.

## 1. What native math solves (observed)

Each of these was a Gate 2 finding against the pre-parser. Under 0.13.4 + `ENABLE_MATH`, each keeps its code, link or autolink text exactly, and the following `$z$` renders:

| Case | Result |
| :--- | :--- |
| Code spans, fenced code, indented code | Protected |
| List-nested fence, ordered-list (`10.`) fence | Protected |
| Blockquote fence, **unclosed** blockquote fence, nested-quote indented code | Protected; container ends where CommonMark says it does |
| False close (`- ~~~` inside a fence) | Protected |
| Autolinks: `https`, `ftp`, email with `$` | Protected |
| Inline link destinations, reference definitions | Protected |
| HTML comments | Protected |
| Escaped backticks (`` \` ``) | Literal; `$x$` between them still renders |
| Footnotes, headings, link text, table cells (inline and `$$`) | Math renders |
| Asterisks and underscores inside math | Preserved verbatim |
| Display math followed immediately by prose and `**bold**` | Both render |
| CRLF paragraph breaks | No pairing across paragraphs |
| Mermaid / PlantUML fences containing `$a$` | Untouched |

**Real corpus:** 28 math events in 6 files, all intended LaTeX. **Zero false positives.** Shell (`$(...)`, `"$x"`, `$?`), prices and currency stayed literal.

## 2. Behavioral differences from the current spec (observed)

| # | Input | 0.13.4 result | Current spec says | Severity |
| :--- | :--- | :--- | :--- | :--- |
| D1 | `Range $5-$10` | `InlineMath("5-")` | literal (closer followed by digit) | Low: rare; author writes `\$` |
| D2 | `$y$2` | `InlineMath("y")` then `2` | literal | Low |
| D3 | `\| $\|x\|$ \|` in a table | cell split at `\|` | `&#124;`-protected | Low: GitHub behaves the same |
| D4 | ```` ```math ```` | ordinary `CodeBlock(Fenced("math"))` | display math | None: Lens intercepts it like Mermaid/PlantUML today |
| D5 | `$$\verb\|$x$\|$$` | fragments | display math | Negligible |

The 0.13 rules come from commonmark-hs (GitHub-compatible): no whitespace just inside the delimiters, no unescaped `$` inside, content passed through verbatim. There is **no digit rule**, so D1 and D2 follow. Adding one back would mean post-filtering parser output, which reintroduces exactly the heuristic layer we are removing.

## 3. Migration hazards

| # | Hazard | Evidence | Containment |
| :--- | :--- | :--- | :--- |
| **H1** | **Security (High):** Since 0.10, inline HTML arrives as `Event::InlineHtml`, not `Event::Html`. Lens today escapes only `Event::Html`. A naive upgrade passes `<img src=x onerror=alert(1)>` through raw. | **Observed** (`inline-html` case) | Escape both `Html` and `InlineHtml`; add a regression test for both |
| **H2** | **Integrity (Medium):** 0.13 no longer escapes `"` in body text. `inject_capability` string-scans the whole rendered HTML for `href="/documents/`. A code block that shows such text would get the **session token appended to visible code text**. | Escaping change **observed** (35 files); injector impact **inferred from code** | Make `inject_capability` strictly tag-aware via single-pass tokenization; add test |
| **H3** | **Scope creep:** `Options::all()` grows from 6 to 15 flags (subscript, superscript, wikilinks, definition lists, GFM alerts, YAML/plus metadata blocks, old footnotes, math). No corpus diff today, but `~` and `^` semantics and frontmatter handling would silently change on future documents. | **Observed** flag sets | Pin an explicit set: the current 6 flags + `ENABLE_MATH` |
| **H4** | **Test churn:** 35/147 files change HTML **only** in `&quot;` → `"` inside code and text. Zero structural differences. | **Observed** | Expect updates to existing `markdown.rs` assertions that expect `&quot;` |
| **H5** | **API migration:** `Tag`/`TagEnd` split, `Tag::Link` becomes a struct variant, end-tag matching for `CodeBlock`/`Table`/`Link`. | Release notes; usage confined to `src/markdown.rs` (`render` is about 130 lines on `main`) | One module; MSRV 1.71.1 ≤ Lens 1.75 ✓ |
| **H6** | **Security (High):** 0.13.4 heading attributes accept arbitrary `key=value` pairs (`# T {onclick=x style=... data-diagram}`), whereas 0.9.3 only accepted `#id` and `.class`. | Source audit (`firstpass.rs:2540`) | Strip heading custom attributes in `src/markdown.rs`, preserving only `id` and `classes` (Invariant I4) |

## 4. Salvage plan for the frozen branch

| Keep (adapt) | Discard |
| :--- | :--- |
| KaTeX 0.16.22 assets, `/katex.*` routes, CSP `font-src 'self' data:`, page tags | `src/markdown/math.rs` (whole pre-parser) |
| `app.js` render loop, `.katex-error` detection, `textContent` fallbacks, security options | Sentinels, placeholder allowlist, `&#10;`/`&#124;` attribute encoding |
| Browser tests for trust/maxSize/maxExpand, error fallbacks, egress, live refresh, tables, fractions | Unit tests for the pre-parser's masking internals |
| `tests/fixtures/math-specification.md` (revisit currency lines under D1) | |
