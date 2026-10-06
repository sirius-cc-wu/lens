---
type: "Architecture Decision"
title: "ADR-027: Recognize Math with the Markdown Parser (pulldown-cmark 0.13.4 ENABLE_MATH)"
description: "Replaces ADR-026's hand-written math pre-parser with pulldown-cmark 0.13.4 native math events, pins the parser option set, and records the four security invariants the parser upgrade requires."
id: "ADR-027"
status: "accepted"
date: "2026-10-05"
tags: [architecture, decision, markdown, math, katex, pulldown-cmark, security]
---

# ADR-027: Recognize Math with the Markdown Parser (pulldown-cmark 0.13.4 ENABLE_MATH)

Status: accepted

Date: 2026-10-05

Supersedes: [ADR-026](adr-026-mathematical-formula-rendering.md) §1 (delimiter rules), all of §2 (pre-parser protection, including the Phase 4 container markup), and the attribute-reading part of §5. ADR-026 §3 and §4, and the §5 security options, stay in force with the KaTeX version corrected to 0.16.22.

## Context

ADR-026 put a context-aware pre-parser in front of `pulldown-cmark` 0.9.3. The pre-parser hid code, links and comments behind placeholders, then extracted `$` and `$$` spans. Implementation took seven Gate 2 rounds (Gate 2 is the code qualification review). Two independent reviewers rejected the result for the same reason: the pre-parser is a second CommonMark parser. Wherever it disagrees with `pulldown-cmark` (fences in lists and blockquotes, autolinks, container-relative indentation, unclosed fences), content is silently corrupted. More patching cannot close that gap.

A spike ([PB-6 report](../reviews/pb-6-native-parser-math-spike.md)) compared `pulldown-cmark` 0.13.4 with `ENABLE_MATH` against 47 edge cases and 147 real documents:

- Every code, link and container case that broke the pre-parser came out right, because the parser that renders the page also decides what is math.
- 28 math events across 6 files, with zero false positives.
- The only HTML difference outside math was `&quot;` becoming `"` in body text.

The upgrade also brings four hazards:

| # | Hazard | Source |
| :--- | :--- | :--- |
| H1 | Inline raw HTML arrives as `Event::InlineHtml`, which Lens does not escape today | Observed in spike |
| H2 | Body text no longer escapes `"`; the current `inject_capability` string scan could append the session token to visible code text | Escaping observed; injector impact inferred from code |
| H3 | `Options::all()` grows from 6 flags to 15 | Observed |
| H6 | Heading attribute blocks accept arbitrary `key=value` pairs (`# T {onclick=x style=… data-diagram}`), which 0.9.3 ignored | `firstpass.rs`, `parse_inside_attribute_block`, 0.13.4 |

## Decision

### 1. One parser decides what is math

- Lens depends on `pulldown-cmark = { version = "=0.13.4", default-features = false, features = ["html"] }`. The new transitive crates `pulldown-cmark-escape` 0.11 and `bitflags` 2 are accepted. `getopts` is not pulled in.
- Delimiter recognition is exactly what `pulldown-cmark` 0.13.4 `ENABLE_MATH` does: the commonmark-hs rules, which GitHub also follows. **Lens adds, removes and post-filters no delimiter rules. No Lens code scans Markdown source for `$`.**

### 2. Event mapping in `src/markdown.rs`

| Parser event | Lens output |
| :--- | :--- |
| `InlineMath(tex)` outside an image | `Event::Html`: `<span class="math-inline" data-math-inline>{escape_html(tex)}</span>` |
| `DisplayMath(tex)` outside an image | `Event::Html`: `<span class="math-display" data-math-display>{escape_html(tex)}</span>` |
| `InlineMath` or `DisplayMath` inside an image | Passed through unchanged. The upstream alt-text writer emits `$tex$` or `$$tex$$`, escaped. |
| Fenced code block whose info string trims to `math` (case-insensitive) | `Event::Html`: the `math-block` container (spec R4), with source escaped in `<details class="math-source">` |
| `Html(v)`, `InlineHtml(v)` | `Text(v)` (I1) |
| `Start(Heading { level, id, classes, attrs })` | Same heading with `attrs` emptied (I4) |

Design notes:

- The TeX travels as element text, not as an attribute value. Without JavaScript, or when KaTeX fails to load, readers see the raw formula instead of a gap.
- Display math uses `span`, because the parser emits it inside `<p>` and `<td>`. CSS sets `display: block`.
- Lens tracks image nesting depth, because the upstream alt-text writer drops `Event::Html`. A mapped formula would vanish from the alt text.
- Lens emits its own trusted markup only as `Event::Html`. It never constructs `Event::InlineHtml`.
- Error fallbacks for spans are strictly phrasing-safe (e.g. `<span class="math-error">Formula error: <code>{source}</code></span>`), avoiding illegal block elements (`<div>`, `<p>`, `<details>`) inside `<p>`, `<td>`, or headings. Block-level `<details>` disclosures are used only for fenced `math-block` containers.

### 3. Pinned parser options

Lens passes exactly this set from one constant, and never uses `Options::all()`:

`ENABLE_TABLES | ENABLE_FOOTNOTES | ENABLE_STRIKETHROUGH | ENABLE_TASKLISTS | ENABLE_SMART_PUNCTUATION | ENABLE_HEADING_ATTRIBUTES | ENABLE_MATH`

These stay disabled: subscript, superscript, wikilinks, definition lists, GFM alerts, YAML and plus metadata blocks, and old footnotes. Lens's own frontmatter handling ([ADR-015](adr-015-yaml-frontmatter-rendering.md)) is unchanged. `ENABLE_FOOTNOTES` selects 0.13's GitHub-compatible footnote semantics (accepted behavior D9). The spike corpus showed no difference from 0.9.3, and tests pin defined-footnote and undefined-footnote output.

### 4. Security invariants (they block the upgrade)

- **I1: No parser raw HTML reaches the page.** Both `Event::Html` and `Event::InlineHtml` from the parser become escaped text. Only Lens templates are emitted as raw HTML.
- **I2: Capability injection never changes text content and preserves URL semantics.** `inject_capability` (`src/viewer/page.rs`) is a single-pass tokenizer that understands tags. It appends the session token only to:
  - an `href` attribute value on an `a` start tag, when the value starts with `/documents/`;
  - a `src` attribute value on an `img` start tag, when the value starts with `/diagrams/`.

  Text between tags is copied byte for byte, by construction. Inside target URLs, `push_capability_url` splits at the actual fragment `#` and query `?` delimiters while correctly distinguishing HTML character references (e.g. `&#x27;` in `/documents/O&#x27;Reilly.md#intro`), preserving the decoded URL destination without corrupting paths.
  I2 is sound because of I1. After I1, every `<` in rendered HTML starts a real tag, and every attribute value is double-quoted with no raw `"` or `>`: parser text, `escape_html`, `escape_href` and Lens's `escape_html` all encode them.
- **I3: The option set is pinned** as in §3.
- **I4: Heading attribute blocks produce only `id` and `class`.** Custom attributes are discarded, restoring 0.9.3 behavior. Without I4, authors could forge `style` (allowed by `style-src 'unsafe-inline'`), `href` or `src` (capability-carrying), or `data-*` markers that `app.js` acts on.

### 5. KaTeX, routes and CSP (ADR-026 §3–§5, version corrected)

- Bundled KaTeX **0.16.22**:
  - `katex.min.js`: identical to npm `katex@0.16.22/dist/katex.min.js` (SHA-256: `e8d885505949f3a5f4abdd5dd0d53696bd1371ad26ffbf4f310dcd77c8cdae89`).
  - `katex.min.css`: `dist` stylesheet with every font `url()` replaced by a `data:font/woff2;base64` URI of the matching `dist/fonts` file (SHA-256: `05f52c1d80561bc3d1024881edd88c25e49352d4ee08493d2f912c27d2ef7a12`).
  - Salvage provenance: ported from `feat/offline-math-rendering` commit `394d7ea4ba`.
- The token-guarded routes `/katex.js` (`text/javascript; charset=utf-8`) and `/katex.css` (`text/css; charset=utf-8`).
- The CSP adds `font-src 'self' data:`. `script-src` stays `'self'`.
- `app.js` renders with a fresh options object per formula: `{displayMode, throwOnError: false, trust: false, maxSize: 500, maxExpand: 1000, strict: "warn"}`, with no shared `macros` object. Error fallbacks are built with phrasing-safe DOM elements using `textContent` only.

### 6. Accepted behaviors

These follow from §1. They are documented for authors and pinned by tests, but not "fixed" in code.

| # | Input | Result | Author guidance |
| :--- | :--- | :--- | :--- |
| D1 | `Range $5-$10` | `5-` renders as math | Write `\$5-\$10` |
| D2 | `$y$2` | `y` renders as math, followed by `2` | None needed |
| D3 | `\| $|x|$ \|` in a table | The cell splits at unescaped `\|` (GitHub behaves the same). Escaping as `$\|x\|$` strips the backslash in TeX yielding `|x|`. | Use `\lvert x \rvert` or `\Vert` for norms |
| D4 | ```` ```math ```` | Intercepted by Lens as a `math-block` (§2) | None |
| D5 | `$$\verb\|$x$\|$$` | Fragments | Avoid `$` inside `\verb` |
| D6 | `$(CC)$(FLAGS)` written in prose | `(CC)` renders as math | Put shell and Make expressions in code spans |
| D7 | A line starting with a block marker (`- `, `+ `, `> `, `1.`, `#`, `===`, or blank line) inside `$$ … $$` | The paragraph ends; no display math | Keep display math free of block markers, use `\begin{aligned} ... \end{aligned}`, or indent continuation lines 4+ spaces |
| D8 | `$x$` in YAML frontmatter | Literal | None |
| D9 | Footnote syntax under 0.13 `ENABLE_FOOTNOTES` | GitHub-compatible: consecutive `[^a]:` lines separate; undefined `[^x]` renders as literal text; indented continuation paragraphs stay inside footnote | Follow standard GitHub Flavored Markdown footnote conventions |

## Alternatives considered

- **Keep patching the pre-parser.** Rejected: a second parser can't be made to agree with the first by patching.
- **Post-filter parser math with a digit rule (D1, D2).** Rejected: it brings back the heuristic layer this ADR removes.
- **Inject the token at the event level by passing it into `render`.** Rejected for C19: it couples the cached rendering to the session and touches every caller. Tag-aware injection keeps rendered HTML token-free and needs one function.
- **Rely on upstream's default math markup (`<span class="math math-inline">`).** Rejected: class selectors can be forged through heading class attributes, and Lens would no longer own its rendering contract.
- **TeX in a `data-math-*` attribute on an empty element.** Rejected: readers see nothing when KaTeX fails or JavaScript is off.
- **Drop `ENABLE_HEADING_ATTRIBUTES`.** Rejected: it would break `{#id}` and `{.class}`, which documents rely on. I4 keeps them.

## Consequences

- The 1,106-line pre-parser, its placeholders, the placeholder allowlist and the `&#10;`/`&#124;` attribute encodings are not carried forward.
- Math correctness becomes a property of the parser, which is tested upstream. Lens tests the event mapping and pins the accepted behaviors (D1–D3, D6, D7, D9) so that an upstream change shows up as a failing test.
- Body text no longer encodes `"` as `&quot;`. On `main`, no Lens assertion depends on it: the only `&quot;` assertion checks Lens's own `escape_html` output for Mermaid source.
- New parser behavior arrives only through a deliberate version bump (the `=` pin) or an option-set change, never implicitly.
- The binary grows by about the size of the KaTeX assets.
- Residual risks:
  - delimiter policy follows upstream;
  - authored `javascript:` link destinations remain a pre-existing, CSP-mitigated risk;
  - very large formulas are bounded only by `maxExpand` and `maxSize`, not by length.

## Verification

- Every C19 slice passes `cargo fmt --check`, `cargo clippy --locked --all-targets --all-features -- -D warnings` and `cargo test --locked`. Slices 4 and 5 also pass `npm run test:browser`.
- Each invariant I1–I4 has at least one Rust test whose name cites it in the C19 board.
