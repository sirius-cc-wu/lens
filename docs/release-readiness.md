---
type: "Verification Evidence"
title: "Lens V1 Release Readiness"
description: "Defines the automated, browser, installation, packaging, and manual checks required for a Lens V1 release."
status: "active"
tags: [release, verification]
---

# V1 Release Readiness

Lens V1 is ready for a Linux source release when every check below has current
evidence. This document is the release checklist; its commands are executable
acceptance checks, meaning they verify observable user behavior rather than only
internal implementation details.

## Automated Checks

```bash
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo package --allow-dirty
```

## Browser End-to-End Checks

Install the JavaScript test dependencies, then run the compiled-command browser
suite:

```bash
npm ci
npm run test:browser
```

Expected result: the suite builds Lens with Cargo, starts Cargo's reported
executable against a temporary documentation repository, uses the installed
Google Chrome channel in headless mode, and completes without contacting the
public PlantUML service. It verifies rendered Markdown, navigation to a
discovered document, repository-scoped navigation from file, directory, and
current-directory targets, explicit target-scoped guidance without source
disclosure, outside-repository guidance without source disclosure, a
persistently collapsible navigation pane, submitted and no-JavaScript paginated
identifier search, automatic refresh after a saved change, 404 guidance for an
undiscovered document, accessible VS Code destinations only for qualifying
source links, preserved document and rejected-link destinations, absent source
routes, and visible PlantUML success and failure behavior.

## Installation Check

On a clean Linux shell with Rust 1.75 or newer:

```bash
cargo install --path . --locked
lens --help
```

Expected result: `lens --help` describes an optional `TARGET` argument and the
`repository` and `target` scope values.

## Binary Archive Check

On a native supported-platform host with the selected Rust target installed,
build a fresh archive:

```bash
scripts/package-release.sh --target x86_64-unknown-linux-gnu --output /tmp/lens-release
cd /tmp/lens-release
sha256sum --check lens-*-x86_64-unknown-linux-gnu.tar.gz.sha256
tar -tzf lens-*-x86_64-unknown-linux-gnu.tar.gz
```

Expected result: checksum verification succeeds and the archive contains a
single target-named directory with `lens` (or `lens.exe` on Windows), `README.md`,
and `LICENSE`. The packaging command refuses to overwrite an existing archive
or checksum.

## Package Metadata

- `Cargo.toml` declares the MIT license and points to `LICENSE`.
- `Cargo.toml` identifies the public repository, homepage, and hosted
  documentation URLs used by release metadata.

## Tagged Release Automation

GitHub Actions runs the same formatting, Rust test, Clippy, package, and
browser checks for pull requests and pushes to `main`. A pushed tag named
`v<package-version>` starts the release workflow. It rejects a tag whose
version does not match `Cargo.toml`, builds native archives on Linux, Intel
macOS, and Windows runners, and uploads all archives and checksums to the
GitHub Release only after every build succeeds.

The release workflow is not a substitute for this checklist: a release manager
still needs to assess native desktop behavior and verify a downloaded archive
on each supported platform.

## Target Checks

From a repository containing a nested feature document, an iteration document,
and a file outside the repository:

```bash
lens
lens docs
lens docs/features/guide.md
lens --scope target docs/features
```

Expected results:

- Lens prints a loopback URL and opens it with `xdg-open`, or prints the URL for
  manual opening if browser launch fails.
- File, directory, and current-directory targets recognize the nearest
  non-symbolic-link `.git` directory or regular `.git` file as their default
  root.
- A direct-file session opens the selected file first. Directory and
  current-directory sessions prefer documents below the selected directory
  before repository-level fallback.
- The default `docs/features` session follows a known-document link to the
  iteration document outside that directory.
- Passing `--scope target docs/features` keeps the explicit narrower scope and
  returns guidance for that cross-directory link.
- A link to the file outside the repository shows the Lens guidance page and
  does not disclose its source.

## VS Code Source-Link Checks

From a disposable repository containing `docs/design.md`, `src/example.rs`, a
source filename with a space, a hidden file, a symbolic link, a directory, a
missing target, a Markdown document, and an outside file:

```bash
cargo build --locked
lens <disposable-repository>
```

Expected results:

- Qualifying regular-file links visibly state **(opens in VS Code)**. Selecting
  them may show browser confirmation and then opens the canonical file in the
  stable VS Code installation, including the filename containing a space.
- The Markdown document opens through Lens. Hidden, symbolic, directory,
  missing, absolute, and out-of-root targets receive no generated `vscode:`
  destination and expose no source through Lens.
- Hovering a source link and refreshing a changed Markdown document do not open
  an editor. The Lens page remains usable after every selection.
- If the `vscode:` handler is unavailable, the browser reports that failure
  without preventing Lens from rendering or continuing to serve the document.
- VS Code Insiders is not selected automatically because it uses the distinct
  `vscode-insiders:` scheme.

## Rendering Checks

Open a document containing a valid PlantUML block and one with invalid PlantUML.
Repeat with `LENS_PLANTUML_SERVER` pointing to a controlled server.

Expected results:

- The valid diagram appears as SVG.
- The failed diagram keeps its source visible with an error.
- The remainder of the document remains readable.
- Every request in the configured session reaches only the controlled server.
- The page exposes retry but no rendering-disable control, and
  `/renderer/disable` returns not found.
- `lens --help` omits `--renderer`; passing it reports an unknown argument
  before a viewing session starts.

## Supported Platforms

- Linux, macOS, and Windows launch their default browser through `xdg-open`,
  `open`, and `cmd /C start` respectively. A launch failure still prints the
  loopback URL for manual opening.
- Documentation-only HTTP surface: Lens does not serve source-code contents;
  qualifying local links may hand a validated path to the optional stable VS
  Code URL handler.
- The public PlantUML server is the default, and `LENS_PLANTUML_SERVER` fixes a
  replacement server for the full session. Local-command and disabled rendering
  modes are not supported.
