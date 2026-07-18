# Supplementary Specification

This specification captures quality constraints with architectural impact. It
does not prescribe the implementation architecture.

## Runtime and Portability

- Lens V1 supports Linux and launches the browser through `xdg-open`.
- The supported source-install command is `cargo install --path . --locked`.
- macOS and Windows are not V1 release platforms.
- The CLI starts a local-only browser session and should not expose the viewer
  to the local network by default.
- Failure to launch a browser must leave the local URL visible in the CLI.

## Content Handling

- Lens reads targets without modifying repository files.
- Lens sends PlantUML block source over HTTPS to
  `https://www.plantuml.com/plantuml` for V1 rendering.
- V1 provides no local renderer or privacy-preserving alternative for PlantUML
  source.
- Lens requests a rendered diagram through its local viewer and exposes the
  returned SVG only as an image, never as inline document markup.
- Lens must not collect telemetry or require an account for the initial release.

## Rendering and Resilience

- Common Markdown content remains readable when an individual PlantUML block
  fails to render.
- Rendered diagrams should preserve aspect ratio and fit within the document
  viewport without horizontal stretching.
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
- The first release does not accept write operations through the browser view.

## Performance

- A single ordinary repository Markdown document should become readable without
  perceptible unnecessary work. Quantitative limits will be set from `E1`
  measurements rather than guessed in inception.
