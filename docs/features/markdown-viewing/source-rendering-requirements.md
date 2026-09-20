---
type: "Feature Specification"
title: "In-Browser Source Code Rendering Requirements"
description: "Product requirements, user stories, example mapping, and acceptance criteria for rendering qualifying repository source files directly within the Lens browser viewer with complete removal of VS Code integration."
id: "FEAT-01-REQ-SOURCE-RENDERING"
status: "active"
scope: "Lens"
tags: [requirements, specification, source-code, viewer, example-mapping]
---

# In-Browser Source Code Rendering Requirements

## 1. Problem Statement & User Value

In Lens, architecture, design, and decision documents frequently reference manifests (`Cargo.toml`, `package.json`), configuration files, test fixtures, and core implementation files (e.g. `[markdown.rs](../src/markdown.rs#L42)`).

Under the previous behavior defined in ADR-021, selecting a qualifying relative source link forced an external application handoff by opening Visual Studio Code via `vscode://file/...`. This introduced major usability and design issues:
- **Disruptive Context Switching:** Reading documentation is primarily an inspection workflow. Developers want to verify types, check logic, or review function signatures without their desktop editor stealing OS focus or opening a separate window.
- **Platform & Environment Failures:** On machines where VS Code is not installed, in browser-based containers, or over remote headless sessions, selecting a source link results in browser error alerts or dead-ends (`The address wasn't understood` or `No application is registered to handle this link`).
- **Awkward Prose Presentation:** Appending `(opens in VS Code)` into every source link cluttered prose and interrupted reading flow.
- **Unnecessary Third-Party Coupling:** Lens should be a self-contained, offline-first tool, independent of any specific commercial editor or proprietary protocol.

### Product Goal

Enable developers and technical writers to inspect referenced repository source files directly inside the Lens browser interface with **zero external VS Code integration**. Source links navigate seamlessly to a clean, line-numbered, in-browser code view that supports deep linking to specific lines, visual line highlighting, and automatic refresh upon file changes.

---

## 2. User Stories

### Story 1: In-Browser Source Inspection
**As a** developer or architect reading repository documentation in Lens,  
**I want to** click a relative link to an implementation or configuration file and view its contents directly in Lens,  
**So that** I can inspect the referenced code without leaving my browser or context-switching to an external editor.

### Story 2: Deep-Link Line Targeting
**As a** technical writer referencing specific functions or structs in documentation,  
**I want to** link to specific source lines using standard fragments (e.g., `#L42`),  
**So that** when readers click the link, Lens automatically scrolls to and visually highlights that line in the source view.

### Story 3: Clean, Self-Contained Typography
**As a** reader reviewing architectural documentation in Lens,  
**I want** source links to render as clean, natural hypertext without external editor tags or `(opens in VS Code)` badges,  
**So that** reading flow is undisturbed.

---

## 3. Collaborative Requirements Discovery (Example Mapping)

```
                           ┌────────────────────────────────────────────────────────┐
                           │                       User Story                       │
                           │   Render qualifying source code directly in browser;   │
                           │         completely remove VS Code integration          │
                           └──────────────────────────┬─────────────────────────────┘
                                                      │
         ┌───────────────────┬────────────────────────┼────────────────────────┐
         ▼                   ▼                        ▼                        ▼
    ┌──────────┐       ┌───────────┐            ┌───────────┐            ┌───────────┐
    │  Rule 1  │       │  Rule 2   │            │  Rule 3   │            │  Rule 4   │
    │ Route &  │       │ Secure    │            │ Line      │            │ Content   │
    │ Rewrite  │       │ On-Demand │            │ Targeting │            │ Guards    │
    └────┬─────┘       └─────┬─────┘            └─────┬─────┘            └─────┬─────┘
         │                   │                        │                        │
         ├─────────┐         ├─────────┐              ├─────────┐              ├─────────┐
         ▼         ▼         ▼         ▼              ▼         ▼              ▼         ▼
     [Ex 1.1]  [Ex 1.2]  [Ex 2.1]  [Ex 2.2]       [Ex 3.1]  [Ex 3.2]       [Ex 4.1]  [Ex 4.2]
     Relative  Doc route Validated Root-crossing  #L42      No fragment    Binary    >2 MB
     file link takes     regular   rejected       scrolls   renders whole  rejected  rejected
     -> /source priority file      404            & glows   file           notice    notice
```

### Business Rules & Concrete Examples

#### Rule 1: Link Resolution & Total VS Code Elimination
*Relative links in Markdown documents targeting qualifying source files must rewrite to `/source/{path}` with authored fragments. No `vscode://` URLs or `(opens in VS Code)` text are generated.*
- **Example 1.1 (Standard source link):** A document contains `[Main](src/main.rs#L10)`. Lens rewrites the destination to `/source/src/main.rs?token=...#L10`. No `(opens in VS Code)` span is appended.
- **Example 1.2 (Markdown precedence):** A document contains `[Architecture](docs/architecture.md)`. Because `docs/architecture.md` is a discovered document, it retains `/documents/docs/architecture.md` rather than `/source/`.
- **Example 1.3 (Authored vscode link):** An authored external link `[Link](vscode://file/...)` is treated as an ordinary external URL without validation or special handling.

#### Rule 2: Secure On-Demand Resolution
*The `/source/*source_path` route must validate that the target is a regular, visible, readable file strictly inside the session document root.*
- **Example 2.1 (Valid source file):** Navigating to `/source/src/markdown.rs?token={valid_token}` resolves the file beneath `document_root`, loads its text, and renders the source viewer page with HTTP 200.
- **Example 2.2 (Path traversal attempt):** A request to `/source/../../etc/passwd` or `/source/../secret.txt` fails containment checks and returns the standard document unavailable 404 page.
- **Example 2.3 (Hidden file rejection):** Requesting `/source/.git/config` or `/source/.env` fails hidden component checks and returns 404.
- **Example 2.4 (Symlink rejection):** Requesting a symlinked file inside the repository returns 404.
- **Example 2.5 (Directory rejection):** Requesting `/source/src` returns 404 without attempting directory listing.

#### Rule 3: Line Numbering & Deep Linking
*Source code must be formatted with an accessible line gutter where each line has an anchor (`L{number}`) that responds to URL fragments.*
- **Example 3.1 (Fragment navigation):** Visiting `/source/src/lib.rs?token=...#L25` highlights line 25 with the `:target` CSS rule and positions the viewport at line 25.
- **Example 3.2 (Line clickability):** Clicking line number `42` updates the browser URL hash to `#L42`, allowing easy link copying.
- **Example 3.3 (HTML safety):** A Rust file containing `if a < b && b > c` is escaped to `&lt;` and `&gt;` so no raw HTML is injected into the DOM.

#### Rule 4: Content Boundaries & Graceful Degradation
*Files that are binary or exceed safe size limits must display helpful diagnostic notices rather than breaking the viewer or consuming excessive memory.*
- **Example 4.1 (Binary file):** Linking to a compiled `.wasm`, `.png`, or binary fixture under `/source/fixtures/test.bin` detects non-UTF-8 content and renders a clean message: *"Binary file cannot be displayed as text."*
- **Example 4.2 (Oversized file):** Linking to a 10 MB generated log or database file exceeds the 2 MB limit and displays: *"File size (10.0 MB) exceeds maximum supported view limit (2.0 MB)."*

#### Rule 5: Automatic Refresh Support
*Source code pages must participate in live polling so that modifying and saving the file externally updates the browser view automatically.*
- **Example 5.1 (Live refresh):** The user is viewing `/source/src/main.rs`. In an external terminal or editor, they edit `main.rs` and save. Lens's background revision poller detects the file modification and refreshes the browser page seamlessly.
