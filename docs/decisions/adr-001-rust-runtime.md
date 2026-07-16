# ADR-001: Use Rust for the MVP Runtime

Status: accepted for MVP implementation

## Context

Lens needs a distributable CLI that resolves local paths, serves a browser
workspace, protects file access, and coordinates PlantUML rendering. The
project preference was Rust, Python, or C#, with TypeScript acceptable only if
the preferred languages were unsuitable.

## Decision

Use Rust for the Lens MVP runtime.

The E1 slice uses Rust's standard library for the CLI, local HTTP server,
filesystem boundary, and testable renderer abstraction. E2 replaces the
subprocess renderer with `ureq` using Rustls for HTTPS PlantUML POST requests,
without coupling the domain-facing operation to one renderer deployment.

## Evidence

- `cargo test` passes six tests covering target forms, path safety, Markdown
  extraction, successful rendering, HTTP routes, and renderer failure.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.
- The executable starts a local workspace with `--no-open` and serves its root
  page successfully.
- The standard-library slice has no Rust dependency download or runtime
  framework requirement.

## Consequences

- The CLI can remain a single native binary with a small runtime footprint.
- Filesystem and HTTP behavior are explicit and easy to test at the boundary.
- The production browser asset pipeline and richer HTTP behavior still need
  deliberate implementation rather than inheriting framework defaults.
- The binary carries a Rustls-based HTTP client and no longer requires an
  external `curl` executable.

## Revisit When

- The browser client needs framework-level asset bundling or websocket support.
- Renderer protocol requirements make a native HTTP client materially safer or
  more portable than the current adapter.
- Packaging evidence shows the runtime or external `curl` dependency blocks
  supported platforms.
