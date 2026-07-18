---
title: Lens Improvement Proposals
---

# Lens Improvement Proposals

Status: proposed

These are candidate improvements after the V1 release. They are not release
commitments; a future iteration should select one based on user value, risk, and
implementation evidence. Implemented proposals are removed from this list; the
remaining numbers are stable and are not reused.

## 1. Local PlantUML Rendering

Add a `--renderer public|local|disabled` option. A local renderer would keep
diagram source on the user's machine and avoid dependence on the public
PlantUML service. This is the highest-value proposal because it addresses the
current privacy and renderer-availability risks.

Status: implemented in P1. `--renderer public|local|disabled` selects the
session renderer. Public remains the default; local invokes the installed
`plantuml` command without a renderer-service request, and disabled keeps the
diagram source visible without an image request.

## 2. Prebuilt Linux Binaries

Publish Linux binaries and checksums with GitHub Releases. This would let users
install Lens without a Rust toolchain and reduce the friction of the current
source-install path.

Status: implemented in P2. The release-packaging command builds a target-bound
Linux archive containing Lens, the README, and license, and writes a SHA-256
checksum beside it. Proposal 8 will publish those verified artifacts from tags.

## 5. Diagram Failure Controls

Expose renderer status, allow a failed diagram request to be retried, and let a
user disable diagram rendering for a session. These controls would make public
renderer failures clearer and give users a predictable fallback.

Status: implemented in P5. Each document exposes its renderer status, a failed
diagram presents a retry control, and a session disable control stops future
renderer requests while leaving PlantUML source readable.

## 6. Standalone PlantUML Files

Allow Lens to open `.puml` files directly as well as PlantUML blocks embedded in
Markdown. This broadens diagram viewing without turning Lens into a general
source-code browser.

Status: implemented in P6. Lens discovers visible `.puml` files alongside
Markdown, accepts a direct `.puml` target, and serves each standalone source as
one diagram through the existing session-bound renderer route.

## 7. Cross-Platform Support

Support macOS and Windows browser launch paths with platform-specific tests and
release artifacts. Linux remains the only supported V1 platform until this work
has evidence.

Status: implemented in P7. Lens selects the native Linux, macOS, or Windows
browser-launch command through platform-tested command construction, and the
target-aware release packager produces archives for each supported target on a
native release runner.

## 8. Release Automation

Add GitHub Actions checks for formatting, tests, Clippy, package verification,
tagged releases, and binary publishing. This would make each release
repeatable and reduce regression risk.

Status: implemented in P8. GitHub Actions verifies formatting, Rust tests,
Clippy, package metadata, and the browser suite on pull requests and `main`.
A `v<package-version>` tag starts native Linux, macOS, and Windows packaging,
then publishes the archives and SHA-256 checksums only after every matrix job
succeeds.

## 10. YAML Frontmatter Rendering

Detect YAML frontmatter at the beginning of Markdown documents and render it as
readable document metadata instead of leaving it hidden or treating it as body
text. Define a consistent presentation for common fields, preserve unknown and
nested values safely, and show an actionable result when frontmatter is
malformed. Add fixtures and browser tests covering valid metadata, delimiters,
and invalid YAML.

Status: implemented in P10. Lens recognizes a leading `---` header and either
`---` or `...` closing delimiter, displays scalar, list, nested, and unknown
YAML values as escaped metadata, and retains the Markdown body with a
correction message when the header cannot be parsed.

## 11. Scalable Document Navigation Search

Status: implemented in C5. Lens searches only the immutable, authorized
session catalog through a native GET form, returns no more than 50 identifiers
per page, and keeps pagination usable without JavaScript.

Replace the complete document list in every navigation pane with server-side
identifier search and a capped result set. Lens would build an index from the
active session's already authorized document identifiers when it starts, then
return only a bounded first page of matching identifiers and allow the user to
request further results. This would make large documentation trees practical
without scanning the filesystem or making arbitrary paths reachable after the
session begins. It requires revisiting ADR-006, which currently requires the
complete catalog in every document response, and defining pagination,
no-JavaScript navigation, result limits, and request-rate behavior.

## 12. Collapsible Document Navigation Pane

Let the user hide and restore the document navigation pane so the document
content can use more of the browser window. Provide a visible, keyboard-
operable control with an accessible expanded-state indication, and retain a
usable way to restore the pane after it is hidden. Remembering the user's
choice for the current viewing session would avoid making the user repeat the
action on every document change. This changes presentation only: hiding the
pane must not change the active session's authorized document set or document
routes.
