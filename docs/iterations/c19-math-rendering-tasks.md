---
type: "Iteration Plan & Task Board"
title: "Iteration C19: Mathematical Formula Rendering (KaTeX) Implementation"
description: "Structured work packages, test specifications, and verification checklists for downstream Workers implementing offline KaTeX formula rendering in Lens."
id: "C19"
status: "ready"
tags: [iteration, tasks, planning, worker-handoff, math, katex]
---

# Iteration C19: Mathematical Formula Rendering (KaTeX) Implementation

## Overview

This task board breaks down the architectural decisions of [ADR-026](../decisions/adr-026-mathematical-formula-rendering.md) and the requirements of [Math Rendering Specification](../features/markdown-viewing/math-rendering-spec.md) into 4 discrete, test-driven vertical slices for execution by downstream Workers.

---

## Work Packages & Execution Slices

### Slice 1: Markdown Delimiter Extraction & Syntax Protection (`src/markdown.rs`)

**Objective:** Enhance `src/markdown.rs` to recognize `$formula$`, `$$formula$$`, and ````math fenced code blocks, masking code blocks and spans to protect LaTeX expressions from CommonMark parser mangling (underscores, asterisks, backslashes).

- [ ] **Task 1.1:** Intercept fenced ````math code blocks in `src/markdown.rs`:
  - Detect `Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(language)))` where `language.trim().eq_ignore_ascii_case("math")`.
  - Capture raw body and emit a display math container `<div class="math-block" data-math-block>...</div>`.
- [ ] **Task 1.2:** Implement context-aware inline (`$`) and display (`$$`) delimiter extractor:
  - Mask code spans (`` `...` ``), code blocks (``` ````), HTML comments, and link URLs with sentinels before math scanning.
  - Extract math spans from remaining text, enforcing opening/closing whitespace boundaries for single `$`.
  - Preserve escaped `\$` and currency patterns (e.g. `$10 and $20`).
  - Emit `<span class="math-inline" data-math-inline="{escaped_tex}"></span>` and `<div class="math-display" data-math-display="{escaped_tex}"></div>`.
  - Restore masked code spans and blocks.
- [ ] **Task 1.3:** Write 3A unit tests in `src/markdown.rs`:
  - `markdown_with_inline_math_then_preserves_latex_and_emits_inline_placeholder`
  - `markdown_with_display_math_then_emits_display_placeholder`
  - `markdown_with_fenced_math_block_then_emits_block_placeholder`
  - `math_with_underscores_then_does_not_convert_to_italics`
  - `code_span_with_dollar_variable_then_retains_code_span_without_math_placeholder`
  - `currency_text_with_dollar_signs_then_retains_literal_text`
  - `escaped_dollar_signs_then_retains_escaped_literals`
  - `markdown_table_with_math_cells_then_renders_math_placeholders_inside_cells`

---

### Slice 2: KaTeX Asset Vendoring, CSP & Loopback Routes (`src/viewer/`)

**Objective:** Bundle offline KaTeX assets into the binary, update CSP, and expose authenticated loopback routes.

- [ ] **Task 2.1:** Vendor standalone KaTeX 0.16.11 assets under `src/viewer/assets/`:
  - `katex.min.js` (minified KaTeX library).
  - `katex.min.css` (KaTeX stylesheet with self-contained, base64-inlined WOFF2 fonts).
- [ ] **Task 2.2:** Embed assets in `src/viewer/page.rs`:
  - `const KATEX_SCRIPT: &str = include_str!("assets/katex.min.js");`
  - `const KATEX_STYLESHEET: &str = include_str!("assets/katex.min.css");`
- [ ] **Task 2.3:** Update CSP and Axum routes in `src/viewer/routes.rs`:
  - Add `font-src 'self' data:` to `Content-Security-Policy` header.
  - `GET /katex.js?token={session_token}` -> serves JavaScript with `application/javascript`.
  - `GET /katex.css?token={session_token}` -> serves CSS with `text/css`.
  - Require valid session token; reject invalid or missing tokens with 401 Unauthorized.
- [ ] **Task 2.4:** Inject stylesheet and script tags in `src/viewer/page.rs`:
  - `<link rel="stylesheet" href="/katex.css?token={}">` in `<head>`.
  - `<script src="/katex.js?token={}"></script>` before `/app.js`.
- [ ] **Task 2.5:** Write integration tests in `src/viewer/routes.rs`:
  - `authenticated_request_for_katex_js_then_returns_200_and_javascript_mime`
  - `authenticated_request_for_katex_css_then_returns_200_and_css_mime`
  - `unauthenticated_request_for_katex_then_returns_unauthorized`
  - `document_response_then_includes_font_src_data_in_csp`

---

### Slice 3: Client-Side Rendering, Security Controls & Error Fallback (`src/viewer/assets/app.js`, `app.css`)

**Objective:** Implement synchronous KaTeX rendering in `app.js` with non-fatal error handling, strict security bounds (`trust: false`), and responsive styles.

- [ ] **Task 3.1:** Initialize and render math in `src/viewer/assets/app.js`:
  - Scan all `[data-math-inline]`, `[data-math-display]`, and `[data-math-block]` elements.
  - Call `katex.render(source, target, { displayMode: boolean, throwOnError: false, trust: false, maxSize: 500, maxExpand: 1000, strict: "warn" })`.
- [ ] **Task 3.2:** Implement error fallback:
  - If KaTeX encounters invalid syntax, display styled error notice while preserving raw source in a collapsible `<details>` element.
  - Prevent errors from bubbling or blocking document rendering.
- [ ] **Task 3.3:** Add styling rules in `src/viewer/assets/app.css`:
  - Display math centering, overflow scroll for wide equations, font-scaling for table math.
- [ ] **Task 3.4:** Ensure live-refresh hook in `app.js` re-renders math when document content updates.

---

### Slice 4: Full System Verification & Regression Testing (`tests/`)

**Objective:** Verify end-to-end functionality, test real-world documents, and verify zero regressions.

- [ ] **Task 4.1:** Add test fixture `tests/fixtures/math-specification.md`:
  - Include representative systems engineering formulas ($T_{\text{frame}}$, $\mu\text{s}$, $\xrightarrow{\text{1}}$, table formulas, currency signs, and inline code spans with `$`).
  - Verify browser renders all formulas crisply.
- [ ] **Task 4.2:** Regression check existing features:
  - Verify PlantUML diagrams still render and retry properly.
  - Verify Mermaid diagrams still render SVG graphs properly.
  - Verify in-browser source code links and deep line anchors continue to function.
- [ ] **Task 4.3:** Run full repository quality gates:
  - `cargo fmt --check`
  - `cargo clippy --locked --all-targets --all-features -- -D warnings`
  - `cargo test --locked`

---

## Acceptance Criteria Checklist

- [ ] All math in single dollar signs (`$...$`) renders inline without markdown italics/bold distortion.
- [ ] All math in double dollar signs (`$$...$$`) and ````math blocks renders in centered display mode.
- [ ] Code spans and code blocks containing dollar signs remain unaffected.
- [ ] Formulas inside Markdown table cells render cleanly without breaking column alignments.
- [ ] CSP includes `font-src 'self' data:` and permits inlined WOFF2 font loading.
- [ ] Rendering is 100% offline with zero external network requests made to any CDN.
- [ ] Existing PlantUML, Mermaid, and in-browser source code rendering features operate without regression.
- [ ] Code passes all cargo fmt, clippy, and unit/integration test gates.
