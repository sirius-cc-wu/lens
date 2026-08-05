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
- An ordinary `lens` invocation is a short-lived client. It automatically
  starts or reuses one background Lens process for the current operating-system
  user and does not require a separate server-start command.
- The client returns after Lens acknowledges that the target's browser view is
  available and the browser handoff has been attempted. It must not wait for
  the browser view to close. A startup, delivery, or acknowledgment failure
  must not be reported as success.
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
- A relative link to a qualifying visible regular file inside the fixed
  document root is rendered as a percent-encoded stable `vscode:` URL with a
  visible and accessible editor-handoff indication. Known Markdown and
  PlantUML documents retain Lens navigation.
- Lens must not collect telemetry or require an account for the initial release.

## Automated Browser Verification

- Browser end-to-end tests start the compiled `lens` command against a
  temporary documentation repository and use a local server with predefined
  responses (a controlled PlantUML server) for PlantUML evidence.
- The test child process sets `LENS_PLANTUML_SERVER` to that controlled server
  through the same supported session-configuration path available to users.
  When the normalized value is empty, Lens uses the public server defined by
  ADR-017.
- Browser checks inspect generated source-link destinations and accessible
  names without selecting the external scheme or requiring VS Code to be
  installed.

## Rendering and Resilience

- Common Markdown content remains readable when an individual PlantUML block
  fails to render.
- Every document identifies server-based PlantUML rendering without exposing
  the configured server URL. A failed diagram can be retried without accepting
  new source or changing its destination.
- Rendered diagrams should preserve aspect ratio and fit within the document
  viewport without horizontal stretching.
- Every successful document response uses one document-focused reading column
  at narrow and wide viewports. Generic document discovery and selection happen
  before Lens starts; authored links and browser history remain available
  within the fixed viewing session.
- Target errors and rendering errors identify the affected path or diagram and
  provide a next action where possible.
- A PlantUML request times out after 10 seconds. Lens rejects a server
  response larger than 2 MiB.

## Security Boundaries

- Lens escapes raw Markdown HTML. PlantUML SVG is not inserted into the
  document markup; it is served only as an image with a restrictive content
  security policy.
- The browser-facing server must restrict access to the resolved document
  root; a request must not permit arbitrary filesystem reads. Repository scope
  selects the nearest recognized repository by default, while
  `--scope target` preserves an explicit directory or file-parent boundary.
- A viewing session serves only its discovered document set. Symbolic links and
  hidden files and directories found during discovery are excluded. Document
  and revision routes resolve only identifiers from that fixed set, even though
  Lens does not expose the set as a searchable catalog.
- Reusing one background Lens process must not merge viewing-session authority.
  Each session retains its own canonical root, fixed document set, target
  scope, source-link resolver, and normalized PlantUML server selection.
- The command channel used to reach the background Lens process must be local
  to the current operating-system user. Loopback reachability alone must not
  authorize a process running as another operating-system user to submit a
  target.
- Lens-generated editor links reuse the canonical session root and require a
  readable, visible, non-symbolic regular file. No browser route accepts a
  source path or serves source contents, and Lens does not launch an editor
  process.
- The browser view does not accept repository writes, PlantUML server
  configuration, or a route that changes diagram-rendering state.
- Failure of a configured PlantUML server must not send the same source to the
  public default or another fallback server.

## Performance

- A single ordinary repository Markdown document should become readable without
perceptible unnecessary work. Quantitative limits will be set from `E1`
measurements rather than guessed in inception.
- Discovery and automatic refresh work should remain practical for ordinary
  repositories. Quantitative document-count, startup, memory, and idle-refresh
  limits require the repeatable measurements proposed in improvement 14.
- Reusing an available background Lens process should add no perceptible delay
  between invoking `lens` and returning control after acknowledgment. A
  construction iteration must establish a measurable threshold and a bounded
  failure timeout before treating this as verified behavior.
