# Supplementary Specification

This specification captures quality constraints with architectural impact. It
does not prescribe the implementation architecture.

## Runtime and Portability

- Lens runs as a command-line application on the initially supported desktop
  platforms. Exact platform and distribution targets are an elaboration decision.
- The CLI starts a local-only browser session and should not expose the viewer
  to the local network by default.
- Failure to launch a browser must leave the local URL visible in the CLI.

## Content Handling and Privacy

- Lens reads targets without modifying repository files.
- The product must state when diagram source is sent to a renderer outside the
  machine.
- A local rendering path must be evaluated before an external renderer becomes
  the default.
- Lens must not collect telemetry or require an account for the initial release.

## Rendering and Resilience

- Common Markdown content remains readable when an individual PlantUML block
  fails to render.
- Rendered diagrams should preserve aspect ratio and fit within the document
  viewport without horizontal stretching.
- Target errors and rendering errors identify the affected path or diagram and
  provide a next action where possible.

## Security Boundaries

- Markdown and renderer output are untrusted content and require a defined
  sanitization policy before insertion into the browser view.
- The browser-facing server must restrict access to the resolved target scope;
  a request must not permit arbitrary filesystem reads.
- The first release does not accept write operations through the browser view.

## Performance

- A single ordinary repository Markdown document should become readable without
  perceptible unnecessary work. Quantitative limits will be set from `E1`
  measurements rather than guessed in inception.
