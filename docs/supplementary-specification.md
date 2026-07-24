---
type: "Supplementary Specification"
title: "Lens Supplementary Specification"
description: "Defines the cross-cutting quality constraints for portability, safety, rendering, resilience, and verification."
status: "active"
tags: [requirements, quality]
---

# Supplementary Specification

This specification captures quality constraints with architectural impact. It
does not prescribe the implementation architecture.

## Runtime and Portability

- Lens supports Linux, macOS, and Windows. It launches the browser through
  `xdg-open`, `open`, or `cmd /C start` respectively.
- The supported source-install command is `cargo install --path . --locked`.
- Release artifacts use a target-specific archive name and contain the native
  binary name for the selected platform.
- The CLI starts a local-only browser session and should not expose the viewer
  to the local network by default.
- Failure to launch a browser must leave the local URL visible in the CLI.

## Content Handling

- Lens reads visible Markdown and `.puml` targets without modifying repository
  files.
- Lens sends PlantUML source to one server fixed when the viewing session
  starts. `LENS_PLANTUML_SERVER` selects a non-empty normalized base URL;
  otherwise Lens uses `https://www.plantuml.com/plantuml`.
- Lens requests a rendered diagram through its local viewer and exposes the
  returned SVG only as an image, never as inline document markup.
- Lens renders a YAML header at the beginning of a Markdown document
  (frontmatter) as escaped metadata before the Markdown body. It preserves
  nested and unknown values structurally, accepts `---` or `...` as the
  closing delimiter, and presents an actionable error without hiding the body
  when the header is malformed.
- Lens must not collect telemetry or require an account for the initial release.

## Automated Browser Verification

- Browser end-to-end tests start the compiled `lens` command against a
  temporary documentation repository and use a local server with predefined
  responses (a controlled renderer) for PlantUML evidence.
- The test child process sets `LENS_PLANTUML_SERVER` to that controlled server
  through the same supported session-configuration path available to users.
  When the normalized value is empty, Lens uses the public server defined by
  ADR-017.

## Rendering and Resilience

- Common Markdown content remains readable when an individual PlantUML block
  fails to render.
- Every document identifies server-based PlantUML rendering without exposing
  the configured server URL. A failed diagram can be retried without accepting
  new source or changing its destination.
- Rendered diagrams should preserve aspect ratio and fit within the document
  viewport without horizontal stretching.
- A user can hide and restore the document navigation pane with an accessible
  control. That presentation preference lasts only in the current browser tab
  and does not alter the viewing session's authorized documents or routes.
- Target errors and rendering errors identify the affected path or diagram and
  provide a next action where possible.
- A PlantUML request times out after 10 seconds. Lens rejects a renderer
  response larger than 2 MiB.

## Security Boundaries

- Lens escapes raw Markdown HTML. PlantUML SVG is not inserted into the
  document markup; it is served only as an image with a restrictive content
  security policy.
- The browser-facing server must restrict access to the resolved target scope;
  a request must not permit arbitrary filesystem reads.
- A viewing session serves only its discovered document set. Symbolic links and
  hidden files and directories found during discovery are excluded.
- The browser view does not accept repository writes, PlantUML server
  configuration, or a route that changes diagram-rendering state.
- Failure of a configured PlantUML server must not send the same source to the
  public default or another fallback server.

## Performance

- A single ordinary repository Markdown document should become readable without
perceptible unnecessary work. Quantitative limits will be set from `E1`
measurements rather than guessed in inception.
- Navigation search uses only the active session's authorized document
  identifiers. A submitted query is at most 256 UTF-8 bytes and each response
  contains at most 50 result links; typing alone does not issue a request.
