---
type: "Architecture Decision"
title: "ADR-014: Verify and Publish Tagged Native Releases"
description: "Defines repository-owned verification and atomic publication of tagged native release artifacts."
id: "ADR-014"
status: "accepted"
date: "2026-07-19"
tags: [architecture, decision, release]
---

# ADR-014: Verify and Publish Tagged Native Releases

Status: accepted

Date: 2026-07-19

## Context

Lens can build a target-specific archive locally, but a release needs repeatable
quality checks and evidence from each native platform. Publishing an artifact
before another platform fails would make the release incomplete, while
reimplementing packaging steps in workflow YAML could make the published output
differ from the documented command.

## Decision

GitHub Actions runs formatting, Rust tests, Clippy, package verification, and
the headless-browser suite for pull requests and pushes to `main`.

A `v*` tag starts a native release matrix: Linux builds
`x86_64-unknown-linux-gnu`, Intel macOS builds `x86_64-apple-darwin`, and
Windows builds `x86_64-pc-windows-msvc`. Each job installs the requested Rust
target and invokes `scripts/package-release.sh`; it does not reproduce archive
logic in the workflow. The package script accepts the GNU `sha256sum` command
or the macOS `shasum -a 256` equivalent so every native job writes the same
checksum format.

The workflow confirms that the tag is exactly `v` followed by the Cargo package
version. After all matrix jobs succeed, a publish job downloads their archives
and checksums and uses the repository-scoped `GITHUB_TOKEN` with
`contents: write` permission to create a GitHub Release with generated notes.

## Consequences

- Pull requests receive a repeatable regression signal before merge.
- A failed native package prevents release publication rather than producing a
  partial set of downloads.
- A matching tag and package version make release asset names traceable to the
  source manifest.
- The workflows require GitHub-hosted runners and do not replace native
  installation or desktop checks in the release checklist.

## Trace

- Proposal: [Release Automation](../improvement-proposals.md#8-release-automation)
- Artifact contract: [ADR-010](adr-010-linux-binary-release-artifacts.md)
- Platform support: [ADR-013](adr-013-cross-platform-support.md)
- Iteration: [`P8`](../iterations/p8-release-automation.md)
