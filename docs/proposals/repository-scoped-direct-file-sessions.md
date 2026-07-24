---
type: "Improvement Proposal"
title: "Scope Repository Target Sessions to the Nearest Repository"
description: "Makes repository-internal links work consistently for file, directory, and current-directory targets while retaining an explicit narrow scope."
id: "PROP-REPOSITORY-SCOPED-DIRECT-FILES"
status: "selected"
tags: [proposal, navigation, security, target]
---

# Scope Repository Target Sessions to the Nearest Repository

Status: selected for D4; the direct-file slice was implemented in D2

## Summary

Lens initially broadened only directly opened Markdown and PlantUML files to a
repository root. A directory or current-directory target still limits the
viewing session to that exact directory. This makes ordinary
repository-relative links work or fail based only on how the same content was
opened. For example, `lens docs/iterations` cannot follow links to
`docs/features`, while opening one iteration file directly can.

For any target inside a Git repository, Lens should use the nearest enclosing
repository root as the default fixed filesystem scope for the session (the
authorization boundary). The selected file or directory should still control
what opens first. `--scope target` should preserve an explicit directory or
file-parent boundary for users who need a narrower session. Outside a
recognized repository, default behavior remains unchanged.

Lens should identify a repository without invoking Git: the nearest canonical
ancestor containing a non-symbolic-link `.git` directory or a regular `.git`
file is the repository root. An ordinary checkout uses the directory form; a
linked checkout (Git worktree) or a nested repository managed by Git
(submodule) uses the file form.

## Motivation

Lens describes itself as a viewer for repository documentation. Repository
documents commonly link across capability-oriented directories, decision
records, iteration records, and indexes. The source path of such a link is
correct for the repository, but an exact-directory session cannot follow it
when the target lies above the selected directory.

For example:

```text
docs/
  features/markdown-viewing/server-rendering-design.md
  iterations/c7-server-only-plantuml-rendering.md
```

Opening `docs/iterations` as a directory currently authorizes only that
directory. A valid link to the feature document therefore remains outside the
discovered document set and reaches the Lens guidance page.

Making the repository root the default boundary for every repository target
matches the user's repository-level intent while retaining a fixed set of
known documents. It does not require an HTTP route to interpret a
browser-provided filesystem path, and it does not let Markdown content decide
how far Lens may read.

## Proposed Behavior

| Situation | Lens behavior |
|---|---|
| A file, directory, or current-directory target has an ancestor containing a non-symbolic-link `.git` directory or a regular `.git` file | In the default repository scope, use the nearest such ancestor as the document root. |
| The target is inside nested repositories or a submodule | Use the nearest repository root, not an enclosing repository. |
| No ancestor contains a supported `.git` marker | Preserve the current behavior: use the selected directory or current directory, or the direct file's canonical parent. |
| The user selects `--scope target` | Skip repository recognition and use the selected directory or current directory, or the direct file's canonical parent. |
| The user supplies a file | Keep that file as the initial document. |
| The user supplies a directory or no target | Prefer a root `README`, `docs/index`, or first document below that selected directory; fall back to repository-level selection only when its subtree contains no supported document. |
| A relative link resolves inside the discovered repository document set | Rewrite it to the corresponding Lens document route. |
| A link resolves outside the selected document root or to an undiscovered resource | Preserve the Lens-owned guidance response without reading the target through a browser route. |

The repository root and selected target are fully resolved paths (canonical
paths). Document discovery continues to include only `.md`, `.markdown`, and
`.puml` files and continues to exclude hidden entries and symbolic links. The
`.git` marker selects the root but its hidden contents are never added to the
document set.

## User Scenario

Primary actor: Developer or technical writer

Goal: Open a repository documentation directory and follow its links to other
repository documents without restarting Lens at a broader directory.

Main success scenario:

1. The user runs `lens docs/iterations` inside a Git repository.
2. Lens selects the nearest enclosing repository as the fixed session scope.
3. Lens discovers supported visible documents in that repository and uses
   `docs/iterations` as the initial-selection anchor.
4. Lens opens the viewing session.
5. The user follows a relative link whose target is outside
   `docs/iterations` but inside the repository.
6. Lens displays the already discovered target document.

Extensions:

- 2a. If Lens finds no repository marker, it uses the selected directory or
  current directory, or the selected file's parent.
- 2b. If repositories are nested, Lens selects the nearest marker so the inner
  repository remains an independent boundary.
- 2c. If the user selects `--scope target`, Lens uses the target's former
  narrow boundary without repository recognition.
- 3a. If the selected directory contains no supported document, Lens falls
  back to the repository root's normal initial-document selection.
- 5a. If the link target is outside the selected root, hidden, symbolic, or
  unsupported, Lens returns its guidance page and does not expose the target
  source.

## Security and Privacy Boundary

The change deliberately broadens directory and current-directory sessions from
one directory to one repository, following the D2 direct-file change. It must
not make the scope unbounded or dependent on document contents.

- Root selection happens once before discovery and browser startup.
- Browser requests continue to address only identifiers created during
  discovery; no route accepts a filesystem path.
- Markdown links never add documents to a running session.
- Canonical-path, hidden-entry, and symbolic-link exclusions remain active.
- A `.git` symbolic link does not establish a repository root.
- `--scope target` retains an explicit narrow boundary when repository-wide
  local reading is not intended.
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

Directory and no-target sessions inside a repository will have a larger
document catalog, and identifiers will become relative to the repository root
instead of the selected directory. The selected file still opens first, and a
selected directory still controls initial-document selection when possible.

Existing users who need the narrower behavior can request target scope
explicitly:

```text
lens docs/features/markdown-viewing/server-rendering-design.md
    -> repository-scoped session

lens --scope target docs/features/markdown-viewing
    -> target-scoped directory session
```

Targets outside a recognized repository retain their existing roots in the
default mode.

## Rejected Alternatives

### Keep directory and current-directory targets exact

This retains existing compatibility, but the same repository link still works
or fails based only on target type. Users do not ordinarily treat a directory
argument as a security-boundary declaration.

### Discover documents by following Markdown links

Following links during discovery would let document contents expand the
filesystem scope. It would also make authorization depend on parse order,
cycles, malformed documents, and supported link syntax. A fixed repository
root is easier to explain and verify.

### Add `--scope repository` as an opt-in

An opt-in preserves compatibility but requires users to understand and work
around the surprising default for ordinary repository links. Repository scope
matches the product's repository-documentation purpose; the narrower behavior
is the one that should require explicit intent.

### Remove narrow target scope

One repository-wide mode is simpler, but users need a deliberate way to limit
local reads and refresh work. `--scope target` preserves that control without
overloading the target type.

## Scope

An implementation of this proposal should:

- apply nearest-repository recognition to file, directory, and
  current-directory targets by default;
- preserve the selected file or directory as the initial-selection anchor;
- add `--scope target` for the former exact-directory and file-parent roots;
- retain known-document routing, hidden-entry exclusion, symbolic-link
  exclusion, and supported-extension filtering;
- update requirements, operation contracts, document-root decisions, risks,
  README guidance, and release-readiness checks; and
- add unit, CLI, and browser evidence for repository-scoped directory
  navigation, target scope, and out-of-repository rejection.

[ADR-018](../decisions/adr-018-repository-scoped-target-sessions.md)
supersedes ADR-017 and the target-root selection rules in
[ADR-003](../decisions/adr-003-document-root-discovery.md) without rewriting
the historical E2, D1, or D2 iteration records.

## Acceptance Criteria

- A file, directory, or current-directory target inside a repository uses the
  nearest enclosing repository root by default.
- A non-symbolic-link `.git` directory and a regular `.git` file both establish
  the boundary without requiring a `git` executable.
- A selected file remains the initial document; a selected directory remains
  the initial-selection anchor.
- A link from a document below the selected directory to a supported visible
  document elsewhere in the repository opens through a known Lens route.
- Nested repositories and submodules use their nearest repository root.
- A target outside a recognized repository retains its former document root.
- `--scope target` retains the selected directory, current directory, or file
  parent as the exact document root.
- Hidden documents, symbolic links, `.git` contents, and documents outside the
  selected root remain undiscovered and unavailable through browser routes.
- The behavior and its broader local discovery scope are documented for users.

## Verification

Automated coverage should include behavior-named scenarios such as:

- `direct_file_in_repository_then_discovers_repository_documents`;
- `direct_file_in_worktree_then_uses_worktree_root`;
- `direct_file_in_nested_repository_then_uses_nearest_repository_root`;
- `direct_file_without_repository_then_discovers_only_parent_documents`;
- `directory_target_inside_repository_then_discovers_repository_documents`;
- `target_scoped_directory_inside_repository_then_discovers_only_target_documents`;
- `empty_selected_directory_then_uses_repository_initial_document`;
- `link_outside_repository_then_returns_guidance_without_source`; and
- `direct_file_link_outside_parent_then_displays_repository_document`.

Each test should use explicit setup, one primary action, and verification
sections. Target-resolution unit tests should cover both scope modes, both
`.git` forms, directory initial selection, and the non-repository fallback.
The browser scenario should exercise the compiled command against a directory
target and follow a link into another repository directory.

### Manual end-to-end test

- **Setup:** Create a disposable Git repository with
  `docs/features/guide.md`, `docs/iterations/evidence.md`, and a Markdown file
  outside the repository. Link the guide to both files.
- **Actions:** Run `lens docs/features`. Follow the in-repository link, return
  to the guide, and follow the out-of-repository link. Restart Lens with
  `lens --scope target docs/features` and try the in-repository link again.
- **Expected result:** The default directory session opens a feature document
  first and displays the iteration evidence through a known document route.
  The out-of-repository link shows Lens guidance without its source. The
  target-scoped session remains limited to `docs/features`.

## Analysis and Design Trace

- User goals and navigation rules:
  [`FEAT-01`, `UC-02` through `UC-04`](../features/markdown-viewing/use-cases.md)
- System interaction:
  [`SSD-02`](../features/markdown-viewing/ssd-02-open-document-root.md)
- Document-root postconditions:
  [`OC-02`](../features/markdown-viewing/oc-02-open-document-root.md)
- Current architecture decision:
  [`ADR-018`](../decisions/adr-018-repository-scoped-target-sessions.md)
- Security and scale risks: `R-03` and `R-09` in
  [`docs/risk-list.md`](../risk-list.md)
- D1 elaboration result:
  [`docs/iterations/d1-repository-scoped-direct-file-design.md`](../iterations/d1-repository-scoped-direct-file-design.md)
  resolves the successor decision.
- D2 construction result:
  [`docs/iterations/d2-repository-scoped-direct-file-sessions.md`](../iterations/d2-repository-scoped-direct-file-sessions.md)
  implements repository-root selection and verifies the target-loader and
  compiled-browser boundaries.
- D3 elaboration result:
  [`docs/iterations/d3-repository-scoped-target-design.md`](../iterations/d3-repository-scoped-target-design.md)
  accepts the unified default and explicit target scope for D4 construction.

## Out of Scope

- Allowing browser routes to add files after session startup.
- Following links to arbitrary source-code or unsupported files.
- Automatically broadening non-Git document trees beyond the selected file's
  parent.
- Adding a general project-root discovery framework or support for non-Git
  repository markers.
- Adding an arbitrary filesystem root override beyond repository and target
  scope.
