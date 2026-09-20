# Technical Design: In-Browser Source Code Rendering and Pruning of VS Code Integration

## Objective

Extend [Lens](file:///home/ccwu/lens/README.md) to render qualifying repository source files natively in the browser viewer and **entirely remove** all VS Code URL generation, external scheme handling, and vestigial editor contracts, keeping Lens 100% self-contained and offline.

## Tech Stack

- **Language / Runtime:** Rust 1.75+ (2021 edition)
- **HTTP / Loopback Server:** `axum` (v0.6.20)
- **HTML Escaping & Markdown AST:** `pulldown-cmark` (v0.9.3)
- **Styling:** Semantic HTML5 table with native CSS `:target` selectors and line gutters
- **Target OS:** Cross-platform (Linux, macOS, Windows)

## Architecture Overview

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                            Markdown Document View                           │
│  [Source Link](src/markdown.rs#L42)                                        │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │ Rewritten to /source/...
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                       Axum Route: GET /source/*path                         │
│  1. Session Auth (token == session_token)                                   │
│  2. SourceLinkResolver::resolve_source_content(path)                        │
│     - Rejects .. traversal, hidden parts, symlinks, directories             │
│     - Validates canonical path starts with document_root                    │
│  3. Content Boundary Checks                                                 │
│     - Size <= 2 MB                                                          │
│     - UTF-8 text encoding check                                             │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Rendered Source Code HTML Page                       │
│  - Document Header: Breadcrumb (← Documentation), path, line count, size    │
│  - Semantic Code Gutter: <tr id="L42"><td>42</td><td>...</td></tr>         │
│  - CSS :target highlight rule: amber background tint & scroll position      │
│  - Auto-refresh polling hook: data-source-path="src/markdown.rs"            │
│  - ZERO external editor links or vscode:// references                       │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Vestigial Contract & Code Pruning Audit

Under the `audit-vestigial-contracts` methodology, removing VS Code integration allows pruning obsolete helpers, constants, and parameters:

1. **`src/source_link.rs` Pruning:**
   - **Delete constant:** `VSCODE_PATH_ENCODE_SET` (AsciiSet used only for `vscode://file/`).
   - **Delete helper:** `fn vscode_url(path: &Path) -> Option<String>`.
   - **Delete helper:** `fn has_ambiguous_vscode_position_suffix(path: &str) -> bool` (previously needed because VS Code parsed trailing `:digits` as `:line:col`).
   - **Prune tests:** Remove tests asserting `vscode://file/...` URI formatting, Windows drive colon preservation, and colon-digit suffix rejection.
2. **`src/markdown.rs` Pruning:**
   - **Remove field:** `ResolvedLink::opens_in_vscode`.
   - **Delete state:** `source_link_stack: Vec<bool>` (previously pushed boolean to emit indicator).
   - **Delete markup insertion:** `<span class="source-link-indicator"> (opens in VS Code)</span>`.
   - **Simplify suffix parsing:** Supported line fragments (`#L42`) are directly preserved on the `/source/...` path instead of being translated into `:line:1`.
3. **`src/viewer/assets/app.css` Pruning:**
   - **Remove class:** `.source-link-indicator` (and any associated styling).

## Module Contracts & Additions

### 1. Link Rewriter (`src/markdown.rs`)

Modify `resolve_link`:
- If `known_documents.contains(&candidate)`: returns `/documents/{candidate}{suffix}`.
- If target qualifies as a regular source file:
  - Formats destination as `/source/{normalized_path}{suffix}`.
  - Emits clean anchor without any indicator text or external scheme.
- Disallowed paths retain authored destination (falling through to 404).

### 2. Source Authorization & Content Resolution (`src/source_link.rs`)

`SourceLinkResolver` becomes the sole domain authority for repository source inspection:

```rust
pub(crate) struct AuthorizedSourceFile {
    pub(crate) relative_identifier: String,
    pub(crate) canonical_path: PathBuf,
    pub(crate) content: String,
    pub(crate) line_count: usize,
    pub(crate) byte_size: u64,
}

pub(crate) enum SourceResolution {
    Authorized(AuthorizedSourceFile),
    TooLarge { byte_size: u64, max_allowed: u64 },
    Binary,
    NotFoundOrUnauthorized,
}

impl SourceLinkResolver {
    pub(crate) fn resolve_source_content(&self, relative_path: &str) -> SourceResolution;
}
```

#### Authorization Invariants:
1. Candidate path components must not start with `.`.
2. No component may be a symbolic link (`symlink_metadata`).
3. Must be a regular file (`metadata.is_file()`).
4. Canonical path must begin with `self.document_root`.
5. Size check: File size must not exceed `MAX_SOURCE_SIZE = 2 * 1024 * 1024` (2 MB).
6. UTF-8 check: `fs::read()` followed by `String::from_utf8()`. If UTF-8 parsing fails, return `SourceResolution::Binary`.

### 3. Route Handlers (`src/viewer/routes.rs`)

Add routes to `router`:
```rust
.route("/source/*source_path", get(source_view))
.route("/revisions/source/*source_path", get(source_revision))
```

- `source_view`:
  - Validates session token via existing `session_auth_middleware`.
  - Calls `state.source_resolver.resolve_source_content(source_path)`.
  - Returns `Html(source_page(...))` with appropriate headers (CSP, no-referrer).
- `source_revision`:
  - Returns the mtime timestamp of the source file for live refresh.

### 4. Page Template & HTML Generation (`src/viewer/page.rs`)

Add `source_page`:
```rust
pub(super) fn source_page(
    relative_path: &str,
    source: &str,
    session_token: &str,
) -> String
```

Generates markup:
```html
<main class="source-view" data-source-path="{escaped_path}" data-session-token="{token}">
  <header class="document-header">
    <p class="eyebrow"><a href="/?token={token}">← Documentation</a></p>
    <div class="source-header-row">
      <h1>{escaped_path}</h1>
      <span class="source-meta">{lines} lines • {size}</span>
    </div>
  </header>
  <article class="source-content">
    <table class="source-table">
      <tbody>
        <tr id="L1"><td class="line-num"><a href="#L1">1</a></td><td class="line-code">...</td></tr>
        ...
      </tbody>
    </table>
  </article>
</main>
```

Update `inject_capability`:
- Ensure `href="/source/"` URLs have `?token={token}` injected dynamically.

### 5. Stylesheet (`src/viewer/assets/app.css`)

Add CSS styles for source table and `:target` highlight:
```css
.source-table {
  width: 100%;
  border-collapse: collapse;
  font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
  font-size: 0.875rem;
  line-height: 1.5;
}

.source-table tr:target {
  background-color: rgba(234, 179, 8, 0.2); /* Soft amber highlight */
  border-left: 3px solid #eab308;
}

.line-num {
  width: 1%;
  min-width: 48px;
  padding: 0 12px;
  text-align: right;
  user-select: none;
  color: #888;
  border-right: 1px solid #e5e7eb;
}

.line-code {
  padding: 0 16px;
  white-space: pre;
}
```

## Security Invariants

1. **Root Containment:** No request to `/source/` can access files outside `ViewerState.document_root`.
2. **Hidden File Masking:** `.git`, `.env`, and other dot-files cannot be accessed.
3. **Memory Protection:** 2 MB size ceiling prevents high memory usage from accidental links to large log files or binaries.
4. **Content-Type & XSS Defense:** Strict HTML entity escaping for all code content; strict CSP headers (`default-src 'self'`).
