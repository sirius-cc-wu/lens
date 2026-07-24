---
type: "Improvement Proposal"
title: "Scope Direct-File Sessions to the Repository Root"
description: "Lets a directly opened document follow links beyond its parent directory while retaining one fixed, repository-bounded document set."
id: "PROP-REPOSITORY-SCOPED-DIRECT-FILES"
status: "implemented"
tags: [proposal, navigation, security, target]
---

# Scope Direct-File Sessions to the Repository Root

Status: implemented in D2

## Summary

When a user opens a Markdown or PlantUML file directly, Lens currently limits
the viewing session to supported documents beneath that file's parent
directory. This makes ordinary repository-relative links fail when they point
to a sibling documentation area, such as a design document linking from
`docs/features/` to an iteration record under `docs/iterations/`.

For a direct file inside a Git repository, Lens should instead use the nearest
enclosing repository root as the fixed filesystem scope for the session (the
authorization boundary). The selected file remains the initial document. A
direct file outside a Git repository retains its parent directory as the
document root.

Lens should identify a repository without invoking Git: the nearest canonical
ancestor containing a non-symbolic-link `.git` directory or a regular `.git`
file is the repository root. An ordinary checkout uses the directory form; a
linked checkout (Git worktree) or a nested repository managed by Git
(submodule) uses the file form.

## Motivation

Lens describes itself as a viewer for repository documentation. Repository
documents commonly link across capability-oriented directories, decision
records, iteration records, and indexes. The source path of such a link is
correct for the repository, but a direct-file session cannot follow it when
the target lies above the selected file's parent.

For example:

```text
docs/
  features/markdown-viewing/server-rendering-design.md
  iterations/c7-server-only-plantuml-rendering.md
```

Opening `server-rendering-design.md` directly currently authorizes only
`docs/features/markdown-viewing`. Its valid relative C7 link therefore remains
outside the discovered document set and reaches the Lens guidance page.

Making the repository root the direct-file boundary matches the user's
repository-level intent while retaining a fixed set of known documents. It
does not require an HTTP route to interpret a browser-provided filesystem path,
and it does not let Markdown content decide how far Lens may read.

## Proposed Behavior

| Situation | Lens behavior |
|---|---|
| A supported direct file has an ancestor containing a non-symbolic-link `.git` directory or a regular `.git` file | Use the nearest such ancestor as the document root and keep the direct file as the initial document. |
| The direct file is inside nested repositories or a submodule | Use the nearest repository root, not an enclosing repository. |
| No ancestor contains a supported `.git` marker | Preserve the current behavior: use the direct file's canonical parent as the document root. |
| The user supplies a directory target | Preserve the explicit directory as the document root, even when it is inside a larger repository. |
| The user supplies no target | Preserve the current directory as the document root. |
| A relative link resolves inside the discovered repository document set | Rewrite it to the corresponding Lens document route. |
| A link resolves outside the selected document root or to an undiscovered resource | Preserve the Lens-owned guidance response without reading the target through a browser route. |

The repository root and selected file are fully resolved paths (canonical
paths). Document discovery continues to include only `.md`, `.markdown`, and
`.puml` files and continues to exclude hidden entries and symbolic links. The
`.git` marker selects the root but its hidden contents are never added to the
document set.

## User Scenario

Primary actor: Developer or technical writer

Goal: Open one repository document directly and follow its links to other
repository documents without first choosing a broader directory target.

Main success scenario:

1. The user runs `lens <supported-file>` for a file inside a Git repository.
2. Lens selects the nearest enclosing repository as the fixed session scope.
3. Lens discovers supported visible documents in that repository and keeps the
   explicitly named file as the initial document.
4. Lens opens the viewing session.
5. The user follows a relative link whose target is outside the initial file's
   parent but inside the repository.
6. Lens displays the already discovered target document.

Extensions:

- 2a. If Lens finds no repository marker, it uses the selected file's parent
  and retains current direct-file behavior.
- 2b. If repositories are nested, Lens selects the nearest marker so the inner
  repository remains an independent boundary.
- 5a. If the link target is outside the selected root, hidden, symbolic, or
  unsupported, Lens returns its guidance page and does not expose the target
  source.

## Security and Privacy Boundary

The change deliberately broadens a direct-file session from one directory to
one repository. It must not make the scope unbounded or dependent on document
contents.

- Root selection happens once before discovery and browser startup.
- Browser requests continue to address only identifiers created during
  discovery; no route accepts a filesystem path.
- Markdown links never add documents to a running session.
- Canonical-path, hidden-entry, and symbolic-link exclusions remain active.
- A `.git` symbolic link does not establish a repository root.
- Lens does not run `git`, parse repository configuration, or read `.git`
  contents to select the root.
- Expanding the root causes Lens to read supported documents throughout the
  repository during discovery. This local scope increase must be documented;
  it does not send those documents or their diagrams to a PlantUML server
  merely because they were discovered.

This boundary refines the mitigation for `R-03`, which protects files outside
the requested repository, and may increase the large-repository discovery cost
tracked by `R-09`.

## Compatibility

Command syntax does not change. Direct-file sessions inside a repository will
have a larger document catalog, and identifiers will become relative to the
repository root instead of the selected file's parent. The selected file still
opens first.

Existing users who need the narrower behavior can pass the parent directory
explicitly:

```text
lens docs/features/markdown-viewing/server-rendering-design.md
    -> repository-scoped direct-file session

lens docs/features/markdown-viewing
    -> explicitly scoped directory session
```

Direct files outside a recognized repository, directory targets, and
current-directory targets retain their existing scope rules.

## Rejected Alternatives

### Use the current working directory

The current directory may be unrelated to the selected file or as broad as a
user's home directory. Using it implicitly could authorize an unexpectedly
large document set.

### Discover documents by following Markdown links

Following links during discovery would let document contents expand the
filesystem scope. It would also make authorization depend on parse order,
cycles, malformed documents, and supported link syntax. A fixed repository
root is easier to explain and verify.

### Add `--root` as the only solution

An explicit root would preserve user control but would require users to
understand and work around a surprising default for ordinary repository links.
A future explicit override may help non-Git document trees, but it is not
required for this repository-scoped behavior.

### Keep the direct file's parent as the root

This preserves the current security boundary but does not satisfy the user
goal. Users would still need to restart Lens with a broader directory whenever
a valid repository-relative link crosses the parent.

## Scope

An implementation of this proposal should:

- add one target-resolution operation that walks canonical ancestors from the
  selected file's parent and chooses the nearest supported `.git` marker;
- preserve the explicitly selected file as the initial document;
- keep directory and current-directory target behavior unchanged;
- retain known-document routing, hidden-entry exclusion, symbolic-link
  exclusion, and supported-extension filtering;
- update direct-file requirements, operation contracts, the document-root
  decision, risks, README guidance, and release-readiness checks; and
- add unit and browser evidence for repository-scoped navigation and
  out-of-repository rejection.

[ADR-017](../decisions/adr-017-repository-scoped-direct-file-sessions.md)
partially supersedes the direct-file-parent rule in
[ADR-003](../decisions/adr-003-document-root-discovery.md) without rewriting
the historical E2 iteration record.

## Acceptance Criteria

- A direct `.md`, `.markdown`, or `.puml` file inside a repository uses the
  nearest enclosing repository root as its document root.
- A non-symbolic-link `.git` directory and a regular `.git` file both establish
  the boundary without requiring a `git` executable.
- The selected direct file remains the initial document.
- A link from the selected file to a supported visible document outside its
  parent but inside the repository opens through a known Lens document route.
- Nested repositories and submodules use their nearest repository root.
- A direct file outside a recognized repository retains its parent as the
  document root.
- An explicit directory target retains that exact directory as its document
  root.
- Hidden documents, symbolic links, `.git` contents, and documents outside the
  selected root remain undiscovered and unavailable through browser routes.
- The behavior and its broader local discovery scope are documented for users.

## Verification

Automated coverage should include behavior-named scenarios such as:

- `direct_file_in_repository_then_discovers_repository_documents`;
- `direct_file_in_worktree_then_uses_worktree_root`;
- `direct_file_in_nested_repository_then_uses_nearest_repository_root`;
- `direct_file_without_repository_then_discovers_only_parent_documents`;
- `directory_target_inside_repository_then_keeps_explicit_scope`;
- `link_outside_repository_then_returns_guidance_without_source`; and
- `direct_file_link_outside_parent_then_displays_repository_document`.

Each test should use explicit setup, one primary action, and verification
sections. Target-resolution unit tests should cover both `.git` forms and the
non-repository fallback. The browser scenario should exercise the compiled
command against a nested initial document and follow a link into another
repository directory.

### Manual end-to-end test

- **Setup:** Create a disposable Git repository with
  `docs/features/guide.md`, `docs/iterations/evidence.md`, and a Markdown file
  outside the repository. Link the guide to both files.
- **Actions:** Run `lens docs/features/guide.md`. Follow the in-repository link,
  return to the guide, and follow the out-of-repository link. Restart Lens with
  `lens docs/features` and try the in-repository link again.
- **Expected result:** The direct-file session opens the guide first and
  displays the iteration evidence through a known document route. The
  out-of-repository link shows Lens guidance without its source. The explicit
  directory session remains limited to `docs/features` and does not broaden to
  the repository.

## Analysis and Design Trace

- User goals and navigation rules:
  [`FEAT-01`, `UC-02` through `UC-04`](../features/markdown-viewing/use-cases.md)
- System interaction:
  [`SSD-02`](../features/markdown-viewing/ssd-02-open-document-root.md)
- Document-root postconditions:
  [`OC-02`](../features/markdown-viewing/oc-02-open-document-root.md)
- Current architecture decision:
  [`ADR-003`](../decisions/adr-003-document-root-discovery.md)
- Security and scale risks: `R-03` and `R-09` in
  [`docs/risk-list.md`](../risk-list.md)
- D1 elaboration result:
  [`docs/iterations/d1-repository-scoped-direct-file-design.md`](../iterations/d1-repository-scoped-direct-file-design.md)
  resolves the successor decision.
- D2 construction result:
  [`docs/iterations/d2-repository-scoped-direct-file-sessions.md`](../iterations/d2-repository-scoped-direct-file-sessions.md)
  implements repository-root selection and verifies the target-loader and
  compiled-browser boundaries.

## Out of Scope

- Allowing browser routes to add files after session startup.
- Following links to arbitrary source-code or unsupported files.
- Automatically broadening non-Git document trees beyond the selected file's
  parent.
- Adding a general project-root discovery framework or support for non-Git
  repository markers.
- Changing directory-target, current-directory-target, or initial-document
  selection behavior.
