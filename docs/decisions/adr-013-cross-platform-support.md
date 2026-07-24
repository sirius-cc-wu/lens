---
type: "Architecture Decision"
title: "ADR-013: Support Native Browser Launch and Archives"
description: "Defines native browser-launch commands and release archives for Linux, macOS, and Windows."
id: "ADR-013"
status: "accepted"
date: "2026-07-19"
tags: [architecture, decision, portability]
---

# ADR-013: Support Native Browser Launch and Archives

Status: accepted

Date: 2026-07-19

## Context

Lens already has loopback browser pages, but command spawning and binary
artifacts differ among Linux, macOS, and Windows. Platform-specific code that
cannot be exercised outside its native operating system needs a portable test
boundary, and release automation needs a single native-target packaging contract.

## Decision

Lens represents its browser launch as a platform command before spawning it:
`xdg-open <URL>` on Linux, `open <URL>` on macOS, and
`cmd /C start "" <URL>` on Windows. Unit tests assert all three command forms
without requiring the corresponding operating system. Unsupported platforms
return an actionable launch error and retain the printed loopback URL fallback.

`scripts/package-release.sh` creates a target-named archive for Linux, macOS,
or Windows targets. It preserves the native executable filename (`lens.exe` on
Windows), README, license, checksum, and no-overwrite rule. Release automation
must run the script on a native runner for each published target.

## Consequences

- Lens has explicit supported launch behavior on three desktop platforms.
- The test suite checks platform command construction on every supported build,
  while native runners provide final process and artifact evidence.
- The legacy Linux packaging command remains available as a compatibility
  wrapper around the generic packager.
- Cross-platform scope remains documentation-only and does not authorize new
  file types beyond the document-set decisions.

## Trace

- Proposal: [Cross-Platform Support](../improvement-proposals.md#7-cross-platform-support)
- Previous scope: [ADR-004](adr-004-v1-release-scope.md)
- Artifact format: [ADR-010](adr-010-linux-binary-release-artifacts.md)
- Iteration: [`P7`](../iterations/p7-cross-platform-support.md)
