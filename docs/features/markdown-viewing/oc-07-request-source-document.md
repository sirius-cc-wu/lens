---
type: "Operation Contract"
title: "OC-07: Request Source Document"
description: "Specifies authorization, containment, encoding, and markup guarantees for the request_source operation with zero external editor dependencies."
id: "OC-07"
operation: "request_source(source_path)"
traces: [UC-12, SSD-07]
status: "active"
tags: [analysis, operation-contract, source-code, viewer]
---

# OC-07: Request Source Document

Operation: `request_source(source_path)`

Cross References: `UC-12`, [SSD-07](ssd-07-view-source-file.md), and [ADR-025](../../decisions/adr-025-in-browser-source-code-rendering.md)

Scope: Lens

## Preconditions

- A Lens viewing session is active with an immutable canonical document root and a valid session token.
- The incoming HTTP request includes a valid session token in the query string (`?token=...`).

## Postconditions on an Authorized, Valid Source File

- **Filesystem Containment:** The decoded relative path `source_path` is verified to contain no `..` traversal escaping the root, no hidden components, and no symbolic links.
- **Regular File Validation:** The resolved file exists, is a regular file (not a directory or FIFO), and is readable by the Lens process. Its canonicalized absolute path begins with `document_root`.
- **Size Boundary:** The file size is less than or equal to 2,097,152 bytes (2 MB).
- **Encoding Validation:** The file content is valid UTF-8.
- **Markup Guarantees:**
  - The returned response has HTTP status 200 and Content-Type `text/html; charset=utf-8`.
  - Every line of source code is HTML-escaped (`&`, `<`, `>`, `"`, `'`).
  - Each line is enclosed within an identifiable DOM element with `id="L{line_number}"` (1-indexed).
  - An accessible gutter renders line numbers linking to their respective `#L{line_number}` hash.
  - The document header renders the file's repository-relative path, total line count, and byte size.
  - No editor URLs, `vscode://` links, or external protocol calls are emitted.
  - The page root carries `data-source-path="{source_path}"` to enable client-side live refresh.

## Postconditions on Disallowed or Unavailable Targets

- If the path targets a non-existent file, a directory, a symlink, a hidden path, or traverses outside `document_root`, Lens returns HTTP 404 with the standard Lens document unavailable page.
- If the target file exceeds 2 MB, Lens returns HTTP 200 with an informative page indicating that the file is too large to render in the browser.
- If the target file contains non-UTF-8 bytes, Lens returns HTTP 200 with an informative page indicating that binary files cannot be displayed as text.

## Refresh Guarantees

- A companion operation `request_source_revision(source_path)` returns the current modification timestamp (mtime) as an integer.
- If the file on disk changes, the client detects the new revision and reloads the view, retaining any line hash (`#L{n}`).
