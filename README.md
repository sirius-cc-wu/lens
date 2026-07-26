# Lens

Lens is a Linux command-line viewer for repository Markdown and PlantUML
diagrams. It starts a loopback-only browser session and does not depend on
Obsidian.

## Requirements

- A browser and the platform launcher: `xdg-open` on Linux, `open` on macOS,
  or `cmd /C start` on Windows.
- Optional: Visual Studio Code with its stable `vscode:` URL handler registered
  to open source-file links from rendered documents.
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
lens --scope target docs/features/markdown-viewing
LENS_PLANTUML_SERVER=http://127.0.0.1:8080/plantuml lens docs
```

Lens is a focused human-review surface. A coding agent can locate a relevant
document with repository context and start Lens with that known path:

```bash
lens docs/features/markdown-viewing/use-cases.md
```

A user working directly in a POSIX shell can compose Lens with a preferred file
finder:

```bash
lens "$(fd --type file --full-path '<file-pattern>' .)"
```

The shell replaces the `fd` expression with its output before Lens starts
(shell command substitution). The command must produce exactly one path because
Lens accepts one optional positional target. `fd` is only an example; Lens does
not install, invoke, or require it, and the file finder's root, matching, and
ignore behavior remain user-managed.

By default, a current-directory, directory, Markdown, or `.puml` target inside
a Git repository uses the nearest enclosing repository as its document root. A
directory or current-directory target outside a recognized repository remains
its own root; a supported file uses its canonical parent. Lens recognizes
ordinary repositories, worktrees, and submodules from a non-symbolic-link
`.git` directory or regular `.git` file without running Git.

A directly named file remains the initial document. A selected directory
initially prefers its own root `README`, then its `docs/index`, then its first
discovered document. When that repository-scoped directory contains no
supported document, Lens falls back to the repository's normal initial
selection.

Lens discovers `.md`, `.markdown`, and `.puml` files under the document root.
It excludes hidden entries and symbolic links. Relative Markdown links resolve
to discovered Markdown and PlantUML documents inside Lens. A relative link to
another existing visible regular file inside the same fixed root opens its
canonical local path in Visual Studio Code. All other local paths retain Lens's
guidance behavior without exposing file contents. Authored links and browser
history support review of meaningful document relationships. To review an
unlinked document, start Lens with that document as the target. A standalone
`.puml` file renders as one diagram when selected directly or requested through
its known document URL during the active session.

A repository-scoped session reads supported visible documents throughout that
repository during discovery and refresh. The fixed discovered set authorizes
known document routes and relative Markdown links even though Lens does not
display it as a catalog. Use `--scope target` when the viewing session should
remain limited to the selected directory, current directory, or selected
file's parent.

### Hidden directories

Lens does not scan hidden directories when a repository is the document root.
To view documents beneath a hidden parent directory, open a visible nested
directory with target scope:

```bash
lens --scope target .hidden/docs
```

## VS Code Source Links

A qualifying local-file link includes the visible text **(opens in VS Code)**.
Selecting it gives the browser a percent-encoded `vscode://file/` URL for the
file's validated canonical path. The browser may ask for confirmation before
handing that URL to the operating system.

Lens generates an editor URL only for a relative link whose current target is a
readable, visible, non-symbolic regular file inside the session's fixed
document root. It never generates one for a missing file, directory, hidden
path, symbolic link, absolute path, or target outside that root. Known Markdown
and PlantUML documents continue to open in Lens.

The root rule follows the selected session scope. For example, `lens docs`
inside a repository normally uses the repository root, so a link from
`docs/design.md` to `../src/lib.rs` can open in VS Code. With
`lens --scope target docs`, the same source file is outside the fixed root and
does not receive an editor URL.

VS Code is optional: if it is missing or its URL handler is not registered,
Lens still starts and the document remains usable while the browser reports
that it cannot open the link. Lens supports the stable `vscode:` scheme only;
it does not automatically choose the separate `vscode-insiders:` scheme. Lens
does not serve source-file contents, add a source-file browser route, or start a
`code` process.

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

Lens remains a documentation viewer. It does not render source-code files,
edit documents, or render Mermaid. Qualifying repository file links may hand
their validated local path to VS Code as described above.

## License

Lens is licensed under the [MIT License](LICENSE).

## Development

```bash
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
```
