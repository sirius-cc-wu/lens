---
title: Lens Improvement Proposals
---

# Lens Improvement Proposals

Status: proposed

These are candidate improvements after the V1 release. They are not release
commitments; a future iteration should select one based on user value, risk, and
implementation evidence.

## 1. Local PlantUML Rendering

Add a `--renderer public|local|disabled` option. A local renderer would keep
diagram source on the user's machine and avoid dependence on the public
PlantUML service. This is the highest-value proposal because it addresses the
current privacy and renderer-availability risks.

## 2. Prebuilt Linux Binaries

Publish Linux binaries and checksums with GitHub Releases. This would let users
install Lens without a Rust toolchain and reduce the friction of the current
source-install path.

## 3. Document Navigation Pane

Add a sidebar containing the discovered document set, current-document
highlighting, and search. The sidebar must use the existing authorized document
set so it does not broaden filesystem access.

## 4. Automatic Refresh

Watch discovered Markdown files and refresh the browser view when they change.
This would support authors who use Lens to preview documentation while editing.

## 5. Diagram Failure Controls

Expose renderer status, allow a failed diagram request to be retried, and let a
user disable diagram rendering for a session. These controls would make public
renderer failures clearer and give users a predictable fallback.

## 6. Standalone PlantUML Files

Allow Lens to open `.puml` files directly as well as PlantUML blocks embedded in
Markdown. This broadens diagram viewing without turning Lens into a general
source-code browser.

## 7. Cross-Platform Support

Support macOS and Windows browser launch paths with platform-specific tests and
release artifacts. Linux remains the only supported V1 platform until this work
has evidence.

## 8. Release Automation

Add GitHub Actions checks for formatting, tests, Clippy, package verification,
tagged releases, and binary publishing. This would make each release
repeatable and reduce regression risk.
