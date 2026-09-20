---
type: "Architecture Decision"
title: "ADR-025: In-Browser Source Code Rendering"
description: "Renders qualifying repository source files natively in the browser with line numbering and fragment targeting, superseding default VS Code redirection while preserving optional editor handoff."
id: "ADR-025"
status: "accepted"
date: "2026-09-21"
tags: [architecture, decision, navigation, source-code, viewer, security]
---

# ADR-025: In-Browser Source Code Rendering

Status: accepted

Date: 2026-09-21

Refines and supersedes the editor-only handoff decision in [ADR-021](adr-021-validated-vscode-source-links.md). Uses the active session root selected by [ADR-019](adr-019-repository-scoped-target-sessions.md).

## Context

Repository documentation frequently references manifests, configuration files, and implementation code. Under [ADR-021](adr-021-validated-vscode-source-links.md), Lens rewrote qualifying relative source links in Markdown documents to external `vscode://file/...` platform URLs.

While this avoided adding source file serving routes to Lens at that time, users find the forced editor context switch disruptive and awkward. In typical documentation reading workflows, developers follow source links to inspect referenced logic, verify types, or check configuration values in context—without intending to edit code or desiring an external desktop application to steal focus. Furthermore, in environments lacking a registered VS Code URL handler (such as remote containers, terminals, or systems without VS Code), selecting these links results in unhandled browser protocol errors.

Lens requires a native, in-browser source viewing capability that keeps the developer in their reading flow while maintaining strict filesystem containment, session token authorization, and documentation-focused catalog performance.

## Decision

Lens renders qualifying repository source files directly in the browser under a dedicated loopback route, rather than redirecting exclusively to an external editor.

### 1. Markdown Source Link Rewriting

When rendering a Markdown document, Lens replaces qualifying relative regular-file links with an in-browser route destination:

```text
/source/{percent-encoded-relative-path}
```

- Target qualification retains the strict security rules of [ADR-021](adr-021-validated-vscode-source-links.md): relative candidates must resolve to an existing, readable, visible, non-symbolic regular file within the session's canonical document root.
- Root-crossing, hidden (`.`-prefixed), symbolic, directory, missing, unreadable, and absolute paths remain unauthorized and retain their authored destinations (triggering standard Lens unavailable-document handling upon navigation).
- Discovered Markdown and PlantUML documents retain their `/documents/...` routes with highest precedence.
- Supported line-number fragments (e.g. `#L42` or `#L42-L60`) are preserved on the `/source/...` destination.
- Inline links no longer append the `(opens in VS Code)` indicator text; they render as standard in-viewer navigation links.

### 2. Dedicated On-Demand Source Route (`/source/*source_path`)

Lens introduces an authenticated loopback route:

```text
GET /source/*source_path?token={session_token}
```

- **On-Demand Authorization:** Source files are resolved, authorized, and read on-demand rather than pre-scanned at startup. The document navigation sidebar catalog remains strictly focused on Markdown and PlantUML documentation, preventing catalog bloat and filesystem polling churn across large code trees.
- **Path Resolution & Containment:** The route decodes `source_path`, verifies that no path component is hidden or a symlink, confirms the target is a regular file, canonicalizes the path, and verifies that the canonical path strictly starts with the session's immutable `document_root`.
- **Content Boundaries & Guards:**
  - **Size Cap:** Enforces a maximum file size limit of 2 MB. Files exceeding 2 MB return a clean, styled notice indicating that the file is too large for browser rendering.
  - **Encoding Check:** Source files must be valid UTF-8. Non-UTF-8 or binary files return a styled notice indicating that binary files cannot be displayed as text.
- **Session Authentication:** The route requires a valid `token` matching `state.session_token`. Lens's `inject_capability` helper automatically appends the session token to all emitted `/source/...` links.

### 3. Presentation, Line Numbering & Fragment Targeting

Source documents are rendered within the standard Lens page layout:
- **Header & Breadcrumbs:** Displays the source file's relative path, line count, and byte size.
- **Line Gutter & Semantic Markup:** Code is presented inside a semantic table or ordered list where each line has a dedicated anchor:
  ```html
  <tr id="L42" class="source-line">
    <td class="line-number"><a href="#L42">42</a></td>
    <td class="line-content">...</td>
  </tr>
  ```
- **HTML Escaping:** All source content is strictly HTML-escaped (`&`, `<`, `>`, `"`, `'`).
- **Target Line Highlighting:** Lens's stylesheet defines CSS `:target` rules that visually highlight the targeted line (e.g., `#L42`) with a distinct background tint and scroll offset, enabling deep linking from documentation directly to specific implementation lines.

### 4. Secondary Non-Disruptive Editor Handoff

To support editing without compromising the reading experience, the rendered source view header includes a secondary, optional action link:

```text
Open in VS Code
```

This link uses the validated `vscode://file/{canonical_absolute_path}` URL. Users who choose to edit the file can explicitly trigger editor handoff with one click, while readers remain undisturbed in the browser.

### 5. Automatic Refresh Coexistence

Source views participate in automatic refresh. The source page emits `data-source-path="{path}"` and a dedicated revision polling endpoint (`/revisions/source/*source_path`) checks the target file's modification timestamp. When an external edit is saved, the browser automatically refreshes the rendered code view.

## Consequences

- **Seamless Reading Workflow:** Developers navigate effortlessly between documentation and referenced source code without switching windows or interrupting their reading focus.
- **Environment Agnostic:** Operates reliably across environments regardless of whether VS Code is installed or configured as an OS URL handler.
- **Preserved Security Boundary:** Lens's HTTP surface only serves regular files strictly validated within the immutable document root; no path outside the root can be accessed.
- **Protected Performance:** On-demand resolution prevents scanning or watching thousands of source files in `node_modules`, `target`, or vendor directories during startup.
- **Graceful Fail-Closed Degradation:** Binary files, oversized files, or out-of-root targets display clear diagnostic notices without crashing or hanging the server.
- **Non-Breaking Editor Access:** The optional "Open in VS Code" header link preserves the utility of editor handoff for authors who wish to modify code.

## Alternatives Considered

### 1. Pre-scan all repository source files into the navigation catalog
Scanning all files at startup would populate the sidebar with thousands of build artifacts, test fixtures, and library sources, overwhelming the documentation catalog and exhausting system file watchers. On-demand resolution keeps the catalog clean and startup instant.

### 2. Full server-side syntax highlighting via `syntect`
Adding a heavyweight syntax-highlighting library introduces significant compilation overhead, increases binary size, and requires bundled syntax themes. Starting with clean semantic lines, line numbers, CSS `:target` highlighting, and client-side styling delivers immediate utility with zero bloated dependencies.

### 3. Inline modal or expandable drawer inside Markdown
Displaying source code in an inline modal breaks standard browser URL navigation, disables browser back/forward history, and prevents sharing or bookmarking direct links to specific source lines.

## Trace

- Proposal: [`PROP-IN-BROWSER-SOURCE-RENDERING`](../proposals/in-browser-source-code-rendering.md)
- Use case: [`UC-12`](../features/markdown-viewing/use-cases.md#uc-12-view-a-referenced-repository-file-in-the-browser)
- System sequence: [`SSD-07`](../features/markdown-viewing/ssd-07-view-source-file.md)
- Contract: [`OC-07`](../features/markdown-viewing/oc-07-request-source-document.md)
- Technical design: [`source-rendering-technical-design.md`](../features/markdown-viewing/source-rendering-technical-design.md)
- Construction tasks: [`c10-source-code-rendering-tasks.md`](../iterations/c10-source-code-rendering-tasks.md)
