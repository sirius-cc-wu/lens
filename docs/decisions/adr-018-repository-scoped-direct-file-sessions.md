---
type: "Architecture Decision"
title: "ADR-018: Scope Direct-File Sessions to the Nearest Repository"
description: "Selects the nearest recognized Git repository as the fixed document root for a directly opened supported file."
id: "ADR-018"
status: "superseded"
date: "2026-07-24"
tags: [architecture, decision, navigation, security]
---

# ADR-018: Scope Direct-File Sessions to the Nearest Repository

Status: superseded by
[ADR-019](adr-019-repository-scoped-target-sessions.md)

Date: 2026-07-24

Partially supersedes
[ADR-003](adr-003-document-root-discovery.md) for direct-file root selection.

ADR-019 later applies repository-root selection to directory and
current-directory targets and replaces the implicit narrow-directory behavior
with an explicit scope option. The decision below remains the historical D1
direct-file decision.

## Context

ADR-003 makes the canonical parent of a directly opened Markdown or PlantUML
file its fixed document root. Repository documents commonly link across
feature, iteration, and decision directories, so a valid repository-relative
link can lie outside that parent and remain unavailable to the session.

Lens must broaden this ordinary direct-file workflow without letting document
contents, browser paths, symbolic links, or a process working directory choose
an unbounded filesystem scope.

## Decision

For a supported direct file, Lens walks canonical ancestors beginning at the
file's parent. The nearest ancestor containing either a non-symbolic-link
`.git` directory or a regular `.git` file becomes the document root. This
supports ordinary checkouts, linked checkouts (Git worktrees), and nested
repositories managed by Git (submodules) without invoking Git or reading
repository configuration.

A `.git` symbolic link does not establish a repository root. If no supported
marker exists, the direct file's canonical parent remains its document root.
The direct file remains the initial document.

Directory targets and the current-directory target retain their canonical,
explicit roots. All target kinds continue to discover one fixed set of visible,
non-symbolic-link `.md`, `.markdown`, and `.puml` files before the session
starts. A direct file below a hidden entry relative to the selected repository
root is rejected because it cannot belong to that discovered set.

Browser routes continue to serve only identifiers from the discovered set.
Links cannot add a document or cause a filesystem-path lookup.

## Consequences

- A directly opened repository document can link to supported documents in
  sibling repository directories.
- The nearest marker keeps nested repositories and submodules independent from
  an enclosing repository.
- A direct-file session can read more local supported documents during startup
  and refresh than the former parent-scoped session.
- Passing a directory remains the explicit way to request a narrower session.
- Hidden entries, symbolic links, `.git` contents, unsupported files, and paths
  outside the selected root remain unavailable.
- Repository recognition has no dependency on a `git` executable or marker
  contents.

## Trace

- Use cases: [`FEAT-01`, `UC-02` through `UC-04`](../features/markdown-viewing/use-cases.md)
- System sequence:
  [`SSD-02`](../features/markdown-viewing/ssd-02-open-document-root.md)
- Contract: [`OC-02`](../features/markdown-viewing/oc-02-open-document-root.md)
- Risks: [`R-03` and `R-09`](../risk-list.md)
- Design iteration:
  [`D1`](../iterations/d1-repository-scoped-direct-file-design.md)
- Construction iteration:
  [`D2`](../iterations/d2-repository-scoped-direct-file-sessions.md)
