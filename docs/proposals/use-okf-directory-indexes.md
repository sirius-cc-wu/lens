---
type: "Improvement Proposal"
title: "Use OKF Directory Indexes as Directory Entry Points"
description: "Gives every directory one overview behavior by opening its OKF index.md or synthesizing an equivalent listing when the file is absent."
id: "PROP-USE-OKF-DIRECTORY-INDEXES"
status: "proposed"
tags: [proposal, navigation, okf, markdown]
---

# Use OKF Directory Indexes as Directory Entry Points

Status: proposed

## Summary

Give every directory one entry behavior: display an overview of that
directory. When the directory contains the exact reserved file `index.md`,
Lens should display it. When `index.md` is absent, Lens should synthesize an
equivalent overview from the directory's immediate visible documents and
subdirectories.

This behavior follows the directory convention in version 0.2 of the
[Open Knowledge Format specification
(OKF)](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md).
OKF is a convention for organizing knowledge as Markdown documents with YAML
frontmatter. It reserves `index.md` as a directory listing that lets a reader
see a smaller summary before opening individual documents, a technique called
progressive disclosure.

An explicitly selected document should remain the initial document. Directory
selection should no longer choose a root `README`, a nested `docs/index`, or an
arbitrary first document based on lexical path order.

This proposal adopts OKF directory-entry semantics. It does not by itself claim
that Lens validates or fully implements every part of OKF.

## Motivation

Lens currently gives a directory several possible entry documents. It prefers
the selected directory's root `README.md` or `README.markdown`, then
`docs/index.md` or `docs/index.markdown`, and finally the first discovered
document in lexical path order.

That behavior is useful for conventional software repositories, but it is not
the hierarchy defined by OKF:

- `index.md` has a reserved meaning in every directory, including the bundle
  root.
- `README.md` is not a reserved directory overview.
- `docs/index.md` describes the `docs/` directory, not its parent directory.
- The first concept in lexical order has no special meaning.

Lexical selection can also change when an unrelated document is added or
renamed. A directory may unexpectedly open a different concept even though the
directory's intended introduction did not change.

OKF makes `index.md` optional. A missing index does not make a bundle invalid,
and the specification permits a consumer to synthesize one. Requiring every
producer to write `index.md` would therefore be stricter than OKF. Synthesizing
the overview preserves one predictable directory behavior without rejecting a
valid bundle or selecting an arbitrary concept.

The convention also improves authored navigation. An OKF index may link to a
subdirectory using a destination such as `tables/`. Lens should interpret that
destination as another directory overview rather than unavailable document
content.

## Goals

- Give CLI directory targets and authored directory links the same predictable
  behavior.
- Open the exact `index.md` belonging to the selected directory when it exists.
- Present a generated directory overview when the file is absent.
- Keep explicitly selected Markdown and PlantUML files as their own initial
  documents.
- Make a directory's entry behavior independent of lexical document order.
- Support OKF links to subdirectories without accepting arbitrary browser
  filesystem paths.
- Preserve the fixed viewing-session authorization boundary.
- Clearly distinguish this focused convention from complete OKF conformance.

## Proposed Behavior

| Situation | Lens behavior |
|---|---|
| The CLI target is a supported file | Display that file as the initial document. |
| The CLI target is a directory containing an immediate child named exactly `index.md` | Display that `index.md` as the initial directory overview. |
| The CLI target is a directory without an immediate `index.md` | Display a generated overview of the directory's immediate visible contents. |
| No CLI target is supplied | Treat the selected current-directory or repository root as a directory and apply the same overview behavior. |
| A known Markdown document links to `subdir/` | Display the overview for that authorized subdirectory. |
| A known Markdown document links directly to `subdir/index.md` | Display the authored index document through its normal document route. |
| A directory contains `README.md` but no `index.md` | Include the README in the generated overview as an ordinary document; do not select it implicitly. |
| A directory contains `docs/index.md` but no root `index.md` | Include `docs/` in the generated overview; do not use its index as the parent directory's overview. |
| A directory contains `index.markdown`, `INDEX.md`, or another spelling | Treat it as an ordinary supported document rather than the OKF-reserved index. |
| A directory has no immediate supported documents or qualifying subdirectories | Display an empty-directory overview with actionable guidance. |
| An authored directory link resolves outside the session root or through a hidden or symbolic-link entry | Preserve unavailable-document guidance without reading the target. |

The generated overview should list only immediate children. Recursively
flattening every descendant into one page would defeat progressive disclosure
and make large bundles difficult to scan.

For each immediate document, the generated overview should use its title and
description from valid YAML frontmatter when available, falling back to its
filename when needed. For each immediate subdirectory that contains visible
supported content, the overview should provide a link to that subdirectory's
overview. Generated content is session-owned presentation and must not be
written back to the repository.

`log.md` remains a visible document but receives no directory-entry priority.
Any future OKF-specific treatment of update history is a separate behavior.

## User Scenario

Primary actor: Knowledge-bundle reader

Goal: Open a directory and first understand what it contains.

Preconditions:

- The user can run Lens.
- The selected directory is inside the authorized viewing-session root.

Main success scenario:

1. The user starts Lens with a directory target.
2. Lens establishes the viewing-session root and discovers its authorized
   content.
3. Lens resolves the target directory as an overview.
4. Lens finds the target directory's immediate `index.md`.
5. Lens displays that authored index.
6. The reader selects a `subdir/` link from the index.
7. Lens displays that subdirectory's authored or generated overview.

Extensions:

- 4a. If the directory has no immediate `index.md`, Lens generates an overview
  from its immediate visible contents.
- 4b. If the directory has no immediate visible contents, Lens displays an
  empty overview instead of falling back to a document outside the directory.
- 6a. If the subdirectory is outside the fixed session root, hidden, symbolic,
  missing, or not known to the session, Lens displays unavailable-document
  guidance.
- 6b. If the reader selects a direct document link, Lens displays that document
  rather than its containing directory.

## Directory and Bundle Roots

An OKF knowledge bundle is a self-contained hierarchy and may be stored as a
subdirectory of a larger Git repository. Its absolute Markdown links begin
with `/` and are interpreted relative to the bundle root.

Lens currently defaults to the nearest Git repository as the viewing-session
root, even when the CLI target is a nested directory. This proposal does not
silently infer an OKF bundle root because OKF makes both `index.md` and the
`okf_version` declaration optional; there is no reliable required marker.

Until Lens adopts an explicit bundle-root operation, a nested OKF bundle should
be opened with target scope:

```bash
lens --scope target path/to/bundle
```

The selected directory remains the initial-overview anchor even when repository
scope makes the authorized session root broader. An authored `subdir/` link is
resolved relative to its containing document, and the resulting directory must
remain inside the fixed session root.

Correct interpretation of OKF absolute bundle-relative links is a related
requirement. Full OKF support should define an explicit bundle root rather than
assuming that every Git repository is one bundle.

## Routing and Authorization

Generated overviews need stable Lens routes but not source files. The root
overview should use the session's root route. A nested directory overview
should use a normalized directory identifier ending in `/`, so an authored
link to `subdir/` and browser history identify the same resource.

Lens should construct the set of authorized directories during initial
discovery from the selected directory and the parents of discovered visible
documents. A browser request must resolve only a directory identifier already
present in that set. It must not turn a route into a new filesystem scan or
accept a browser-supplied filesystem path. An empty subdirectory that is not
the selected directory and contains no discovered document need not become an
authorized route.

An authored `index.md` remains an ordinary discovered source for reading,
rendering, and automatic refresh. A generated overview is derived from the
session's fixed document and directory sets. Adding a new file after startup
must not make it appear unless a future rescan operation explicitly broadens
the session set.

Directory-link normalization must preserve the existing protections against
path traversal, hidden entries, symbolic links, and destinations outside the
canonical session root.

## Compatibility and Trade-Offs

### Existing repository landing pages will change

A repository with `README.md` but no root `index.md` currently opens the
README. Under this proposal it opens a generated overview containing a link to
the README. Repositories that want authored landing content should add a root
`index.md`.

This is an intentional observable change. Preserving `README` as another
implicit priority would retain multiple directory-entry rules and make Lens's
behavior diverge from OKF.

### Empty selected directories no longer fall back to the repository

Repository scope currently permits an empty selected directory to fall back to
the repository's initial document. This proposal instead displays the selected
directory's empty overview. The target should continue to describe what the
user initially sees even when the authorization boundary is broader.

### Generated overviews are not repository documents

A generated overview has no canonical source text, modification time, or
automatic-refresh revision of its own. It should be visibly identified as a
Lens-generated directory overview. Editing or exporting it is outside Lens's
current read-only scope.

### Exact OKF spelling is narrower than current conventions

OKF reserves lowercase `index.md`, not case variants or the `.markdown`
extension. Lens may continue discovering those other files, but treating them
as directory indexes would be a Lens extension rather than OKF behavior.

## Implementation Approach

Implementation should proceed in focused slices:

1. Represent authorized directories and their immediate child relationships in
   the target model without changing existing file discovery.
2. Resolve a selected directory to its exact immediate `index.md`.
3. Add a generated overview representation for a directory without an index.
4. Route normalized trailing-slash directory identifiers only through the
   session's authorized directory set.
5. Rewrite qualifying authored directory links to those directory routes.
6. Replace the existing `README`, `docs/index`, lexical, and repository
   fallback selection rules.
7. Update use cases, the open-document-root operation contract, architecture
   decisions, README guidance, and release notes before implementation is
   considered complete.

The target module should remain responsible for filesystem discovery,
authorization, and initial selection. Generated Markdown or HTML presentation
should remain with rendering or page construction. If directory relationships
give the target module an independent set of types, helpers, dependencies, and
tests, extract that concern along the capability boundary rather than growing
one mixed implementation file.

## Rejected Alternatives

### Require `index.md`

OKF explicitly permits missing index files and permits consumers to synthesize
them. Rejecting a directory without `index.md` would be stricter than the
format and would make incremental bundle authoring unnecessarily fragile.

### Preserve `README` before `index.md`

This would make the generic repository convention override the file that OKF
reserves for the directory listing. Producers could not predict whether their
authored OKF index would be the entry point.

### Fall back to the first document in lexical order

Filename order does not express directory-entry intent. Adding or renaming an
unrelated concept could silently change the initial page.

### Detect OKF automatically

The bundle-root `okf_version` declaration and `index.md` itself are optional.
Frontmatter in an individual concept is also insufficient to prove the root of
a bundle. Automatic detection would therefore be heuristic and could switch
behavior unexpectedly.

### Add an OKF mode only for initial selection

A mode that changes only the initial document would suggest broader
compatibility while leaving directory links and bundle-relative paths
incorrect. This proposal adopts the useful directory convention directly and
keeps full conformance as a separately testable product decision.

### Write generated `index.md` files

Lens is a viewer and must not modify the selected repository. Generated
overviews should exist only in the viewing session.

## Acceptance Criteria

- An explicit supported file remains the initial document.
- A directory with an immediate exact `index.md` displays that file first.
- A directory without `index.md` displays a generated overview rather than a
  README, nested index, or lexical-first document.
- A generated overview lists immediate visible documents and qualifying
  subdirectories without flattening descendants.
- Generated entries use valid frontmatter title and description values when
  available and safe filename fallbacks otherwise.
- A relative authored link ending in `/` opens the authorized subdirectory
  overview.
- Hidden, symbolic-link, missing, and out-of-root directories never become
  authorized overview routes.
- Browser directory routes cannot initiate filesystem discovery or broaden the
  fixed session set.
- Generated overviews are never written to disk.
- An empty selected directory displays an empty overview and does not fall back
  outside the selection anchor.
- Documentation states that this behavior adopts one OKF convention without
  claiming full OKF conformance.

## Verification

Automated coverage should include behavior-named scenarios such as:

- `directory_with_index_then_opens_its_index`;
- `directory_without_index_then_opens_generated_overview`;
- `directory_with_readme_only_then_overview_links_readme`;
- `directory_with_nested_docs_index_then_does_not_open_nested_index`;
- `empty_selected_directory_then_opens_empty_overview`;
- `directory_overview_then_lists_only_immediate_children`;
- `directory_link_then_opens_authorized_subdirectory_overview`;
- `directory_link_outside_root_then_shows_unavailable_guidance`;
- `directory_link_through_symlink_then_shows_unavailable_guidance`; and
- `unknown_directory_route_then_does_not_scan_filesystem`.

Each test should use explicit `// Arrange`, `// Act`, and `// Assert` phases
unless the behavior requires interleaved actions and observations. Unit tests
should cover target selection, directory authorization, and overview
construction separately. Browser tests should cover a CLI directory target,
an authored `subdir/` link, browser history, and unavailable-directory
guidance.

### Manual end-to-end test

- **Setup:** Create a disposable target-scoped bundle containing root
  `index.md`, `README.md`, two concept documents, one indexed subdirectory, one
  subdirectory without an index, a hidden directory, and a symbolic link to a
  directory outside the bundle. Create a separate empty directory. Add
  authored links from the root index to each bundle directory case.
- **Actions:** Start Lens on the bundle directory. Follow the indexed,
  unindexed, hidden, and symbolic directory links. Use browser Back and Forward
  between the resulting pages. Remove the root `index.md`, restart Lens, and
  inspect the generated root overview. Finally, start Lens with target scope on
  the separate empty directory.
- **Expected result:** The authored root index opens first when present. Each
  authorized subdirectory opens its authored or generated overview, and
  disallowed links expose no filesystem content. Browser history retains the
  directory routes. After restart without the root index, Lens opens a
  generated root overview rather than the README or first concept. The
  separately selected empty directory explains that it has no visible
  contents.

## Analysis and Design Trace

- OKF bundle structure, reserved filenames, index files, conformance, and
  versioning: [Open Knowledge Format
  specification](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
- Current initial-selection decision:
  [`ADR-003`](../decisions/adr-003-document-root-discovery.md)
- Current repository-root and selection-anchor decision:
  [`ADR-019`](../decisions/adr-019-repository-scoped-target-sessions.md)
- Current directory-target use case:
  [`FEAT-01`](../features/markdown-viewing/use-cases.md)
- Current open-document-root contract:
  [`OC-02`](../features/markdown-viewing/oc-02-open-document-root.md)

## Out of Scope

- Claiming or validating complete OKF conformance.
- Requiring YAML frontmatter on every concept.
- Deriving trust tiers, freshness, lifecycle, provenance, or attestations.
- Generating or modifying repository `index.md` files.
- Automatically detecting the root of a nested OKF bundle.
- Changing repository-versus-target authorization scope.
- Defining complete OKF absolute bundle-relative link behavior.
- Adding editing, export, or bundle-authoring features.
