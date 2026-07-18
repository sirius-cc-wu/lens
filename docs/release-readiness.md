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
discovered document, automatic refresh after a saved change, 404 guidance for
an undiscovered document, and visible PlantUML success and failure behavior.

## Installation Check

On a clean Linux shell with Rust 1.75 or newer:

```bash
cargo install --path . --locked
lens --help
```

Expected result: `lens --help` describes an optional `TARGET` argument.

## Linux Binary Archive Check

On a Linux host with the selected Rust target installed, build a fresh archive:

```bash
scripts/package-linux-release.sh --target x86_64-unknown-linux-gnu --output /tmp/lens-release
cd /tmp/lens-release
sha256sum --check lens-*-x86_64-unknown-linux-gnu.tar.gz.sha256
tar -tzf lens-*-x86_64-unknown-linux-gnu.tar.gz
```

Expected result: checksum verification succeeds and the archive contains a
single target-named directory with `lens`, `README.md`, and `LICENSE`. The
packaging command refuses to overwrite an existing archive or checksum.

## Package Metadata

- `Cargo.toml` declares the MIT license and points to `LICENSE`.
- `Cargo.toml` identifies the public repository, homepage, and hosted
  documentation URLs used by release metadata.

## Target Checks

From a repository that contains `README.md` or `docs/index.md`:

```bash
lens
lens docs
lens docs/features/markdown-viewing/oc-02-open-document-root.md
```

Expected results:

- Lens prints a loopback URL and opens it with `xdg-open`, or prints the URL for
  manual opening if browser launch fails.
- The initial document follows the root README, `docs/index`, then lexical-path
  selection order.
- A sibling Markdown link opens its discovered target document.
- An unknown or out-of-root local path shows the Lens guidance page and does not
  disclose a file.

## Rendering Checks

Open a document containing a valid PlantUML block and one with invalid PlantUML.

Expected results:

- The valid diagram appears as SVG.
- The failed diagram keeps its source visible with an error.
- The remainder of the document remains readable.

## V1 Boundaries

- Linux only.
- Documentation-only: source-code browsing is not part of V1.
- Public PlantUML rendering is the default; local and disabled renderer modes
  are documented in the README.
