# ADR-009: Package Linux Binary Release Artifacts

Status: accepted

Date: 2026-07-19

## Context

Installing Lens from source requires a Rust toolchain. A Linux release needs a
small artifact that users can verify before extracting, while the repository
still needs one packaging definition that later tag automation can reuse.

## Decision

`scripts/package-linux-release.sh` builds Lens in release mode for an explicit
Linux Rust target and writes two sibling files: a target-named `tar.gz` archive
and its SHA-256 checksum. The archive contains one target-named directory with
the executable, `README.md`, and `LICENSE`.

The command defaults to the host Rust target, accepts `--target` and `--output`,
requires a Linux target name, and refuses to overwrite either output file. It
uses `cargo build --locked --release --target` so the package follows the locked
dependency set. GitHub Release upload is deliberately deferred to proposal 8;
that workflow must call this command rather than duplicate packaging logic.

## Consequences

- Users can install a verified binary without a Rust toolchain.
- Each architecture is explicit in its filename and must be built by a runner
  with that Rust target installed.
- The release process depends on standard Linux `tar` and `sha256sum` tools.
- A maintainer must choose or automate the set of target architectures.

## Trace

- Proposal: [Prebuilt Linux Binaries](../improvement-proposals.md#2-prebuilt-linux-binaries)
- Risk: `R-07` in [`docs/risk-list.md`](../risk-list.md)
- Verification: [`docs/release-readiness.md`](../release-readiness.md)
- Iteration: [`P2`](../iterations/p2-linux-binary-packaging.md)
