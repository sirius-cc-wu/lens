---
type: "Improvement Proposal"
title: "Open Source-File Links in Visual Studio Code"
description: "Rewrites validated repository file links so selecting one opens the local file in Visual Studio Code without serving its contents through Lens."
id: "PROP-OPEN-SOURCE-LINKS-IN-VSCODE"
status: "implemented"
tags: [proposal, navigation, source-code, vscode]
---

# Open Source-File Links in Visual Studio Code

Status: implemented through D5, C8, and C9

Implementation trace:

- [ADR-020: Emit validated VS Code source links](../decisions/adr-020-validated-vscode-source-links.md)
- [D5 design iteration](../iterations/d5-validated-vscode-source-link-design.md)
- [C8 authorization iteration](../iterations/c8-source-link-authorization.md)
- [C9 transition iteration](../iterations/c9-accessible-source-link-handoff.md)

## Summary

When a rendered Markdown document links to a source file, selecting the link
should open that local file in Visual Studio Code (VS Code). Lens should
recognize a relative link whose target is an existing, visible, regular file
inside the viewing session's fixed document root and rewrite it to VS Code's
platform URL:

```text
vscode://file/{absolute-file-path}
```

Links to discovered Markdown and PlantUML documents should retain their Lens
navigation behavior. External links, same-document fragments, directories,
missing files, hidden paths, symbolic links, and paths outside the document
root should not become VS Code links.

Lens should validate each source-file target before emitting the URL. It should
not add a browser route that accepts a filesystem path, read the source file
into the browser response, or start a `code` process when the link is selected.
The browser and operating system should hand the validated `vscode:` URL to the
registered VS Code installation.

## Motivation

Lens is useful for reading architecture, requirements, decisions, and
iteration evidence, but those documents frequently refer to the implementation
that realizes them. Today a relative link such as
`../../src/markdown.rs` is not a discovered document, so Lens preserves the
destination and the browser eventually receives Lens's unavailable-document
guidance instead of opening the implementation.

This interrupts a common repository-reading workflow:

1. Read an explanation in Lens.
2. Follow its implementation reference.
3. Inspect or edit the referenced file in the editor.
4. Return to the document in Lens.

Opening the file in VS Code completes that workflow while preserving Lens as a
focused documentation viewer. Lens does not need to add syntax highlighting,
source-file navigation, editing, or another browser content type.

The URL form is a supported VS Code integration. The
[VS Code command-line documentation](https://code.visualstudio.com/docs/configure/command-line#_opening-vs-code-with-urls)
defines `vscode://file/{full path to file}` for opening a file through the
platform URL handler.

## Proposed Behavior

A source-file link is a Markdown link with a relative filesystem destination
that resolves to a qualifying repository file. "Source file" is intentionally
based on the target's properties rather than a programming-language extension
list: architecture documents also need to reach manifests, build files,
fixtures, and extensionless scripts.

| Link target | Lens behavior |
|---|---|
| A discovered `.md`, `.markdown`, or `.puml` document | Preserve the existing Lens document route and in-browser navigation. |
| An existing visible regular file inside the fixed document root | Emit a `vscode://file/...` link for its canonical absolute path. |
| A directory | Preserve the existing unavailable-document behavior; do not open a VS Code folder or workspace. |
| A missing, unreadable, hidden, or symbolic-link target | Preserve the existing unavailable-document behavior; do not emit a `vscode:` URL. |
| A path outside the document root after normalization or canonicalization | Preserve the existing unavailable-document behavior; do not emit a `vscode:` URL. |
| An absolute local path | Preserve the existing behavior; absolute paths are not authorized by the document. |
| An external URL or non-file scheme | Preserve the authored destination and standard browser behavior. |
| An authored `vscode:` URL | Preserve it as an external destination; Lens does not validate or endorse its target. |
| A same-document fragment | Preserve normal in-page navigation. |

For example, when Lens is started at the repository root and
`docs/design.md` contains:

```markdown
[Markdown rendering](../src/markdown.rs)
```

Lens should render an ordinary accessible link whose destination is equivalent
to:

```text
vscode://file/<canonical-repository-root>/src/markdown.rs
```

The absolute path must be encoded as a URL without changing its filesystem
meaning. Spaces, non-ASCII characters, Windows drive letters, and platform path
separators require explicit cross-platform coverage. This proposal opens the
file at its beginning. Translating source-position fragments into a line and
column is separate follow-up behavior.

## User Scenario

Primary actor: Developer or technical writer

Goal: Move from repository documentation to its referenced implementation in
VS Code with one link selection.

Preconditions:

- Lens has an active viewing session.
- The current Markdown document contains a relative link to a qualifying file
  inside the session root.
- VS Code is installed and registered as the handler for the `vscode:` URL
  scheme.

Main success scenario:

1. Lens renders the Markdown document.
2. Lens resolves the relative file destination from the current document.
3. Lens verifies that the target is a visible, non-symbolic-link regular file
   inside the fixed document root.
4. Lens renders the link with a properly encoded `vscode://file/...`
   destination.
5. The user selects the link.
6. The browser asks the operating system to open the URL.
7. VS Code opens the selected file.

Extensions:

- 3a. If the target is a discovered document, Lens renders its existing local
  document route instead.
- 3b. If the target does not qualify, Lens does not generate a VS Code URL and
  retains its current guidance behavior.
- 6a. The browser may ask the user for confirmation before launching an
  external application.
- 6b. If no application handles `vscode:`, the browser reports that failure;
  the Lens page remains usable and Lens starts no fallback process.

## Resolution and Authorization

Source-link resolution should reuse the document-link vocabulary where
possible, but it has a different outcome: it authorizes an editor handoff, not
a Lens content response.

For each candidate relative link, Lens should:

1. Separate the filesystem path from any query or fragment without treating
   either suffix as part of the path.
2. Resolve the path relative to the Markdown document that contains the link.
3. Reject absolute paths, invalid encodings, traversal beyond the fixed root,
   and platform-specific path forms that could bypass component checks.
4. Inspect the target without following any symbolic-link path component.
5. Require a regular file whose canonical path remains within the canonical
   document root.
6. Reject a target beneath any hidden path component.
7. Prefer the existing Lens route when the target is an authorized Markdown or
   PlantUML document.
8. Encode the validated canonical path in a `vscode://file/...` URL.

The browser request never supplies the path to this operation. Lens derives it
only while rendering an already authorized Markdown document. A link therefore
cannot add an HTTP-readable file to the viewing session, and no loopback route
should translate a browser-provided identifier into a source path.

Validation may occur while the document is rendered, including after Lens
detects a document change. The document root remains fixed for the session, so
a changed link can select a different in-root file but cannot broaden the
filesystem boundary.

## Security and Privacy

This feature delegates a user-selected file to a local editor. It must not turn
Lens into a general local-file server or a browser-triggered process launcher.

- Lens emits VS Code URLs only for validated relative targets.
- Lens does not serve, embed, or copy the target file's contents.
- Lens does not accept an arbitrary path through an HTTP request.
- Hidden entries and every symbolic-link path component remain excluded.
- Canonicalization must prove that the target remains inside the fixed root.
- Selecting a link is the only trigger; Lens does not launch VS Code while
  rendering, refreshing, prefetching, or hovering.
- The emitted URL necessarily contains an absolute local path. That path is
  visible in the page markup and to the browser, but it is not sent to a remote
  service by Lens.
- The existing content security policy should remain restrictive. The
  implementation must verify that supported browsers can navigate an explicit
  link to the `vscode:` scheme without allowing script-created editor launches.

Markdown can already contain an authored external `vscode:` destination.
Blocking or warning on such authored URLs is a broader external-link policy
decision and is not required here. The security claim of this proposal applies
only to URLs that Lens generates from relative repository links.

## Session-Root Interaction

This proposal does not broaden the document root. A source file is eligible
only when it is inside the root selected for the current session.

For example, `lens docs` cannot authorize a link from `docs/design.md` to
`../src/lib.rs`, because `src/lib.rs` is outside the explicit `docs` root. The
user can run `lens` or `lens <repository-root>` to make both paths part of the
same fixed scope.

The
[repository-scoped target-session decision](../decisions/adr-019-repository-scoped-target-sessions.md)
lets a directly opened document inside a Git repository use the repository
root. This feature can be implemented independently and must follow the active
session's selected root.

## Compatibility

The command-line interface and Lens HTTP routes do not change. Existing
document links continue to open in Lens, and external links retain their
authored destinations.

The observable change applies to relative links that currently lead to a
regular non-document file inside the session root: selecting one will launch
VS Code instead of reaching Lens's guidance page. Repositories that intend a
relative file link to be handled by the browser rather than an editor will need
to use an explicit external URL.

VS Code becomes an optional integration, not a requirement for reading
documents. A missing VS Code installation must not prevent Lens from starting
or rendering the page. VS Code Insiders uses a different URL scheme and is not
selected automatically.

## Implementation Approach

An implementation should proceed in focused slices:

1. Retain the canonical document root in the target and viewer session state.
2. Extract shared relative-path normalization from Markdown-only link
   rewriting without changing current behavior.
3. Add a source-target resolver that enforces regular-file, hidden-path,
   symbolic-link, and canonical-root rules.
4. Add platform-aware VS Code URL construction with percent encoding.
5. Render qualifying links with an accessible indication that they open in VS
   Code.
6. Add unit and browser evidence, then update requirements, architecture
   decisions, user guidance, risks, and release notes.

The accessible indication may be visible text or an icon with equivalent
screen-reader text, but it must not rely on color alone. It should identify the
external-editor handoff without replacing the link's authored label.

If link-resolution responsibilities make `markdown.rs` contain independent
filesystem authorization, URL encoding, and Markdown rendering concerns, the
implementation should place source-target resolution in a cohesive module and
keep the Markdown renderer focused on event transformation.

## Rejected Alternatives

### Add a loopback `/open-source?path=...` route

A route that accepts a filesystem path would make browser input participate in
local path resolution and process launch. It would require request
authentication, method and origin protection, replay behavior, and careful
command construction. Direct validated VS Code URLs avoid that new authority.

### Spawn the `code` command from Lens

The command name, installation path, and availability vary by platform and
installation. A browser request would also need to trigger a local process.
Using VS Code's registered URL handler keeps application launch with the
browser and operating system.

### Render source files in Lens

Browser source viewing would require content routes, media-type and encoding
rules, syntax presentation, large-file limits, refresh behavior, and a broader
authorization model. The requested workflow is an editor handoff, so those
responsibilities are unnecessary.

### Rewrite every relative non-document link

Without validation, VS Code may create a missing file, open a path outside the
session root, or follow a symbolic link. Lens should emit only links whose
existing target it has safely classified.

### Maintain a programming-language extension allowlist

An allowlist would omit extensionless scripts, manifests, build files, and new
languages while implying that listed extensions make a path safe. Filesystem
location and type establish authorization; the editor determines how to
present the file.

### Make the editor configurable in the first iteration

Configurable schemes or executable commands add command-line, validation, and
support surface before the VS Code workflow is proven. A later proposal can
generalize the editor integration if users need it.

## Acceptance Criteria

- Selecting a relative link to an existing visible regular file inside the
  session root asks the operating system to open that file in VS Code.
- The generated URL uses the target's canonical absolute path and correctly
  encodes supported Linux, macOS, and Windows paths.
- A discovered Markdown or PlantUML target continues to open through its Lens
  document route.
- A missing target, directory, hidden path, symbolic link, absolute path, or
  out-of-root target never receives a Lens-generated `vscode:` URL.
- No Lens route accepts a source-file path or serves source-file contents.
- Rendering, refresh polling, and pointer hover never launch VS Code.
- A missing or unregistered VS Code installation does not prevent Lens from
  rendering or continuing to serve the document.
- Source links have a visible and screen-reader-accessible indication that
  selecting them opens VS Code.
- Existing external URLs and same-document fragments preserve their browser
  behavior.
- User documentation explains the fixed-root requirement, optional VS Code
  dependency, browser confirmation behavior, and unsupported VS Code Insiders
  scheme.

## Verification

Automated coverage should include behavior-named scenarios such as:

- `source_link_inside_root_then_emits_vscode_file_url`;
- `source_link_with_spaces_then_percent_encodes_canonical_path`;
- `known_document_link_then_keeps_lens_document_route`;
- `source_link_outside_root_then_omits_vscode_url`;
- `source_link_through_symlink_then_omits_vscode_url`;
- `source_link_beneath_hidden_directory_then_omits_vscode_url`;
- `missing_source_link_then_omits_vscode_url`; and
- `source_link_then_does_not_add_source_content_route`.

Each test should use explicit setup, one primary action, and verification
sections. Unit tests should cover path validation and platform URL
serialization separately. Browser tests should inspect the rendered link
destination and accessible indication without requiring the test machine to
have VS Code or actually launching an external application.

### Manual end-to-end test

- **Setup:** Create a disposable repository with `docs/design.md`,
  `src/example.rs`, a file whose name contains a space, a hidden file, a
  symbolic link, and a file outside the repository. Add links from the design
  document to each target and to another Markdown document. Install VS Code
  with its `vscode:` URL handler registered.
- **Actions:** Run Lens at the disposable repository root. Select the regular
  source links, accept any browser confirmation, and then select the document,
  hidden, symbolic-link, missing, and out-of-root links.
- **Expected result:** VS Code opens each qualifying regular file at its
  canonical path, including the filename containing a space. The Markdown
  document opens inside Lens. No disallowed target receives a generated VS Code
  destination or exposes source through Lens, and the Lens page remains usable
  after every selection.

## Analysis and Design Trace

- Deferred user goal: `UC-06` in
  [`FEAT-01`](../features/markdown-viewing/use-cases.md)
- Current document authorization:
  [`ADR-003`](../decisions/adr-003-document-root-discovery.md)
- Deferred V1 scope:
  [`ADR-004`](../decisions/adr-004-v1-release-scope.md)
- Filesystem exposure risk: `R-03` in
  [`docs/risk-list.md`](../risk-list.md)
- Future elaboration: define the editor-handoff system event, its
  source-target resolution contract, and a successor decision that narrows the
  deferred source-code goal to validated VS Code handoff.

## Out of Scope

- Displaying, searching, editing, or syntax-highlighting source files in Lens.
- Opening directories or VS Code workspaces.
- Opening a source file at a line, column, range, or symbol.
- Configuring another editor or the `vscode-insiders:` scheme.
- Detecting, installing, or starting VS Code.
- Broadening the fixed document root or changing direct-file root selection.
- Serving source-file bytes through Lens.
- Changing the policy for authored external application URLs.
