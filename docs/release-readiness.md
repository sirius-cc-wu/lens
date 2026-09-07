---
type: "Verification Evidence"
title: "Lens V1 Release Readiness"
description: "Defines the automated, browser, installation, packaging, and manual checks required for a Lens V1 release."
status: "active"
tags: [release, verification]
---

# V1 Release Readiness

Lens V1 is ready for a supported-platform release when every check below has
current evidence. This document is the release checklist; its commands are
executable acceptance checks, meaning they verify observable user behavior
rather than only internal implementation details.

## Automated Checks

```bash
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo package --allow-dirty
```

Pull requests and pushes to `main` run the locked Rust test, Clippy, and package
checks on native x86-64 Linux, macOS, and Windows runners. Formatting and the
compiled-browser suite run on Linux.

## Browser End-to-End Checks

Install the JavaScript test dependencies, then run the compiled-command browser
suite:

```bash
npm ci
npm run test:browser
```

Expected result: the suite builds Lens with Cargo, starts an explicit
background service and Cargo's reported ordinary executable against a
temporary documentation repository, waits for the ordinary command to exit,
then uses the installed Google Chrome channel in headless mode without
contacting the public PlantUML service. It verifies rendered Markdown,
navigation to a discovered document through an authored link, browser Back
history, repository-scoped navigation from file, directory, and
current-directory targets, explicit target-scoped guidance without source disclosure,
outside-repository guidance without source disclosure, narrow and wide
single-column layout without navigation controls or storage, inert former
`query` and `page` parameters, automatic refresh after a saved change, 404
guidance for an undiscovered document, accessible VS Code destinations only
for qualifying source links, preserved document and rejected-link
destinations, absent source routes, and visible direct-target,
known-document-route, success, and failure behavior for PlantUML.

## Background Service Checks

Use a disposable repository and a browser launcher that can be observed or
controlled. Run two ordinary commands from one terminal:

```bash
lens README.md
lens docs/release-readiness.md
```

Expected results:

- The first command starts the per-user service automatically, prints a ready
  loopback URL, attempts one browser handoff, and exits without waiting for the
  page to close.
- The second command reuses the process, creates a different loopback URL, and
  exits while both pages remain available.
- Saving a displayed document refreshes its page after the client has exited.
- A missing or unsupported target returns an actionable CLI error and performs
  no browser handoff. A missing browser launcher reports the manual ready URL
  and leaves that URL available.
- Concurrent first commands reach one endpoint owner and both receive isolated
  views. After forcibly stopping the service, its old URLs fail and the next
  command removes the verified stale endpoint and starts a replacement.
- A peer that does not satisfy the native per-user policy is rejected before a
  request frame is decoded.

The optimized C16 Linux reference fixture uses one Markdown document. Ten cold
starts measured 31-32 ms, and 95% of 30 reuse acknowledgments completed within
5 ms (the 95th percentile). A service that accepted a frame without responding
returned the configured acknowledgment error after 10,005 ms. These numbers
are comparison baselines, not cross-platform timing guarantees. Investigate a
reference-host regression when 95% of either cold-start or reuse samples no
longer complete within 250 ms; the hard portable bounds remain three seconds
for service startup and ten seconds for acknowledgment.

The same fixture measured 5,264 KiB resident memory (RSS) before a session and
12,804 KiB after 50 retained sessions. The complete increase was about
151 KiB per session; after first-use initialization it was about 117 KiB per
additional session. Fifty polling sessions consumed about 70 ms CPU time
(7 Linux process-accounting ticks) over five idle seconds versus no measurable
baseline work. Investigate growth above 256 KiB per additional one-document
session after the first or polling above 2% of one CPU at 50 such sessions.
Large-document-set measurements remain tracked separately by `R-09` and
improvement 14.

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

GitHub Actions runs native Rust test, Clippy, and package checks plus Linux
formatting and browser checks for pull requests and pushes to `main`. A pushed
tag named `v<package-version>` starts the release workflow. It rejects a tag
whose version does not match `Cargo.toml`, builds native archives on Linux,
Intel macOS, and Windows runners, and uploads all archives and checksums to the
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
  manual opening if browser launch fails. The command then exits while its view
  remains available through the background service.
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

## Focused Review Checks

Prepare a repository with a root `README`, an unlinked nested Markdown
document, two Markdown documents that link to each other, a standalone `.puml`
file, a hidden document, and a symbolic-link document.

Start Lens with paths selected by a coding agent or shell:

```bash
lens path/to/linked-document.md
lens path/to/unlinked-document.md
lens path/to/diagram.puml
lens --scope target path/to/linked-document.md
```

An optional POSIX-shell composition such as
`lens "$(fd --type file --full-path '<file-pattern>' .)"` is valid only when
the selector returns exactly one path. `fd` is not a Lens dependency.

Expected results:

- The selected document opens first in one reading column at narrow and wide
  viewports. No catalog, search form, pagination, current-result marker, pane
  control, collapsed attribute, or navigation browser-storage value appears.
- Authored Markdown links work within the discovered scope, browser Back
  returns to the prior document, and the unlinked document opens through a new
  direct invocation.
- The standalone `.puml` target and its known document URL render one diagram.
- A known document URL with former `query` and `page` parameters returns the
  same document response as the URL without them.
- Unknown, hidden, symbolic-link, traversal, and out-of-root identifiers return
  Lens-owned guidance without exposing source.
- Repository scope retains repository links; `--scope target` retains the
  narrower filesystem boundary. Saving the displayed document still refreshes
  the page, and diagram failure/retry remains per diagram.

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
- Linux and macOS use a private per-user Unix-domain socket and verify the peer
  user before decoding commands. Windows uses a user-SID-specific named pipe
  whose access policy permits only that user and LocalSystem and rejects remote
  clients. Native pull-request checks compile and execute each platform's
  implementation.
- Documentation-only HTTP surface: Lens does not serve source-code contents;
  qualifying local links may hand a validated path to the optional stable VS
  Code URL handler.
- The public PlantUML server is the default, and `LENS_PLANTUML_SERVER` fixes a
  replacement server for the full session. Local-command and disabled rendering
  modes are not supported.
