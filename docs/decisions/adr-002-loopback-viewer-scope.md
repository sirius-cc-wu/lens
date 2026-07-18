# ADR-002: Restrict a Viewing Session to Loopback and Fixed Resources

Status: accepted

Date: 2026-07-18

## Context

Lens opens a browser-facing viewing session for a selected Markdown file. The
session must render the document and its recognized diagrams without becoming a
general filesystem or URL proxy.

## Decision

Each viewing session binds an ephemeral port on `127.0.0.1`. It keeps the
canonical selected Markdown document and its recognized diagram identifiers in
memory.

The session exposes only the document root, fixed stylesheet and script assets,
and a diagram route addressed by a known diagram identifier. No route accepts a
filesystem path or arbitrary renderer URL.

## Consequences

- The browser session is not reachable from the local network by default.
- A failed automatic browser launch can safely report the loopback URL for
  manual opening.
- Browser requests cannot select other repository or host files.
- Directory browsing requires a new target-root authorization decision before
  it can be added.

## Trace

- Requirement: `UC-01` in
  [`docs/features/markdown-viewing/use-cases.md`](../features/markdown-viewing/use-cases.md)
- System operation: `open_markdown_target(target_path)` in
  [`SSD-01`](../features/markdown-viewing/ssd-01-open-markdown-target.md)
- Contract: [`OC-01`](../features/markdown-viewing/oc-01-open-markdown-target.md)
- Risk: `R-03` in [`docs/risk-list.md`](../risk-list.md)
