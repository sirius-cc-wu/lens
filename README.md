# Lens

Lens is a Linux command-line viewer for repository Markdown and PlantUML
diagrams. It starts a loopback-only browser session and does not depend on
Obsidian.

## Requirements

- A browser and the platform launcher: `xdg-open` on Linux, `open` on macOS,
  or `cmd /C start` on Windows.
- Rust 1.75 or newer to build from source.
- Network access to `https://www.plantuml.com/plantuml` when using the default
  public PlantUML renderer.
- An installed `plantuml` command when using `--renderer local`.

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
lens --renderer local docs
lens --renderer disabled docs
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

### Agent skills

Lens does not scan the hidden `.agents` directory when the repository is the
document root. To view project agent skills, open the skills directory directly:

```bash
lens .agents/skills
```

## PlantUML

Lens uses the public PlantUML server by default. A failed diagram request leaves
the source visible in the document. Select `--renderer local` to run the
installed `plantuml` command on the current machine; Lens passes the diagram
source over that command's standard input and does not send it to a renderer
service. Select `--renderer disabled` to display PlantUML source without
requesting a rendered diagram for the viewing session.

Every document page identifies the selected renderer. When an individual
diagram fails, use its **Retry diagram rendering** button after the renderer is
available again. Use **Disable diagram rendering for this session** to stop
further renderer requests while preserving each diagram's source.

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
