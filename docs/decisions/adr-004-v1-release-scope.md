# ADR-004: Release V1 as a Linux Documentation Viewer

Status: accepted

Date: 2026-07-18

## Context

Lens can now select a document root, display discovered Markdown documents, and
render PlantUML through the public server. The original phrase "display the
codebase's code and document" still leaves code-file browsing open to multiple
product interpretations.

## Decision

V1 is a documentation-only release for Linux. It ships as a Rust Cargo package
under the MIT License and uses the existing `xdg-open` browser-launch path.

V1 does not display or navigate repository source-code files. Code-file browsing
requires a later use case, authorization model, and release decision.

## Consequences

- The V1 acceptance surface is Markdown target selection, discovered-document
  navigation, PlantUML rendering and failure behavior, and loopback browser
  launch fallback.
- Linux is the only supported release platform. The macOS and Windows launch
  paths are not release commitments until separately validated.
- `cargo install --path . --locked` is the supported source-install command.
- No code-file route or code-syntax dependency is added to the viewing session.

## Trace

- Product scope: [`Vision`](../vision.md)
- Deferred code browsing: [`UC-06`](../features/markdown-viewing/use-cases.md)
- Release verification: [`release readiness`](../release-readiness.md)
