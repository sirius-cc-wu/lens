---
type: "Iteration Plan & Task Board"
title: "Iteration C10: In-Browser Source Code Rendering Implementation & VS Code Pruning"
description: "Structured work packages, test specifications, and verification checklists for downstream Workers implementing in-browser source rendering and entirely pruning VS Code integration."
id: "C10"
status: "ready"
tags: [iteration, tasks, planning, worker-handoff]
---

# Iteration C10: In-Browser Source Code Rendering & VS Code Pruning

## Overview

This task board breaks the architectural specification of [ADR-025](../decisions/adr-025-in-browser-source-code-rendering.md), [FEAT-01-REQ-SOURCE-RENDERING](../features/markdown-viewing/source-rendering-requirements.md), [SSD-07](../features/markdown-viewing/ssd-07-view-source-file.md), [OC-07](../features/markdown-viewing/oc-07-request-source-document.md), and [Technical Design](../features/markdown-viewing/source-rendering-technical-design.md) into 4 discrete, test-driven vertical slices for execution by downstream Workers.

---

## Work Packages & Execution Slices

### Slice 1: Source Content Authorization & Prune Vestigial Helpers (`src/source_link.rs`)

**Objective:** Extend `SourceLinkResolver` to read and authorize source file contents safely on-demand with size caps and UTF-8 verification, while pruning dead VS Code URL helpers.

- [ ] **Task 1.1:** Define `AuthorizedSourceFile` and `SourceResolution` enum (`Authorized`, `TooLarge`, `Binary`, `NotFoundOrUnauthorized`).
- [ ] **Task 1.2:** Implement `SourceLinkResolver::resolve_source_content(&self, relative_path: &str) -> SourceResolution`.
  - Validate containment against `document_root`.
  - Reject hidden components and symlinks.
  - Enforce `MAX_SOURCE_SIZE = 2 * 1024 * 1024` (2 MB).
  - Verify UTF-8 valid byte sequence.
- [ ] **Task 1.3:** Prune vestigial VS Code code from `src/source_link.rs`:
  - Remove `VSCODE_PATH_ENCODE_SET`.
  - Remove `fn vscode_url(...)`.
  - Remove `fn has_ambiguous_vscode_position_suffix(...)`.
- [ ] **Task 1.4:** Write 3A unit tests following `<condition_or_action>_then_<observable_result>` naming:
  - `valid_repository_source_file_then_resolves_content_and_metadata`
  - `target_outside_document_root_then_returns_not_found`
  - `hidden_target_path_then_returns_not_found`
  - `symlinked_file_target_then_returns_not_found`
  - `file_exceeding_size_limit_then_returns_too_large`
  - `binary_file_with_null_bytes_then_returns_binary`

---

### Slice 2: Markdown Link Rewriting & Prune Indicator (`src/markdown.rs`)

**Objective:** Transition relative link rewriting to `/source/...` destinations and prune all VS Code indicator scaffolding.

- [ ] **Task 2.1:** Update `resolve_link` in `src/markdown.rs`:
  - When target qualifies as a regular source file, rewrite destination to `/source/{normalized_path}{suffix}`.
  - Remove `opens_in_vscode` field from `ResolvedLink`.
  - Remove `source_link_stack` and `<span class="source-link-indicator"> (opens in VS Code)</span>`.
  - Preserve discovered document routes (`/documents/...`) as highest precedence.
  - Preserve line number fragments (`#L42`).
- [ ] **Task 2.2:** Update `tests` in `src/markdown.rs`:
  - `relative_source_link_then_rewrites_to_source_route_without_vscode_indicator`
  - `source_link_with_line_fragment_then_preserves_fragment_on_source_route`
  - `known_markdown_document_then_retains_documents_route_precedence`

---

### Slice 3: Loopback Route & In-Browser View Rendering (`src/viewer/`)

**Objective:** Implement `/source/*source_path` route and semantic line-numbered HTML generation.

- [ ] **Task 3.1:** Add `source_page` in `src/viewer/page.rs`:
  - Render breadcrumb navigation (`← Documentation`) and file metadata.
  - Format code in semantic table with `<tr id="L{n}">` and `<td class="line-num"><a href="#L{n}">{n}</a></td>`.
  - Contain ZERO VS Code references.
- [ ] **Task 3.2:** Update `inject_capability` in `src/viewer/page.rs`:
  - Dynamically inject `?token={session_token}` into all `href="/source/..."` URLs.
- [ ] **Task 3.3:** Add Axum routes in `src/viewer/routes.rs`:
  - `GET /source/*source_path` -> `source_view`
  - `GET /revisions/source/*source_path` -> `source_revision`
- [ ] **Task 3.4:** Add CSS styles in `src/viewer/assets/app.css`:
  - Monospace font, table layout, line gutter styling.
  - `tr:target` rule for amber highlight and visual focus.
  - Remove obsolete `.source-link-indicator` CSS rule.
- [ ] **Task 3.5:** Write integration route tests:
  - `authenticated_request_to_valid_source_then_returns_html_with_lines_and_token`
  - `unauthenticated_request_to_source_then_returns_unauthorized`
  - `request_to_unauthorized_source_then_returns_document_unavailable_page`
  - `request_to_binary_file_then_returns_binary_notice`
  - `request_to_oversized_file_then_returns_size_limit_notice`

---

### Slice 4: Live Refresh & Verification (`src/viewer/assets/app.js` & Acceptance)

**Objective:** Wire up client-side auto-refresh and conduct complete qualification checks.

- [ ] **Task 4.1:** Update `app.js` to poll `/revisions/source/*` when `main[data-source-path]` is active.
- [ ] **Task 4.2:** Verify end-to-end flow with manual and browser integration tests.
- [ ] **Task 4.3:** Run full repository lint and test verification:
  - `cargo fmt --check`
  - `cargo clippy --locked --all-targets --all-features -- -D warnings`
  - `cargo test --locked`

---

## Acceptance Criteria Checklist

- [ ] Clicking a relative source link in a rendered Markdown document navigates to `/source/...` in the same browser session.
- [ ] No `(opens in VS Code)` indicator text or `vscode://` URLs are generated anywhere in markup or headers.
- [ ] The rendered source view contains line numbers and clickable anchor links.
- [ ] Fragment URLs like `/source/src/main.rs#L42` highlight line 42 with the `:target` CSS selector.
- [ ] Files larger than 2 MB display a clean size warning instead of hanging or crashing.
- [ ] Binary files display a clean binary asset notice.
- [ ] Directory traversal attempts (e.g. `../../etc/passwd`) return HTTP 404.
- [ ] Live refresh automatically updates the view when the file is modified on disk.
