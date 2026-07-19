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
- Lens defaults to sending PlantUML block source over HTTPS to
  `https://www.plantuml.com/plantuml`, and also supports a local `plantuml`
  command or disabled diagram rendering for a viewing session.
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
- The test child process may set `LENS_PLANTUML_SERVER` to that controlled
  renderer. When the variable is absent or empty, Lens uses the public server
  defined by ADR-001.

## Rendering and Resilience

- Common Markdown content remains readable when an individual PlantUML block
  fails to render.
- Every document identifies the active diagram renderer. A failed diagram can
  be retried without accepting new source, and the user can disable rendering
  for the remaining viewing session.
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
- The browser view does not accept repository writes. Its only mutable route
  disables diagram rendering for the current in-memory viewing session.

## Performance

- A single ordinary repository Markdown document should become readable without
perceptible unnecessary work. Quantitative limits will be set from `E1`
measurements rather than guessed in inception.
- Navigation search uses only the active session's authorized document
  identifiers. A submitted query is at most 256 UTF-8 bytes and each response
  contains at most 50 result links; typing alone does not issue a request.
