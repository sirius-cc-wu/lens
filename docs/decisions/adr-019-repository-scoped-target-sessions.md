---
type: "Architecture Decision"
title: "ADR-019: Scope Repository Targets to the Nearest Repository"
description: "Uses the nearest recognized Git repository as the default document root for file, directory, and current-directory targets."
id: "ADR-019"
status: "accepted"
date: "2026-07-24"
tags: [architecture, decision, navigation, security]
---

# ADR-019: Scope Repository Targets to the Nearest Repository

Status: accepted

Date: 2026-07-24

Supersedes [ADR-018](adr-018-repository-scoped-direct-file-sessions.md) and the
target-root selection rules in
[ADR-003](adr-003-document-root-discovery.md).

## Context

ADR-018 broadened only a directly opened file to the nearest repository.
Directory and current-directory targets remained exact filesystem boundaries.
That distinction is not visible in the repository's Markdown links: running
`lens docs/iterations` leaves valid links to `docs/features` unavailable even
though opening one iteration file directly makes the same links work.

Users select a target to choose what Lens should open first. They do not
necessarily intend the target type to impose a different authorization
boundary. Lens still needs an explicit narrow mode for privacy, startup cost,
and non-repository document trees.

## Decision

Lens exposes a viewing-session boundary choice (scope) through
`--scope repository|target`, defaulting to `repository`.

In repository scope, Lens starts repository recognition at the canonical
directory target, current directory, or supported file's parent. The nearest
ancestor containing a non-symbolic-link `.git` directory or regular `.git`
file becomes the document root. If no supported marker exists, a directory or
current-directory target remains its own root and a file target uses its
parent.

In target scope, Lens does not broaden through repository recognition. A
directory or current-directory target is the document root, and a supported
file's parent is the document root. This mode preserves the pre-ADR-018
boundary explicitly.

The original target remains the initial-selection anchor:

- A supported file remains the initial document.
- A directory or current-directory target prefers its own root `README`, its
  own `docs/index`, and then its first supported document in lexical path
  order.
- If a repository-scoped directory contains no supported document, Lens falls
  back to the repository root's normal `README`, `docs/index`, and lexical
  selection order.

Repository recognition still selects the nearest supported marker without
invoking Git or reading marker contents. A `.git` symbolic link does not
establish a root. If repository broadening would place the selected target
below a hidden repository-relative entry, Lens rejects that target because
repository discovery cannot admit it. The user can select a visible nested
directory with `--scope target` when an exact narrow session is intended.

All sessions discover one fixed set of visible, non-symbolic-link `.md`,
`.markdown`, and `.puml` files before browser startup. Browser routes continue
to serve only identifiers from that set.

## Consequences

- File, directory, and no-target invocations inside one repository expose the
  same repository-relative document identifiers and valid internal links.
- A selected file still opens first. A selected directory still controls the
  initial document when it contains supported documents.
- Directory and no-target invocations inside repositories can read more local
  supported documents during discovery and refresh than before.
- Users who need the former narrow behavior must choose
  `--scope target`; target type no longer implies a security boundary.
- Non-repository targets retain their former roots in the default mode.
- Nested repositories and submodules remain independent because the nearest
  supported marker wins.
- Hidden entries, symbolic links, `.git` contents, unsupported files, and
  paths outside the selected root remain unavailable.

## Alternatives Considered

### Keep directories exact

This preserves compatibility but repeats the original navigation failure for
a common command such as `lens docs/iterations`. It also makes session scope
depend on a target-type distinction that users cannot infer from document
links.

### Remove narrow target scope

One repository-wide default is simpler, but users still need a deliberate way
to limit local reads and refresh work. An explicit option communicates that
intent better than overloading a directory argument.

### Discover linked documents on demand

Document-controlled discovery would make authorization depend on Markdown
contents and browser navigation. A fixed root and known-document routes remain
easier to explain and verify.

## Trace

- Use cases: [`FEAT-01`, `UC-02` through `UC-04`](../features/markdown-viewing/use-cases.md)
- System sequence:
  [`SSD-02`](../features/markdown-viewing/ssd-02-open-document-root.md)
- Contract: [`OC-02`](../features/markdown-viewing/oc-02-open-document-root.md)
- Risks: [`R-03` and `R-09`](../risk-list.md)
- Design iteration:
  [`D3`](../iterations/d3-repository-scoped-target-design.md)
- Construction iteration:
  [`D4`](../iterations/d4-repository-scoped-target-sessions.md)
