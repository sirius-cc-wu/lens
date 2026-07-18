# ADR-009: Package Target-Specific Binary Release Artifacts

Status: accepted

Date: 2026-07-19

## Context

Installing Lens from source requires a Rust toolchain. A native-platform release
needs a small artifact that users can verify before extracting, while the
repository still needs one packaging definition that later tag automation can
reuse.

## Decision

`scripts/package-release.sh` builds Lens in release mode for an explicit Linux,
macOS, or Windows Rust target and writes two sibling files: a target-named
`tar.gz` archive and its SHA-256 checksum. The archive contains one target-named
directory with the native executable, `README.md`, and `LICENSE`.

The command defaults to the host Rust target, accepts `--target` and `--output`,
requires a supported target name, and refuses to overwrite either output file.
It uses `cargo build --locked --release --target` so the package follows the
locked dependency set. `scripts/package-linux-release.sh` remains a compatibility
entry point. GitHub Release upload is deliberately deferred to proposal 8; that
workflow must call this command rather than duplicate packaging logic.

## Consequences

- Users can install a verified native binary without a Rust toolchain.
- Each architecture is explicit in its filename and must be built by a runner
  with that Rust target installed.
- The release process depends on `tar` and SHA-256 tooling supplied by each
  native release runner.
- A maintainer must choose or automate the set of target architectures.

## Trace

- Proposal: [Prebuilt Linux Binaries](../improvement-proposals.md#2-prebuilt-linux-binaries)
- Risk: `R-07` in [`docs/risk-list.md`](../risk-list.md)
- Verification: [`docs/release-readiness.md`](../release-readiness.md)
- Iteration: [`P2`](../iterations/p2-linux-binary-packaging.md)
