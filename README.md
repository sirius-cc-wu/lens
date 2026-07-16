# Lens

Lens is a standalone CLI that opens a local browser workspace for a codebase
and renders PlantUML blocks in Markdown documents.

## Build

```bash
cargo build --release
```

## Run

```bash
target/release/lens [PATH]
```

With no path, Lens uses the current Git repository. A file or directory may be
provided explicitly. Use `--no-open` for headless environments.

## Renderer

Remote rendering is disabled by default. Configure a PlantUML POST endpoint
with either:

```bash
LENS_RENDERER_URL=https://kroki.io/plantuml/svg target/release/lens
```

or:

```bash
target/release/lens --renderer-url http://127.0.0.1:8000/plantuml/svg
```

Only the selected PlantUML block source is sent to the configured renderer.

## Ignore Rules

Lens skips common generated and vendor directories. Add project-specific
gitignore-style patterns to `.lensignore`; `.gitignore` is also imported.
Explicitly selected files remain readable even when ignored in listings.

## Verification

```bash
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo package --locked --no-verify
```
