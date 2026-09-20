---
type: "System Sequence Diagram"
title: "SSD-07: View a Referenced Repository Source File"
description: "Shows Lens returning an in-browser source document route and rendering line-numbered source code with zero external editor dependencies."
id: "SSD-07"
use_case: "UC-12"
scenario: "Render and view a qualifying repository source file inside Lens."
status: "active"
tags: [analysis, ssd, source-code, viewer]
---

# SSD-07: View a Referenced Repository Source File

Use case: `UC-12`

Scenario: The developer requests a known Markdown document containing a relative link to a repository source file, selects the link, and views the source code directly within Lens.

## Actors

- Developer or technical writer
- Operating system browser
- Lens Viewer Loopback Service
- Filesystem (session document root)

## System Events

```text
Developer             Browser                      Lens Loopback                    Filesystem
    │                    │                               │                              │
    │ 1. Click source    │                               │                              │
    │    link in doc     │                               │                              │
    │───────────────────>│                               │                              │
    │                    │ 2. GET /source/{path}?token   │                              │
    │                    │──────────────────────────────>│                              │
    │                    │                               │ 3. Resolve & authorize path  │
    │                    │                               │─────────────────────────────>│
    │                    │                               │ 4. Read metadata & content   │
    │                    │                               │<─────────────────────────────│
    │                    │                               │                              │
    │                    │                               │ 5. Validate containment,     │
    │                    │                               │    size <= 2MB, UTF-8 text   │
    │                    │                               │                              │
    │                    │ 6. HTTP 200 (HTML with lines, │                              │
    │                    │    gutter & line anchors)     │                              │
    │                    │<──────────────────────────────│                              │
    │                    │                               │                              │
    │ 7. View code,      │                               │                              │
    │    scroll to #L42  │                               │                              │
    │<───────────────────│                               │                              │
```

## Discovered System Operations

- `request_source(source_path)`: Resolves an on-demand relative path against the fixed session root, enforces filesystem containment, validates regular file readability, verifies UTF-8 text and size limits, and returns a fully styled HTML page with line numbers and anchors.
- `request_source_revision(source_path)`: Polls the modification time of an authorized source file to trigger client-side auto-refresh when changes are detected.

## Extensions & Failure Cases

- **Containment or Access Failure:** If the requested path escapes the document root, attempts directory traversal, targets a hidden file or directory, targets a symlink, or does not exist, Lens returns HTTP 404 with the standard document unavailable guidance.
- **Binary Content:** If the file contains non-UTF-8 bytes or null characters, Lens returns a styled diagnostic notification explaining that binary assets cannot be rendered as text.
- **Oversized Content:** If the file exceeds 2 MB, Lens returns a diagnostic notification reporting the file size and the view limit, preventing browser hangs.
