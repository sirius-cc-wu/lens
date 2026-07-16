# Release

## Versioning

Lens follows Semantic Versioning. The current release line is `0.1.x` while
the CLI and browser workspace contracts are still stabilizing.

- Patch: bug fixes without intentional contract changes.
- Minor: backward-compatible features or documented API additions.
- Major: incompatible CLI, workspace, or renderer contract changes.

## Artifacts

Release archives use this naming scheme:

```text
lens-[version]-[target].tar.gz
lens-[version]-[target].tar.gz.sha256
```

Each archive contains the native `lens` binary and `README.md`. CI currently
builds Linux GNU, macOS ARM64, and Windows MSVC targets.

## Checklist

- Update the package version in `Cargo.toml`.
- Update release notes and verify renderer/privacy behavior.
- Run `cargo fmt --check`.
- Run `cargo test --locked`.
- Run `cargo clippy --locked --all-targets --all-features -- -D warnings`.
- Run `cargo build --locked --release`.
- Run `cargo package --locked --allow-dirty --no-verify` locally.
- Run `node scripts/asset-check.mjs`.
- Run the Chrome shell and interaction tests against a running local fixture
  workspace.
- Run `bash scripts/install-smoke.sh ARCHIVE` for each release archive.
- Verify each archive checksum before publishing.
