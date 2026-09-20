---
type: "Architecture Decision"
title: "ADR-025: In-Browser Source Code Rendering and Removal of VS Code Integration"
description: "Renders qualifying repository source files natively in the browser with line numbering and fragment targeting, and entirely removes external VS Code URL generation and editor dependencies."
id: "ADR-025"
status: "accepted"
date: "2026-09-21"
tags: [architecture, decision, navigation, source-code, viewer, security]
---

# ADR-025: In-Browser Source Code Rendering and Removal of VS Code Integration

Status: accepted

Date: 2026-09-21

Supersedes [ADR-021: Emit Validated VS Code Source Links](adr-021-validated-vscode-source-links.md). Uses the active session root selected by [ADR-019](adr-019-repository-scoped-target-sessions.md).

## Context

Repository documentation frequently references manifests, configuration files, and implementation code. Under [ADR-021](adr-021-validated-vscode-source-links.md), Lens rewrote qualifying relative source links in Markdown documents to external `vscode://file/...` platform URLs.

External editor redirection proved awkward and disruptive in practice:
- Reading documentation is primarily an inspection workflow. Forcing an external desktop application to steal operating-system focus when following an implementation reference breaks reading flow.
- In headless environments, browser-based containers, or setups without VS Code registered with the operating-system URL handler, clicking a source link fails with unhandled browser errors.
- The `(opens in VS Code)` indication cluttered Markdown typography.
- Coupling Lens to an external vendor editor protocol (`vscode://`) compromised Lens's design identity as a self-contained, offline repository viewer.

Lens requires a native, 100% self-contained in-browser source viewing capability that keeps the developer in their reading flow while maintaining strict filesystem containment, session token authorization, and documentation-focused catalog performance, with all VS Code integration entirely removed.

## Decision

Lens renders qualifying repository source files directly in the browser under a dedicated loopback route and entirely removes all VS Code URL generation, custom scheme handling, and editor handoff mechanisms.

### 1. Complete Removal of VS Code Integration

- Lens generates **no** `vscode://` URLs, supports no editor scheme options, and emits no external application handoff markup.
- The `(opens in VS Code)` indicator text and associated `.source-link-indicator` markup and styles are completely removed.
- Vestigial VS Code helpers (such as `vscode_url`, `VSCODE_PATH_ENCODE_SET`, and ambiguous position suffix checks tailored for VS Code's `:line:col` syntax) are excised.
- [ADR-021](adr-021-validated-vscode-source-links.md) and [`UC-06`](../features/markdown-viewing/use-cases.md#uc-06-open-a-referenced-repository-file-in-vs-code) are superseded.

### 2. Markdown Source Link Rewriting

When rendering a Markdown document, Lens replaces qualifying relative regular-file links with an in-browser route destination:

```text
/source/{percent-encoded-relative-path}
```

- Target qualification retains the strict security rules of ADR-021: relative candidates must resolve to an existing, readable, visible, non-symbolic regular file within the session's canonical document root.
- Root-crossing, hidden (`.`-prefixed), symbolic, directory, missing, unreadable, and absolute paths remain unauthorized and retain their authored destinations (triggering standard Lens unavailable-document handling upon navigation).
- Discovered Markdown and PlantUML documents retain their `/documents/...` routes with highest precedence.
- Supported line-number fragments (e.g. `#L42` or `#L42-L60`) are preserved on the `/source/...` destination.
- Links render as ordinary in-viewer navigation links without editor annotations.

### 3. Dedicated On-Demand Source Route (`/source/*source_path`)

Lens introduces an authenticated loopback route:

```text
GET /source/*source_path?token={session_token}
```

- **On-Demand Authorization:** Source files are resolved, authorized, and read on-demand rather than pre-scanned at startup. The document navigation sidebar catalog remains strictly focused on Markdown and PlantUML documentation, preventing catalog bloat and filesystem polling churn across large code trees.
- **Path Resolution & Containment:** The route decodes `source_path`, verifies that no path component is hidden or a symlink, confirms the target is a regular file, canonicalizes the path, and verifies that the canonical path strictly starts with the session's immutable `document_root`.
- **Content Boundaries & Guards:**
  - **Size Cap:** Enforces a maximum file size limit of 2 MB. Files exceeding 2 MB return a clean, styled notice indicating that the file is too large for browser rendering.
  - **Encoding & Binary Check:** Source files must be valid UTF-8 and must not contain NUL (`\0`) bytes. Binary files or non-UTF-8 content return a styled notice indicating that binary files cannot be displayed as text.
- **Session Authentication:** The route requires a valid `token` matching `state.session_token`. Lens's `inject_capability` helper automatically appends the session token to all emitted `/source/...` links.

### 4. Presentation, Line Numbering & Deep Linking

Source documents are rendered within the standard Lens page layout:
- **Header & Breadcrumbs:** Displays the source file's relative path, line count, byte size, and a back-link to documentation.
- **Line Gutter & Semantic Markup:** Code is presented inside a semantic table where each line has a dedicated anchor:
  ```html
  <tr id="L42" class="source-line">
    <td class="line-number"><a href="#L42">42</a></td>
    <td class="line-content">...</td>
  </tr>
  ```
- **HTML Escaping:** All source content is strictly HTML-escaped (`&`, `<`, `>`, `"`, `'`).
- **Target Line Highlighting:** Lens's stylesheet defines CSS `:target` rules that visually highlight the targeted line (e.g., `#L42`) with a distinct background tint and scroll offset, enabling deep linking from documentation directly to specific implementation lines.

### 5. Automatic Refresh Coexistence

Source views participate in automatic refresh. The source page emits `data-source-path="{path}"` and a dedicated revision polling endpoint (`/revisions/source/*source_path`) checks the target file's modification timestamp. When an external edit is saved, the browser automatically refreshes the rendered code view.

## Consequences

- **100% Self-Contained:** Lens operates as a complete, self-sufficient repository reader with zero external application dependencies or URL scheme couplings.
- **Seamless Reading Workflow:** Developers navigate effortlessly between documentation and referenced source code without switching windows or interrupting their reading focus.
- **Environment Agnostic:** Operates reliably across headless environments, containers, and machines without any desktop editor installed.
- **Codebase Simplification:** Eliminates custom-scheme URL encoding, ambiguous suffix workarounds, and editor-specific indicators.
- **Preserved Security Boundary:** Lens's HTTP surface only serves regular files strictly validated within the immutable document root; no path outside the root can be accessed.
- **Protected Performance:** On-demand resolution prevents scanning or watching thousands of source files in `node_modules`, `target`, or vendor directories during startup.

## Alternatives Considered

### 1. Optional "Open in VS Code" secondary button
Providing a secondary editor link was considered, but rejected to maintain Lens as a pure, focused viewer unencumbered by external editor ties. Users who wish to edit code can open files via their terminal or preferred editor independently.

### 2. Pre-scan all repository source files into the navigation catalog
Scanning all files at startup would populate the sidebar with thousands of build artifacts, test fixtures, and library sources, overwhelming the documentation catalog and exhausting system file watchers. On-demand resolution keeps the catalog clean and startup instant.

### 3. Full server-side syntax highlighting via `syntect`
Adding a heavyweight syntax-highlighting library introduces significant compilation overhead, increases binary size, and requires bundled syntax themes. Starting with clean semantic lines, line numbers, CSS `:target` highlighting, and client-side styling delivers immediate utility with zero bloated dependencies.

## Trace

- Proposal: [`PROP-IN-BROWSER-SOURCE-RENDERING`](../proposals/in-browser-source-code-rendering.md)
- Use case: [`UC-12`](../features/markdown-viewing/use-cases.md#uc-12-view-a-referenced-repository-file-in-the-browser)
- System sequence: [`SSD-07`](../features/markdown-viewing/ssd-07-view-source-file.md)
- Contract: [`OC-07`](../features/markdown-viewing/oc-07-request-source-document.md)
- Technical design: [`source-rendering-technical-design.md`](../features/markdown-viewing/source-rendering-technical-design.md)
- Construction tasks: [`c10-source-code-rendering-tasks.md`](../iterations/c10-source-code-rendering-tasks.md)
