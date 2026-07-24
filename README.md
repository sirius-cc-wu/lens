# Lens

Lens is a Linux command-line viewer for repository Markdown and PlantUML
diagrams. It starts a loopback-only browser session and does not depend on
Obsidian.

## Requirements

- A browser and the platform launcher: `xdg-open` on Linux, `open` on macOS,
  or `cmd /C start` on Windows.
- Rust 1.75 or newer to build from source.
- Network access to the selected PlantUML server. Lens uses
  `https://www.plantuml.com/plantuml` by default.

## Install

From a Lens checkout:

```bash
cargo install --path . --locked
```

## Release Archive

Build a release archive and its SHA-256 checksum for a supported Rust target:

```bash
scripts/package-release.sh --target x86_64-unknown-linux-gnu
```

The command writes `dist/lens-<version>-<target>.tar.gz` and a matching
`.sha256` file. The archive contains the `lens` binary, `README.md`, and
`LICENSE`; Windows archives contain `lens.exe`. Verify the checksum before
extracting it.

## Use

```bash
lens
lens docs
lens docs/features/markdown-viewing/oc-02-open-document-root.md
lens diagrams/architecture.puml
LENS_PLANTUML_SERVER=http://127.0.0.1:8080/plantuml lens docs
```

With no argument, Lens uses the current directory as the document root. A
directory argument uses that directory; a Markdown or `.puml` file argument
uses the file's canonical parent. Lens initially opens a root `README`, then
`docs/index`, then the first discovered document.

Lens discovers `.md`, `.markdown`, and `.puml` files under the document root.
It excludes hidden entries and symbolic links. Relative Markdown links resolve
only when their target is a discovered Markdown document; all other local paths
receive a Lens guidance page without filesystem access. A standalone `.puml`
file appears in the same navigation pane and renders as one diagram.

Use **Hide documents** to give the current document more room, and **Show
documents** to restore the pane. Lens remembers that choice while the same
browser tab views documents from the current loopback session; it does not
change which documents are available.

### Hidden directories

Lens does not scan hidden directories when the repository is the document root.
To view documents beneath a hidden parent directory, open a visible nested
directory directly:

```bash
lens .hidden/docs
```

## PlantUML

Lens uses one PlantUML server for each viewing session. It defaults to
`https://www.plantuml.com/plantuml`. Set `LENS_PLANTUML_SERVER` before starting
Lens to use a self-hosted or private server instead; Lens trims surrounding
whitespace and trailing `/` characters from that base URL.

Every document page identifies server-based PlantUML rendering without exposing
the configured URL. A failed diagram request leaves the source visible; use its
**Retry diagram rendering** button after the same server is available again.
Lens does not fall back to the public server when a configured server fails.
Lens also does not provide local-command or disabled rendering modes, so
configure a controlled server before opening source that must not be sent to
the public service.

## Markdown Metadata

A short YAML header at the very beginning of a Markdown document (frontmatter)
is shown as document metadata before the rendered body. Simple fields appear as
labels and values; lists and nested fields retain their structure. Lens removes
the opening and closing `---` or `...` delimiters from the body. If the YAML is
invalid, the page explains how to correct the header and still renders the
Markdown body.

## V1 Scope

Lens is a documentation viewer. It does not browse source-code files, edit
documents, or render Mermaid.

## License

Lens is licensed under the [MIT License](LICENSE).

## Development

```bash
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
```
