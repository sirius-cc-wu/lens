---
type: "Architecture Decision"
title: "ADR-007: Poll Only Known Document Paths for Automatic Refresh"
description: "Constrains automatic refresh to the canonical document paths authorized when a viewing session starts."
id: "ADR-007"
status: "accepted"
date: "2026-07-18"
tags: [architecture, decision, refresh]
---

# ADR-007: Poll Only Known Document Paths for Automatic Refresh

Status: accepted

Date: 2026-07-18

## Context

Automatic refresh should show authors the current saved Markdown while they
edit. At session startup, ADR-003 has already authorized a fixed set of
canonical Markdown paths. Re-running discovery when any filesystem event occurs
would change the authorization boundary during a session, could expose a newly
added or hidden file, and would complicate partial-save behavior. Browser
polling of full document content would also make file observation depend on a
browser-provided path.

## Decision

Lens will run a session-local, bounded polling task over the canonical paths
that belong to the existing document set. It compares newly read contents with
each document's last successfully read source. When a known document changed,
Lens renders it using the existing Markdown and PlantUML rules, replaces that
document's cached representation, and increments its session-local revision.

Lens retains the prior representation and revision when a read fails. It does
not rescan the root, add or remove documents, or mutate repository files.

Each page includes its known document identifier and revision. Browser script
polls a revision-only endpoint for that current identifier. It reloads the page
when that revision changes; a changed background document does not reload an
unrelated page.

## Consequences

- Automatic refresh is platform-independent and adds no filesystem watcher
  dependency or server push connection.
- The interval bounds how quickly a saved change appears and causes reads of
  every known document while Lens is running. The interval and behavior should
  be measured before expanding the supported repository size.
- The session's file-access boundary stays fixed, including when files are
  added, hidden, deleted, renamed, or converted to symbolic links after
  startup.
- An incomplete editor save cannot replace readable browser content or trigger
  a browser reload; a later successful read advances the revision.
- Browsers without scripting retain ordinary navigation and manual reload, but
  cannot receive automatic reload.
