---
type: "Iteration Plan & Task Board"
title: "Iteration C19: Native-Parser Math Rendering (pulldown-cmark 0.13.4 + KaTeX 0.16.22)"
description: "Ordered, test-first vertical slices for replacing the math pre-parser with pulldown-cmark native math events, hardening the parser upgrade, and rendering formulas offline with bundled KaTeX."
id: "C19"
status: "ready"
tags: [iteration, tasks, planning, worker-handoff, math, katex, pulldown-cmark, security]
---

# Iteration C19: Native-Parser Math Rendering

## Overview

This board turns [ADR-027](../decisions/adr-027-native-parser-math-pulldown-cmark-013.md) and the [math rendering spec](../features/markdown-viewing/math-rendering-spec.md) (rules R1–R11) into five vertical slices. A vertical slice is a thin change that crosses every needed layer and leaves the product shippable. Each slice is one commit.

The order is chosen by risk, so that **no intermediate commit is less safe than `main`**:

1. Slice 1 makes capability injection tag-aware while the old parser is still in place (I2).
2. Slice 2 upgrades the parser together with its security controls (I1, I3, I4).
3. Slice 3 adds math recognition. Without KaTeX, formulas show as raw TeX.
4. Slice 4 adds KaTeX rendering.
5. Slice 5 adds end-to-end proof.

Salvage source (read-only): `/home/ccwu/.treehouse/lens-8a0594/1/feat-offline-math-rendering/`, called *salvage* below.

## Builder Ground Rules

- **Test first.** For every task, write the named test, run it, and see it fail for the expected reason. Then implement until it passes. Record each red/green pair in the report.
- **Test style.** Names are `<condition_or_action>_then_<observable_result>`, with `// Arrange`, `// Act`, `// Assert` and one behavior per test.
- **Gates for every slice:** `cargo fmt --check`, `cargo clippy --locked --all-targets --all-features -- -D warnings`, `cargo test --locked`. Slices 4 and 5 also run `npm run test:browser`.
- **Prohibited** (each one means stop and ask):
  - `Options::all()`;
  - a Lens scan of Markdown source for `$`;
  - constructing `Event::InlineHtml`;
  - `trust: true`;
  - a CDN URL;
  - `innerHTML` in math code;
  - any dependency change other than the one in Slice 2;
  - copying `src/markdown.rs`, `src/markdown/math.rs`, `Cargo.toml`, `Cargo.lock` or `docs/` from salvage.
- If `src/markdown.rs` passes about 500 non-test lines, report it. Do **not** split it in C19 (AGENTS.md: a file split is not combined with other changes).

## Staging Rules (every slice)

1. Stage only the slice's listed paths, one by one: `git add <path> <path> …`. Never use `git add .`, `git add -A`, `git add -u` or `git commit -a`.
2. Before committing, run `git diff --cached --name-only` and confirm it lists exactly the slice's files. Run `git status --short` and confirm nothing else is staged.
3. Never stage `docs/verification/`, `test-results/`, `playwright-report/`, `target/` or `node_modules/`. Verification logs belong in the Builder report. If the operator asks for them to be kept in the repository, they go into a separate commit under `docs/verification/c19/` that the operator authorizes.
4. Exactly one commit per slice, in Conventional Commits style. Don't push between slices. Amend only the most recent unpublished slice commit, and only to repair that slice.

## Slice 1: Tag-Aware Capability Injection (I2, R6)

**Objective:** Replace the string scan in `inject_capability` with the single-pass tag tokenizer from ADR-027 §4, on the current parser. Behavior for genuine links and diagrams is unchanged.

**Files:** `src/viewer/page.rs`

- [ ] **1.1** Add `CAPABILITY_ATTRIBUTES`, `scan`, `is_capability_attribute` and `push_capability_url`. Rewrite `inject_capability` as specified: it changes only quoted attribute values of matching (element, attribute, prefix) entries.
- [ ] **1.2** Tests in `page.rs` (the existing `inject_capability_then_attaches_token_to_document_and_diagram_urls` must keep passing unchanged):
  - `document_link_with_fragment_then_inserts_token_before_fragment`
  - `document_link_with_existing_query_then_appends_token_with_ampersand`
  - `diagram_image_with_trailing_boolean_attribute_then_appends_token`
  - `body_text_resembling_document_attribute_then_remains_byte_identical` (input `<p>href="/documents/a.md"</p>`; this fails against the old scanner)
  - `code_text_resembling_anchor_markup_then_remains_byte_identical` (input `<pre><code>&lt;a href="/documents/a.md"&gt;</code></pre>`; this fails against the old scanner)
  - `attribute_name_ending_in_href_then_is_not_rewritten` (`data-href`)
  - `capability_attribute_on_other_element_then_is_not_rewritten` (`<link href=…>`, `<h1 src=…>`)
  - `truncated_tag_then_copies_remainder_unchanged`
  - `non_ascii_text_around_tags_then_is_preserved` (UTF-8 boundary safety)
- **Commit:** `fix: inject session capability only into tag attributes`
- **Stage:** `git add src/viewer/page.rs`

## Slice 2: Parser Upgrade with Security Controls (H1, H3, H5, H6 → I1, I3, I4; R5, R7, R10)

**Objective:** Move to `pulldown-cmark` 0.13.4 with no behavior change beyond `&quot;` → `"` in body text. **`ENABLE_MATH` is not turned on in this slice.**

**Files:** `Cargo.toml`, `Cargo.lock`, `src/markdown.rs`, `src/viewer/page.rs` (end-to-end tests only)

- [ ] **2.1** In `Cargo.toml`: `pulldown-cmark = { version = "=0.13.4", default-features = false, features = ["html"] }`. Update the lockfile with `cargo update -p pulldown-cmark --precise 0.13.4` **only**. A blanket `cargo update` would break the Rust 1.75 pins (for example `idna_adapter`). Confirm the new `bitflags` 2.x, `unicase` and `pulldown-cmark-escape` resolve and build on the repository toolchain.
- [ ] **2.2** Migrate the API:
  - `Tag::Link { link_type, dest_url, title, id }`, `End(TagEnd::Link)`, `End(TagEnd::CodeBlock)`, `Start(Tag::Table(a))` and `End(TagEnd::Table)`;
  - the Email-autolink branch keeps `dest_url` unchanged;
  - move the source-link indicator push onto `TagEnd::Link`.
- [ ] **2.3** Add `const LENS_OPTIONS: Options` = the six current flags (`TABLES | FOOTNOTES | STRIKETHROUGH | TASKLISTS | SMART_PUNCTUATION | HEADING_ATTRIBUTES`) and use it in `Parser::new_ext` (I3).
- [ ] **2.4** I1: `Event::Html(v) | Event::InlineHtml(v) => events.push(Event::Text(v))`.
- [ ] **2.5** I4: `Event::Start(Tag::Heading { level, id, classes, attrs: _ })` → push the same heading with `attrs: Vec::new()`.
- [ ] **2.6** Tests in `markdown.rs`:
  - `inline_raw_html_then_is_escaped` (R5, I1)
  - `block_raw_html_with_event_handler_then_is_escaped` (R5)
  - `heading_attribute_block_with_custom_attributes_then_keeps_only_id_and_classes` (R10, I4)
  - `subscript_and_superscript_markers_then_render_literally` (R7)
  - `wikilink_syntax_then_renders_literally` (R7)
  - `definition_list_syntax_then_renders_no_definition_list` (R7)
  - `gfm_alert_marker_then_renders_plain_blockquote` (R7)
  - `defined_footnote_then_renders_reference_and_definition` (R7 parity)
  - `strikethrough_tasklist_and_smart_quotes_then_render_as_before` (R7 parity)
  - all existing tests pass; list every edited assertion in the report.
- [ ] **2.7** End-to-end tests in `page.rs`, using `crate::markdown::render` and then `page`:
  - `markdown_code_span_with_document_anchor_text_then_page_adds_no_token_to_code` (R6)
  - `markdown_raw_anchor_html_then_page_adds_no_token_to_escaped_text` (R6)
- **Commit:** `build: upgrade pulldown-cmark to 0.13.4 with pinned options and escaped inline HTML`
- **Stage:** `git add Cargo.toml Cargo.lock src/markdown.rs src/viewer/page.rs`

## Slice 3: Native Math Event Mapping (R1–R4)

**Objective:** Turn on `ENABLE_MATH` and map math events to Lens spans and blocks. Formulas show as raw TeX until Slice 4.

**Files:** `src/markdown.rs`

- [ ] **3.1** Add `ENABLE_MATH` to `LENS_OPTIONS`.
- [ ] **3.2** Track `image_depth`:
  - `event @ Event::Start(Tag::Image { .. })` adds 1;
  - `End(TagEnd::Image)` subtracts 1 (saturating);
  - both are pushed unchanged.
- [ ] **3.3** `InlineMath(tex) if image_depth == 0` → `Event::Html(math_inline_html(&tex))`. `DisplayMath(tex) if image_depth == 0` → `Event::Html(math_display_html(&tex))`. When `image_depth > 0`, math events are pushed unchanged. These guarded arms must come before the catch-all arm.
- [ ] **3.4** Intercept ```` ```math ```` like `mermaid` (a third `Option<String>` collector, same pattern) and emit `math_block_placeholder(&source)` exactly as in R4.
- [ ] **3.5** Tests in `markdown.rs` (each cites its rule):
  - R1:
    - `inline_math_with_underscores_then_emits_span_with_verbatim_tex`
    - `inline_math_with_asterisks_then_emits_no_emphasis`
    - `display_math_spanning_lines_then_keeps_line_breaks_in_span_text`
    - `spaced_dollar_delimiters_then_render_literal_text`
    - `currency_amounts_then_render_literal_text`
    - `escaped_dollar_signs_then_render_literal_dollars`
    - `code_span_and_code_block_with_dollar_math_then_emit_no_math_span`
    - `nested_list_and_blockquote_fences_with_dollar_math_then_emit_no_math_span`
    - `link_destination_and_autolink_with_dollars_then_preserve_urls`
    - `html_comment_with_dollar_math_then_renders_escaped_comment_text`
    - `math_in_heading_link_footnote_and_table_cell_then_emits_spans_in_place`
    - `diagram_fences_with_dollar_math_then_preserve_source_without_math_span`
    - `currency_range_without_escapes_then_renders_math_per_parser_rule` (D1)
    - `digit_after_closing_dollar_then_renders_math_followed_by_digit` (D2)
    - `shell_expression_in_prose_then_renders_math_per_parser_rule` (D6)
  - R2:
    - `math_tex_with_html_characters_then_escapes_span_text`
    - `display_math_in_table_cell_then_emits_display_span_inside_cell`
    - `display_math_paragraph_then_contains_no_block_element_inside_paragraph`
  - R3:
    - `image_alt_with_inline_math_then_contains_literal_dollar_source`
    - `image_alt_with_display_math_then_contains_literal_double_dollar_source`
  - R4:
    - `fenced_math_block_then_emits_math_block_container_with_escaped_source`
    - `fenced_math_block_with_mixed_case_info_then_emits_math_block_container`
    - `fenced_math_block_inside_list_item_and_blockquote_then_emits_container_inside`
    - `fenced_mathematica_block_then_remains_code_block`
    - `fenced_math_source_with_script_tag_then_escapes_source`
- **Commit:** `feat: map native parser math events to Lens math containers`
- **Stage:** `git add src/markdown.rs`

## Slice 4: KaTeX Assets, Routes, CSP and Client Rendering (R8, R9, R11)

**Objective:** Bring in the salvaged KaTeX assets and rendering, adapted to the text-content contract.

**Files:** `src/viewer/assets/katex.min.js`, `src/viewer/assets/katex.min.css`, `src/viewer/assets/app.js`, `src/viewer/assets/app.css`, `src/viewer/page.rs`, `src/viewer/routes.rs`

- [ ] **4.1** Copy both KaTeX assets byte for byte from salvage. Record SHA-256 of each. Check that `katex.min.js` matches npm `katex@0.16.22/dist/katex.min.js`, and that every `data:font/woff2` in the CSS decodes to the matching `dist/fonts/*.woff2`. Put the results in the report.
- [ ] **4.2** In `page.rs`:
  - add the `KATEX_SCRIPT` and `KATEX_STYLESHEET` constants with provenance comments, plus `katex_script()` and `katex_stylesheet()`;
  - add `<link rel="stylesheet" href="/katex.css?token={}">` before `app.css`, and `<script src="/katex.js?token={}"></script>` before `mermaid.js`;
  - add `font-src 'self' data:` to `content_security_policy()`.
- [ ] **4.3** In `routes.rs`: add the routes `/katex.css` and `/katex.js` with handlers, add both paths to the existing unauthenticated-request test list, and update the exact-CSP test string.
- [ ] **4.4** In `app.js`: port `createInlineMathError`, `createDisplayMathError` and `renderMath` from salvage, then adapt:
  - read `el.textContent` before rendering;
  - guard with `const engine = window.katex; if (!engine || typeof engine.render !== 'function') return;`;
  - build a fresh options object per call, with no `macros`;
  - use `el.replaceChildren(fallback)`;
  - for blocks, read `.math-source code` text, and on failure hide `.math-target`, un-hide `.math-error` and set `details.open = true`;
  - call `renderMath()` inside `try { … } catch {}` after the PlantUML loop and before the Mermaid block.
- [ ] **4.5** In `app.css`: port salvage lines 45–50, and add a raw-TeX style for spans that haven't been rendered yet.
- [ ] **4.6** Tests:
  - `routes.rs`:
    - `authenticated_katex_script_request_then_returns_javascript_content`
    - `authenticated_katex_stylesheet_request_then_returns_css_content`
    - `unauthenticated_katex_asset_request_then_returns_unauthorized`
    - updated `document_request_then_sets_restrictive_content_security_policy`
  - `page.rs`:
    - `document_page_then_loads_katex_assets_before_app_assets`
    - `katex_stylesheet_then_references_only_inlined_data_fonts` (no `url(` except `data:`)
- **Commit:** `feat: render math offline with bundled KaTeX 0.16.22`
- **Stage:** `git add src/viewer/assets/katex.min.js src/viewer/assets/katex.min.css src/viewer/assets/app.js src/viewer/assets/app.css src/viewer/page.rs src/viewer/routes.rs`

## Slice 5: Fixture, Browser Verification and Full Gates (R1–R11 end to end)

**Files:** `tests/fixtures/math-specification.md`, `tests/browser/lens.spec.mjs`; also `docs/release-notes.md`, but only if it has an unreleased section

- [ ] **5.1** Fixture: port it from salvage, then:
  - add the sentence `A price range is written as \$5-\$10.` under "Financial & Environment Constraints";
  - add the table row `| Magnitude | $\lvert x \rvert$ | $\le 1$ |`;
  - add `` `$(CC)$(FLAGS)` `` to the shell sentence.

  Expected `.katex` count: **13**. Confirm it by running the test and record it.
- [ ] **5.2** Port the salvage math browser tests, updating counts to the fixture. **Replace** `table_with_pipe_in_math_formula_then_renders_without_breaking_columns` with `table_cell_with_lvert_absolute_value_then_renders_math_in_single_cell` (D3).
- [ ] **5.3** Add browser tests:
  - `katex_script_unavailable_then_shows_raw_tex_and_renders_rest_of_document` (R11; abort `**/katex.js*` with `page.route`)
  - `currency_range_with_escapes_then_renders_literal_dollars` (D1 guidance)
  - `image_alt_with_math_then_exposes_literal_alt_text` (R3; use an image `src` of `data:`, so no request is made)
- [ ] **5.4** Run all gates and `npm run test:browser`, and attach the logs to the Builder report (not to the commit).
- **Commit:** `test: verify native math rendering end to end in the browser`
- **Stage:** `git add tests/fixtures/math-specification.md tests/browser/lens.spec.mjs`, plus `docs/release-notes.md` if it was edited

## Builder Report Template

For each slice: commit SHA; staged file list; red/green evidence per named test; edited pre-existing assertions with reasons; gate logs. Slice 4 also records the asset SHA-256 values and the font check. Slice 5 also records the confirmed `.katex` count.

## Acceptance Checklist

- [ ] I1–I4 each have a passing Rust test cited by ID.
- [ ] R1–R11 each have at least one passing test.
- [ ] No `Options::all()`, no `Event::InlineHtml(` construction, and no `$` scanner in `src/` (grep evidence attached).
- [ ] The CSP string matches R9 exactly, and zero off-origin requests are observed.
- [ ] Five commits, one per slice, each passing its gates on its own.
- [ ] PlantUML, Mermaid, links, source links and frontmatter: existing tests pass, with only the allowed assertion edits.
