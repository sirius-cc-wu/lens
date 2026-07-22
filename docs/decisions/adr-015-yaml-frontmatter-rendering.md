---
type: "Architecture Decision"
title: "ADR-015: Render Leading YAML Metadata Safely"
description: "Defines safe structured rendering for leading YAML frontmatter and actionable handling of malformed metadata."
id: "ADR-015"
status: "accepted"
date: "2026-07-19"
tags: [architecture, decision, markdown]
---

# ADR-015: Render Leading YAML Metadata Safely

Status: accepted

Date: 2026-07-19

## Context

Authors commonly put a short YAML header at the very beginning of a Markdown
document to record fields such as a title, author, date, tags, or project-
specific values. This header is called frontmatter. Rendering it as ordinary
Markdown makes the delimiters and YAML syntax noisy, while hiding it loses
information. YAML values can also be nested or contain browser-sensitive text.

## Decision

Lens treats the first line `---` as a frontmatter opening delimiter only at the
beginning of a Markdown document. It accepts either `---` or `...` on its own
line as the closing delimiter. A successful YAML mapping renders before the
Markdown body as a compact semantic table with field names as row headers.
Scalar values are escaped text, sequences are marker-free comma-separated
lists, and nested mappings are definition lists. The same structure preserves
unknown fields without needing a Lens-specific schema.

An empty header produces an explicit empty-metadata section. If parsing fails,
or a completed header is not a mapping of fields, Lens shows an escaped,
actionable alert before still rendering the remaining Markdown body. A missing
closing delimiter also produces an alert and leaves the source available to the
Markdown renderer rather than discarding document content.

## Consequences

- Common metadata is readable without duplicating a schema or treating unknown
  values as an error.
- User-controlled keys and values cannot create document markup because every
  rendered YAML string is escaped.
- A malformed header is visible and fixable without concealing the body.
- Only leading, delimiter-bounded YAML receives this treatment; ordinary YAML
  snippets elsewhere in a document remain Markdown content.

## Trace

- Proposal: [YAML Frontmatter Rendering](../improvement-proposals.md#10-yaml-frontmatter-rendering)
- Security boundary: [`docs/supplementary-specification.md`](../supplementary-specification.md)
- Iteration: [`P10`](../iterations/p10-yaml-frontmatter-rendering.md)
