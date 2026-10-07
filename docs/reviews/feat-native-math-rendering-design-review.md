# Native Math Rendering (KaTeX) Design Review

Reviewed architectural deliverables for client-side mathematical formula rendering via native parser events:
- [`docs/decisions/adr-027-native-parser-math-pulldown-cmark-013.md`](../decisions/adr-027-native-parser-math-pulldown-cmark-013.md) (`ADR-027`)
- [`docs/features/markdown-viewing/math-rendering-spec.md`](../features/markdown-viewing/math-rendering-spec.md) (`Spec`)
- [`docs/iterations/c19-math-rendering-tasks.md`](../iterations/c19-math-rendering-tasks.md) (`Iteration C19`)
- [`docs/reviews/pb-6-native-parser-math-spike.md`](pb-6-native-parser-math-spike.md) (`PB-6 Spike Report`)
- [`docs/index.md`](../index.md) (`Index`)

**Reviewers:**
- Reviewer 1: **GPT-6.1 Sol** (`--thinking xhigh`) via Pi with GitHub Copilot
- Reviewer 2: **Claude Sonnet 5.5** (`--thinking xhigh`) via Pi with GitHub Copilot
- Architect Analysis: Thinker (`doubt-driven-development`)

**Verdict:** **Reconciled & Design Approved (Ready for Implementation)**.

---

## 1. Executive Summary & Review Findings

Initial adversarial review by GPT-6.1 Sol and Claude Sonnet 5.5 confirmed that the fundamental architectural decision—replacing the 1,106-line pre-parser with `pulldown-cmark 0.13.4`'s native `ENABLE_MATH`, enforcing Invariants I1–I4, and structuring vertical slices with Slice 1 injector hardening—is sound and robust.

Both reviewers identified actionable specification and task board defects that have now been fully reconciled. A subsequent delta review confirmed:
- **Claude Sonnet 5.5**: **`VERIFIED (Design Approved)`** — all findings (G1-H1, G1-M1, G1-M2, G1-M3, F-H1, F-H2, F-M2, F-M3, F-M4) completely and accurately addressed.
- **GPT-6.1 Sol**: **`Approved on Architecture & Invariants`** with two minor documentation polish items resolved immediately:
  - ADR-027 §6.1 D3 table example formatted as fenced code block to eliminate markdown pipe collision.
  - Iteration C19 Slice 3 explicitly segmented into 3.1 Red-First tests (fail without `ENABLE_MATH`) and 3.2 Characterization / Preserved Behavior tests (pass before and after).

### Actionable Review Findings & Resolutions

1. **G1-H1 / F-M1: HTML Entity vs. URL Fragment Boundary in `inject_capability`**
   - *Finding*: If a document filename contains an apostrophe (e.g. `O'Reilly.md`), upstream encodes it in `href` as `&#x27;`. Splitting on `#` naively treats the entity's `#` as a fragment delimiter, corrupting the URL to `/documents/O&?token=T#x27;Reilly.md#intro`.
   - *Resolution*: Updated Invariant I2 and task board Slice 1 to specify entity-aware delimiter scanning in `push_capability_url` and added regression test `document_link_with_apostrophe_then_preserves_path_and_fragment`.

2. **G1-M1 / F-L3: Phrasing-Safe Math Error Fallbacks**
   - *Finding*: Inserting block-level elements (`<div>`, `<p>`, `<details>`) into math spans (`span.math-display` or `span.math-inline`) inside `<p>`, `<td>`, or headings violates HTML phrasing content rules.
   - *Resolution*: Updated R8 and task board Slice 4 so error fallbacks for spans emit strictly phrasing-safe elements (`<span class="math-error">Formula error: <code>{source}</code></span>` with CSS display styling), reserving `<details class="math-source">` disclosures exclusively for fenced `math-block` containers. Added post-render DOM structure tests.

3. **G1-M2 / F-M5: Core Browser Qualification Brought into Slice 4**
   - *Finding*: Slice 4 implemented client rendering (`app.js`) but deferred browser verification to Slice 5, violating the independently qualified slice contract.
   - *Resolution*: Re-scoped Slice 4 to include core Playwright tests in `tests/browser/lens.spec.mjs` (formula rendering, `{trust: false, maxSize, maxExpand}`, phrasing error fallbacks, missing KaTeX degradation, zero off-origin requests). Slice 5 focuses on comprehensive fixture verification and multi-feature integration.

4. **G1-M3: Separation of Degradation Contracts in R11**
   - *Finding*: R11 promised Mermaid "works" when JavaScript is disabled, but Mermaid requires client JavaScript to generate SVG.
   - *Resolution*: Split R11 into explicit modes: Mode A (KaTeX script unavailable, JS enabled -> raw math shown, Mermaid interactive SVG and live refresh active); Mode B (JavaScript disabled -> raw math and diagram source disclosures readable, PlantUML images load, dynamic features inactive); Mode C (`id="katex"` element without KaTeX engine -> returns cleanly).

5. **F-H1: Test-First Protocol & Missing Characterization Rows**
   - *Finding*: Characterization tests pass before and after the change; asserting they fail first would stall the Builder. Also, several edge cases (D3 table pipe, forged `<span data-math-inline>`, `math extra`, `id="katex"`) lacked explicit test rows.
   - *Resolution*: Formally clarified the test-first protocol in Builder Ground Rules (red-first for new features/guards; green-first for characterization tests; temporary mutation to verify guards on unchanged code). Added all missing test rows across Slices 1, 2, and 3.

6. **F-H2: Footnote Semantics Acceptance (D9)**
   - *Finding*: `ENABLE_FOOTNOTES` in 0.13.4 adopts GitHub-compatible footnote semantics (consecutive definitions separated, undefined references rendered literally).
   - *Resolution*: Formally accepted as behavior **D9** in ADR-027 and Spec R1, with test coverage added to Slice 2.

7. **F-M2 & F-M3: Generalized D7 Block Starters & D3 Pipe Guidance**
   - *Finding*: Any block starter line (e.g. `+ `, `- `, `> `, `1.`, `#`, `===`) terminates display math paragraphs; table pipe escaping behavior required clear guidance.
   - *Resolution*: Generalized D7 to cover all block starter lines with author guidance (`\begin{aligned}` or 4+ space indent), clarified D3 regarding unescaped `|` splitting vs `\|` stripping, recommending `\lvert x \rvert` or `\Vert`. Added pin tests for both.

8. **F-M4: Pinned KaTeX Asset Provenance & SHA-256**
   - *Finding*: KaTeX asset hashes were unpinned in documentation.
   - *Resolution*: Pinned verified SHA-256 hashes in ADR-027 §5:
     - `katex.min.js`: `e8d885505949f3a5f4abdd5dd0d53696bd1371ad26ffbf4f310dcd77c8cdae89`
     - `katex.min.css`: `05f52c1d80561bc3d1024881edd88c25e49352d4ee08493d2f912c27d2ef7a12` (with inlined WOFF2 fonts)

---

## 2. Invariants Verification Summary

- **I1: No parser raw HTML reaches the page.** Both `Event::Html` and `Event::InlineHtml` map to `Event::Text`. Upstream `escape_html_body_text` escapes `<` to `&lt;`. All Lens HTML templates escape interpolations. **Confirmed.**
- **I2: Capability injection never modifies text content and preserves URL semantics.** Tag-aware tokenizer matches start tags and matching attribute prefixes. `push_capability_url` is entity-safe. **Confirmed.**
- **I3: Pinned parser options.** `LENS_OPTIONS` pins exactly 7 flags using `.union()`. **Confirmed.**
- **I4: Heading attribute blocks produce only `id` and `class`.** Heading `attrs` are stripped to empty `Vec::new()`, blocking forged attributes. **Confirmed.**

---

## 3. Gate 1 Sign-Off Checklist

- [x] Dependency bump to `pulldown-cmark = "=0.13.4"` verified and accepted.
- [x] Delimiter policy D1–D9 accepted with author guidance documented.
- [x] Invariants I1–I4 formally specified and test-mapped.
- [x] Phrasing-safe error fallbacks (R8) specified for inline and display spans.
- [x] Entity-safe URL boundary handling specified in I2 and Slice 1 (G1-H1).
- [x] Vertical slice order verified (Slice 1 injector hardening before Slice 2 parser upgrade).
- [x] Slice 4 incorporates core client browser qualification (G1-M2, F-M5).
- [x] Degradation modes in R11 clearly distinguished and assigned tests (G1-M3).
- [x] Asset SHA-256 hashes pinned in ADR-027 §5 (F-M4).
- [x] Every rule R1–R11 has explicit test coverage in the task board (F-H1).
- [x] Explicit staging and builder non-goals enforced.

**Status: Definition of Ready Met — Design Approved for Builder Dispatch.**
