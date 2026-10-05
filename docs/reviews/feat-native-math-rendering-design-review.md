# Native Math Rendering (KaTeX) Design Review

Reviewed architectural deliverables for client-side mathematical formula rendering via native parser events:
- [`docs/decisions/adr-027-native-parser-math-pulldown-cmark-013.md`](../decisions/adr-027-native-parser-math-pulldown-cmark-013.md) (`ADR-027`)
- [`docs/features/markdown-viewing/math-rendering-spec.md`](../features/markdown-viewing/math-rendering-spec.md) (`Spec`)
- [`docs/iterations/c19-math-rendering-tasks.md`](../iterations/c19-math-rendering-tasks.md) (`Iteration C19`)
- [`docs/reviews/pb-6-native-parser-math-spike.md`](pb-6-native-parser-math-spike.md) (`PB-6 Spike Report`)
- [`docs/index.md`](../index.md) (`Index`)

**Reviewer Model:** Claude Opus 5.5 (Medium Thinking) via Pi with GitHub Copilot and Thinker Architecture Analysis.  
**Verdict:** **Design Approved (Ready for Implementation)**.

---

## 1. Architectural Critique & Invariants Proof

### 1.1 Single-Parser Stream Architecture

The original attempt (ADR-026) introduced a custom pre-parser (`src/markdown/math.rs`, 1,106 lines) that diverged from CommonMark on list-nested fences, blockquotes, container-relative indents, and autolinks across 7 failed Gate 2 rounds.

Under ADR-027, Lens eliminates the second parser completely. The Markdown parser (`pulldown-cmark 0.13.4` with `ENABLE_MATH`) is the sole component that discovers math. Lens operates as a single-pass event stream transformer:
```text
frontmatter split ─► Parser::new_ext(body, LENS_OPTIONS)        (I3)
                     │
                     ▼  one event loop in src/markdown.rs
   Html | InlineHtml ───────────────► Text                        (I1)
   Start(Heading{attrs}) ───────────► Start(Heading{attrs: []})   (I4)
   Start(Image) / End(Image) ───────► image_depth ±1, pass through
   InlineMath / DisplayMath
       image_depth > 0 ─────────────► pass through (upstream writes "$tex$" into alt)
       otherwise ───────────────────► Event::Html(Lens math span)
   fenced ```math ──────────────────► collect Text ─► Event::Html(math-block)
   plantuml / mermaid / links / tables: unchanged except Tag/TagEnd API
                     │
                     ▼
              html::push_html ─► cached fragment (no token)
                     │
                     ▼  page() at request time
              inject_capability: tag-aware tokenizer               (I2)
```

Lens constructs trusted markup only as `Event::Html`. It never constructs `Event::InlineHtml`.

### 1.2 Math Containers: TeX in Text Content

The design specifies:
```html
<span class="math-inline" data-math-inline>{escape_html(tex)}</span>
<span class="math-display" data-math-display>{escape_html(tex)}</span>
```
- **Graceful degradation:** If JavaScript is disabled or `/katex.js` fails to load, readers see raw TeX instead of an empty element.
- **Unified reading:** `app.js` reads `textContent` for spans and `math-block`, identical to Mermaid's model.
- **Display math in inline containers:** Because display math is emitted inside `<p>` or `<td>`, using `span` (with `display: block` in CSS) avoids illegal nested `<div>` elements that force early paragraph closure.
- **Unforgeable markers:** `data-math-*` is a boolean attribute; authors cannot inject `data-*` attributes due to Invariants I1 and I4.

### 1.3 Invariants Proof

- **I1: No parser raw HTML reaches the page.** `Event::Html` and `Event::InlineHtml` both map to `Event::Text`, which is safely escaped by `escape_html_body_text` (`<` becomes `&lt;`).
- **I2: Capability injection never modifies text content.**
  - *Lemma A:* In the fragment produced by `render()`, every `<` byte begins a genuine HTML tag.
  - *Lemma B:* Attribute values are double-quoted and cannot contain raw `"` or `>`.
  - *Lemma C:* The linear tokenizer matches only `href` on `<a>` or `src` on `<img>` starting with `/documents/` or `/diagrams/`. All text between tags is copied byte-for-byte.
- **I3: Pinned parser options.** `LENS_OPTIONS` pins exactly 7 flags, preventing inadvertent feature creep or syntax changes.
- **I4: Heading attribute blocks produce only `id` and `class`.** Heading `attrs` are stripped to empty `Vec::new()`, blocking forged `style`, `data-diagram`, `onclick`, or capability attributes.

---

## 2. Adversarial Doubt Analysis & Threat Matrix

| Attack / Hazard | Mitigating Control | Status |
| :--- | :--- | :--- |
| Block, inline, comment or CDATA HTML | I1 → `Text` | Closed |
| `# h {onclick=… style=… data-diagram href=/documents/x}` | I4 strips custom heading attributes | Closed |
| Forged `data-math-*` or `data-diagram` elements to hijack client JS | Needs raw HTML or heading attributes | Closed by I1 + I4 |
| `"` in text or code turning into a token-bearing attribute | I2 tag tokenizer + Lemma A | Closed |
| `<` hidden in title, alt, id, class or URL to fake a tag | `escape_html` / `escape_href` | Closed |
| TeX breaking out of the span (`</span><script>`) | `escape_html` on the formula text | Closed |
| `\href`, `\url`, `\includegraphics`, `\html*` | `trust: false` in KaTeX config | Closed |
| Macro recursion bombs, huge `\rule` or `\kern` | `maxExpand: 1000`, `maxSize: 500` | Bounded |
| `\gdef` redefining commands across formulas | Fresh options, no shared `macros` object | Closed |
| `{#katex}` / `{#mermaid}` DOM clobbering | `typeof engine.render` guard | Closed |
| CSS exfiltration through `\color` | KaTeX color validation; `img-src 'self' data:` | Bounded |
| Image alt text math disappearance | Track `image_depth`; pass math events untouched in alt | Closed |
| Token injection before parser upgrade | Slice 1 implements tag-aware injector on 0.9.3 | Closed |

---

## 3. Implementation Pitfall Checklist for Builders

Downstream builders must specifically avoid these common errors:
1. **Never map math inside images:** Math events inside `Tag::Image` must pass through untouched so upstream `raw_text` emits `$tex$` into the `alt="..."` attribute.
2. **Never emit spans as `Event::InlineHtml`:** Always emit Lens markup as `Event::Html`.
3. **Never use `<div>` for display math:** Always use `span.math-display` with `display: block` in CSS.
4. **Never run a blanket `cargo update`:** Use `cargo update -p pulldown-cmark --precise 0.13.4` only to preserve Rust 1.75 toolchain pins.
5. **Never enable `ENABLE_MATH` in Slice 2:** Keep Slice 2 behavior-preserving; defer math recognition to Slice 3.
6. **Never leave `<details>` hidden on `math-block`:** Start it collapsed without `hidden` so readers without JS can inspect the LaTeX source.
7. **Correct browser test command:** Run `npm run test:browser`.

---

## 4. Gate 1 Sign-Off Checklist

- [x] Dependency bump to `pulldown-cmark = "=0.13.4"` verified and accepted.
- [x] Delimiter policy D1–D8 accepted with author guidance documented.
- [x] Invariants I1–I4 formally specified and test-mapped.
- [x] Text-content span contract (R2, R11) specified for graceful degradation.
- [x] Vertical slice order verified (Slice 1 injector hardening before Slice 2 parser upgrade).
- [x] PB-6 spike report committed to `docs/reviews/pb-6-native-parser-math-spike.md`.
- [x] Every rule R1–R11 has explicit test coverage in the task board.
- [x] Explicit staging and builder non-goals enforced.

**Status: Gate 1 Design Approved.**
