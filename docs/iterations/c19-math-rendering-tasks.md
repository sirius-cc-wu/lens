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

1. Slice 1 makes capability injection tag-aware and entity-safe while the old parser is still in place (I2, R6).
2. Slice 2 upgrades the parser together with its security controls (I1, I3, I4; R5, R7, R10, D9).
3. Slice 3 adds math recognition (R1–R4, D1–D3, D6, D7). Without KaTeX, formulas show as raw TeX.
4. Slice 4 adds bundled KaTeX rendering and qualifies core client behavior in the browser (R8, R9, R11).
5. Slice 5 adds fixture verification, full integration, edge guidance, and degradation testing.

Salvage source (read-only): `/home/ccwu/.treehouse/lens-8a0594/1/feat-offline-math-rendering/`, called *salvage* below.

## Builder Ground Rules

- **Test-first and Characterization Protocol:**
  - **Red-first tests** (new features, tightened security controls, bug fixes): write the named test, run it, observe expected failure (RED), implement until it passes (GREEN).
  - **Characterization tests** (regression bounds, preserved baseline behaviors, negative syntax assertions): run the test against pre-change code to demonstrate established behavior (GREEN), and confirm it continues to pass post-change (GREEN). For security guards on unchanged code (e.g. `raw_html_then_is_escaped`), introduce a temporary mutation (e.g. raw pass-through) to verify failure (RED), then revert it (GREEN).
- **Test style:** Names are `<condition_or_action>_then_<observable_result>`, with `// Arrange`, `// Act`, `// Assert` and one behavior per test.
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

---

## Slice 1: Tag-Aware & Entity-Safe Capability Injection (I2, R6)

**Objective:** Replace the string scan in `inject_capability` with the single-pass tag tokenizer from ADR-027 §4 on the current parser, correctly parsing URL boundaries in the presence of HTML character references (G1-H1).

**Files:** `src/viewer/page.rs`

- [x] **1.1** Add `CAPABILITY_ATTRIBUTES`, `scan`, `is_capability_attribute` and `push_capability_url`.
  - Rewrite `inject_capability`: tokenizes start tags, rewriting only matching attribute values (`href` on `a` starting with `/documents/`, `src` on `img` starting with `/diagrams/`).
  - Inside `push_capability_url`: locate query `?` and fragment `#` delimiters while ignoring HTML character references (e.g. `&#x27;`). Insert `token=` before fragment or query cleanly without corrupting the path.
- [x] **1.2** Tests in `page.rs`:
  - *Red-first tests* (must fail on the old scanner before 1.1):
    - `body_text_resembling_document_attribute_then_remains_byte_identical` (input `<p>href="/documents/a.md"</p>`)
    - `code_text_resembling_anchor_markup_then_remains_byte_identical` (input `<pre><code>&lt;a href="/documents/a.md"&gt;</code></pre>`)
    - `attribute_name_ending_in_href_then_is_not_rewritten` (`data-href`)
    - `capability_attribute_on_other_element_then_is_not_rewritten` (`<link href=…>`, `<h1 src=…>`)
    - `document_link_with_apostrophe_then_preserves_path_and_fragment` (input `<a href="/documents/O&#x27;Reilly.md#intro">`)
  - *Characterization tests* (pass before and after):
    - `document_link_with_fragment_then_inserts_token_before_fragment`
    - `document_link_with_existing_query_then_appends_token_with_ampersand`
    - `document_link_with_query_and_fragment_then_preserves_query_and_fragment`
    - `diagram_image_with_trailing_boolean_attribute_then_appends_token`
    - `truncated_tag_then_copies_remainder_unchanged`
    - `non_ascii_text_around_tags_then_is_preserved` (UTF-8 boundary safety)
- **Commit:** `fix: inject session capability only into tag attributes with entity-safe URL boundaries` (`3f4d982`, remediated in `7ca7c85`)
- **Stage:** `git add src/viewer/page.rs`

---

## Slice 2: Parser Upgrade with Security Controls (H1, H3, H5, H6 → I1, I3, I4; R5, R7, R10, D9)

**Objective:** Move to `pulldown-cmark` 0.13.4 with pinned options, escaping all raw HTML events, stripping custom heading attributes, and accepting GitHub-style footnotes (D9). **`ENABLE_MATH` is not turned on in this slice.**

**Files:** `Cargo.toml`, `Cargo.lock`, `src/markdown.rs`, `src/viewer/page.rs` (end-to-end tests only)

- [ ] **2.1** In `Cargo.toml`: `pulldown-cmark = { version = "=0.13.4", default-features = false, features = ["html"] }`. Update lockfile with `cargo update -p pulldown-cmark --precise 0.13.4` **only**.
- [ ] **2.2** Migrate API to compile:
  - `Tag::Link { link_type, dest_url, title, id }`, `End(TagEnd::Link)`, `End(TagEnd::CodeBlock)`, `Start(Tag::Table(a))` and `End(TagEnd::Table)`.
  - Retain `Options::all()` temporarily to observe red test failures for I1, I3, and I4.
- [ ] **2.3** Write *red-first tests* in `src/markdown.rs` and observe failures:
  - `inline_raw_html_then_is_escaped` (fails under 0.13 before I1)
  - `block_raw_html_with_event_handler_then_is_escaped` (fails under 0.13 before I1)
  - `raw_script_tag_in_markdown_then_is_escaped` (R5: fails before I1)
  - `forged_math_span_in_raw_markdown_then_is_escaped` (R5: `<span data-math-inline>x</span>` fails before I1)
  - `heading_attribute_block_with_custom_attributes_then_keeps_only_id_and_classes` (fails under 0.13 before I4)
  - `subscript_and_superscript_markers_then_render_literally` (fails under `Options::all()` before I3)
  - `wikilink_syntax_then_renders_literally` (fails before I3)
  - `definition_list_syntax_then_renders_no_definition_list` (fails before I3)
  - `gfm_alert_marker_then_renders_plain_blockquote` (fails before I3)
  - `metadata_blocks_plus_and_minus_then_render_literally` (fails before I3)
- [ ] **2.4** Implement controls:
  - Define `const LENS_OPTIONS: Options` using `.union()` to pin the 6 baseline flags (I3).
  - I1: `Event::Html(v) | Event::InlineHtml(v) => events.push(Event::Text(v))`.
  - I4: `Event::Start(Tag::Heading { level, id, classes, attrs: _ })` pushes `attrs: Vec::new()`.
- [ ] **2.5** New accepted behavior (D9) and characterization tests:
  - D9 Footnote behavior tests:
    - `defined_footnote_then_renders_reference_and_definition`
    - `undefined_footnote_reference_then_renders_literal_text` (D9)
    - `consecutive_footnote_definitions_then_render_as_separate_items` (D9)
    - `indented_footnote_continuation_then_stays_inside_footnote` (D9)
  - Characterization / positive tests:
    - `heading_attribute_block_with_valid_id_and_classes_then_preserves_them` (R10)
    - `html_comment_with_math_then_renders_escaped_comment_text` (R5)
    - `strikethrough_tasklist_and_smart_quotes_then_render_as_before`
  - End-to-end tests in `page.rs`:
    - `markdown_code_span_with_document_anchor_text_then_page_adds_no_token_to_code` (R6)
    - `markdown_raw_anchor_html_then_page_adds_no_token_to_escaped_text` (R6)
- **Commit:** `build: upgrade pulldown-cmark to 0.13.4 with pinned options and escaped inline HTML`
- **Stage:** `git add Cargo.toml Cargo.lock src/markdown.rs src/viewer/page.rs`

---

## Slice 3: Native Math Event Mapping (R1–R4, D1–D3, D6, D7)

**Objective:** Turn on `ENABLE_MATH` and map math events to Lens spans and blocks. Formulas show as raw TeX until Slice 4.

**Files:** `src/markdown.rs`

- [ ] **3.1** Write *red-first tests* in `src/markdown.rs` (these test new math mapping and fail while `ENABLE_MATH` is disabled):
  - R1 & Accepted Behaviors:
    - `inline_math_with_underscores_then_emits_span_with_verbatim_tex`
    - `inline_math_with_asterisks_then_emits_no_emphasis`
    - `display_math_spanning_lines_then_keeps_line_breaks_in_span_text`
    - `math_in_heading_link_footnote_and_table_cell_then_emits_spans_in_place`
    - `math_in_blockquote_and_list_item_then_emits_spans_in_place` (R1)
    - `currency_range_without_escapes_then_renders_math_per_parser_rule` (D1)
    - `digit_after_closing_dollar_then_renders_math_followed_by_digit` (D2)
    - `shell_expression_in_prose_then_renders_math_per_parser_rule` (D6)
  - R2:
    - `math_tex_with_html_characters_then_escapes_span_text`
    - `display_math_in_table_cell_then_emits_display_span_inside_cell`
    - `display_math_paragraph_then_contains_no_block_element_inside_paragraph`
  - R4:
    - `fenced_math_block_then_emits_math_block_container_with_escaped_source`
    - `fenced_math_block_with_mixed_case_info_then_emits_math_block_container`
    - `fenced_math_block_inside_list_item_and_blockquote_then_emits_container_inside`
    - `fenced_math_source_with_script_tag_then_escapes_source`
- [ ] **3.2** Characterization / preserved behavior tests (demonstrate established behavior passes before and after):
  - `spaced_dollar_delimiters_then_render_literal_text`
  - `currency_amounts_then_render_literal_text`
  - `escaped_dollar_signs_then_render_literal_dollars`
  - `code_span_and_code_block_with_dollar_math_then_emit_no_math_span`
  - `nested_list_and_blockquote_fences_with_dollar_math_then_emit_no_math_span`
  - `link_destination_and_autolink_with_dollars_then_preserve_urls`
  - `diagram_fences_with_dollar_math_then_preserve_source_without_math_span`
  - `table_with_unescaped_pipe_in_math_then_splits_cell_per_parser_rule` (D3 pin test)
  - `table_with_escaped_pipe_in_math_then_preserves_cell_and_strips_escape` (D3 escaped pipe pin test)
  - `display_math_with_block_starter_then_terminates_paragraph` (D7 pin test)
  - `image_alt_with_inline_math_then_contains_literal_dollar_source` (R3)
  - `image_alt_with_display_math_then_contains_literal_double_dollar_source` (R3)
  - `image_alt_with_html_and_math_then_escapes_without_math_span` (R3 row 3)
  - `fenced_math_extra_block_then_remains_code_block` (R4)
- [ ] **3.3** Implement mapping:
  - Add `ENABLE_MATH` to `LENS_OPTIONS`.
  - Track `image_depth` on `Tag::Image` / `TagEnd::Image`.
  - `InlineMath(tex) if image_depth == 0` -> `<span class="math-inline" data-math-inline>{escape_html(tex)}</span>`.
  - `DisplayMath(tex) if image_depth == 0` -> `<span class="math-display" data-math-display>{escape_html(tex)}</span>`.
  - Intercept ```` ```math ```` (exact trimmed case-insensitive match) emitting `math_block_placeholder` with unhidden `<details class="math-source">`.
- **Commit:** `feat: map native parser math events to Lens math containers`
- **Stage:** `git add src/markdown.rs`

---

## Slice 4: Bundled KaTeX Assets, Routes, CSP, Client Rendering & Browser Qualification (R8, R9, R11)

**Objective:** Vendor KaTeX 0.16.22 assets, implement client-side rendering with phrasing-safe error fallbacks, and verify core client behavior in the browser.

**Files:** `src/viewer/assets/katex.min.js`, `src/viewer/assets/katex.min.css`, `src/viewer/assets/app.js`, `src/viewer/assets/app.css`, `src/viewer/page.rs`, `src/viewer/routes.rs`, `tests/browser/lens.spec.mjs`

- [ ] **4.1** Vendor assets and verify SHA-256 against ADR-027:
  - `katex.min.js`: `e8d885505949f3a5f4abdd5dd0d53696bd1371ad26ffbf4f310dcd77c8cdae89`
  - `katex.min.css`: `05f52c1d80561bc3d1024881edd88c25e49352d4ee08493d2f912c27d2ef7a12`
  - Confirm CSS references only `data:font/woff2;base64` URIs.
- [ ] **4.2** In `page.rs` and `routes.rs`:
  - Expose `/katex.js` and `/katex.css` with token auth.
  - Update CSP: `default-src 'self'; base-uri 'none'; font-src 'self' data:; img-src 'self' data:; object-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'`. (Authorized pre-existing assertion update in `routes.rs:424`).
  - Inject `<link rel="stylesheet" href="/katex.css?token={}">` and `<script src="/katex.js?token={}"></script>`.
- [ ] **4.3** In `app.js` and `app.css`:
  - Implement `renderMath()` reading `textContent` before render.
  - Options per formula: `{displayMode, throwOnError: false, trust: false, maxSize: 500, maxExpand: 1000, strict: "warn"}`, no shared `macros`.
  - Phrasing-safe error fallbacks (G1-M1): for spans, replace children via `replaceChildren` with `<span class="math-error" title="Formula rendering failed">Formula error: <code>{source}</code></span>`. For blocks, hide `.math-target`, un-hide `.math-error`, and set `.math-source` `open = true`.
  - Guard with `const engine = window.katex; if (!engine || typeof engine.render !== 'function') return;` (R11 Mode C).
  - Add styles for math spans and phrasing error notices in `app.css`.
- [ ] **4.4** Server & Route tests:
  - `authenticated_katex_script_request_then_returns_javascript_content`
  - `authenticated_katex_stylesheet_request_then_returns_css_content`
  - `unauthenticated_katex_asset_request_then_returns_unauthorized`
  - `document_page_then_loads_katex_assets_before_app_assets`
  - `katex_stylesheet_then_references_only_inlined_data_fonts`
  - `document_request_then_sets_restrictive_content_security_policy` (asserts exact updated CSP with `font-src 'self' data:`)
- [ ] **4.5** Core browser qualification tests in `tests/browser/lens.spec.mjs` (G1-M2, F-M5):
  - `inline_and_display_math_rendering_then_produces_katex_elements`
  - `untrusted_href_math_formula_then_renders_no_anchor_element`
  - `math_dimension_limit_exceeded_then_clamps_rule_size_safely`
  - `math_macro_expansion_limit_exceeded_then_renders_error_fallback_without_hanging`
  - `malformed_math_span_in_paragraph_then_renders_phrasing_safe_fallback_without_block_elements` (G1-M1)
  - `katex_script_unavailable_then_shows_raw_tex_and_renders_rest_of_document` (R11 Mode A)
  - `page_with_katex_element_id_when_engine_missing_then_does_not_throw` (R11 Mode C)
  - `math_rendering_then_makes_zero_external_network_requests`
- **Commit:** `feat: render math offline with bundled KaTeX 0.16.22 and qualify client safety`
- **Stage:** `git add src/viewer/assets/katex.min.js src/viewer/assets/katex.min.css src/viewer/assets/app.js src/viewer/assets/app.css src/viewer/page.rs src/viewer/routes.rs tests/browser/lens.spec.mjs`

---

## Slice 5: Full Fixture, Complex Integration, Edge Guidance and Final Verification

**Objective:** Verify the full engineering fixture, multi-diagram coexistence, author edge guidance, and offline degradation modes.

**Files:** `tests/fixtures/math-specification.md`, `tests/browser/lens.spec.mjs`, `docs/release-notes.md` (if pending)

- [ ] **5.1** Port fixture `tests/fixtures/math-specification.md`:
  - Include 13 `.katex` formula occurrences (inline, display aligned, fenced code block, table formulas).
  - Add author guidance: `A price range is written as \$5-\$10.` (D1)
  - Add table row: `| Magnitude | $\lvert x \rvert$ | $\le 1$ |` (D3)
  - Add shell expressions in code span: `` `$(CC)$(FLAGS)` `` (D6)
- [ ] **5.2** Browser tests in `tests/browser/lens.spec.mjs`:
  - `math_specification_fixture_then_renders_13_katex_formulas`
  - `table_cell_with_lvert_absolute_value_then_renders_math_in_single_cell` (D3 guidance)
  - `currency_range_with_escapes_then_renders_literal_dollars` (D1 guidance)
  - `mixed_document_with_mermaid_and_math_then_renders_both_cleanly`
  - `live_document_refresh_with_math_then_updates_formulas_automatically`
  - `image_alt_with_math_then_exposes_literal_alt_text` (R3)
  - `javascript_disabled_then_preserves_readable_math_and_diagram_sources` (R11 Mode B; G1-M3)
- [ ] **5.3** Run all gates:
  - `cargo fmt --check`
  - `cargo clippy --locked --all-targets --all-features -- -D warnings`
  - `cargo test --locked`
  - `npm run test:browser`
- **Commit:** `test: verify native math rendering end to end across fixtures and degradation modes`
- **Stage:** `git add tests/fixtures/math-specification.md tests/browser/lens.spec.mjs docs/release-notes.md`

---

## Builder Report Template

For each slice: commit SHA; staged file list; red/green evidence per named test; edited pre-existing assertions with reasons; gate logs. Slice 4 also records the asset SHA-256 values and the font check. Slice 5 also records the confirmed `.katex` count.

## Acceptance Checklist

- [ ] I1–I4 each have a passing Rust test cited by ID.
- [ ] R1–R11 each have at least one passing test.
- [ ] No `Options::all()`, no `Event::InlineHtml(` construction, and no `$` scanner in `src/` (grep evidence attached).
- [ ] The CSP string matches R9 exactly, and zero off-origin requests are observed.
- [ ] Five commits, one per slice, each passing its gates on its own.
- [ ] PlantUML, Mermaid, links, source links and frontmatter: existing tests pass, with only the allowed assertion edits.
