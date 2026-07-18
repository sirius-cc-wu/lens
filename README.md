# Lens

Lens is a Linux command-line viewer for repository Markdown and PlantUML
diagrams. It starts a loopback-only browser session and does not depend on
Obsidian.

## Requirements

- Linux with `xdg-open` and a browser available.
- Rust 1.75 or newer to build from source.
- Network access to `https://www.plantuml.com/plantuml` for PlantUML diagrams.

## Install

From a Lens checkout:

```bash
cargo install --path . --locked
```

## Use

```bash
lens
lens docs
lens docs/features/markdown-viewing/oc-02-open-document-root.md
```

With no argument, Lens uses the current directory as the document root. A
directory argument uses that directory; a Markdown file argument uses the
file's canonical parent. Lens initially opens a root `README`, then
`docs/index`, then the first discovered Markdown document.

Lens discovers `.md` and `.markdown` files under the document root. It excludes
hidden entries and symbolic links. Relative links resolve only when their target
is a discovered Markdown document; all other local paths receive a Lens guidance
page without filesystem access.

### Agent skills

Lens does not scan the hidden `.agents` directory when the repository is the
document root. To view project agent skills, open the skills directory directly:

```bash
lens .agents/skills
```

## PlantUML

Lens sends PlantUML block source to the public PlantUML server. A failed diagram
request leaves the source visible in the document. Do not use Lens V1 with
PlantUML source that must remain local.

## V1 Scope

V1 is a documentation viewer. It does not browse source-code files, edit
documents, render Mermaid, or support macOS or Windows releases.

## License

Lens is licensed under the [MIT License](LICENSE).

## Development

```bash
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
```
