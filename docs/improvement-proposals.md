---
title: Lens Improvement Proposals
---

# Lens Improvement Proposals

Status: proposed

These are candidate improvements after the V1 release. They are not release
commitments; a future iteration should select one based on user value, risk, and
implementation evidence. Implemented proposals remain as historical context
with their status recorded, and proposal numbers are stable and are not reused.

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

Status: implemented in C6. Lens provides a browser-operated hide/show control
for the navigation pane, exposes its state to assistive technology, and keeps
the preference while the same browser tab views other authorized documents.

Let the user hide and restore the document navigation pane so the document
content can use more of the browser window. Provide a visible, keyboard-
operable control with an accessible expanded-state indication, and retain a
usable way to restore the pane after it is hidden. Remembering the user's
choice for the current viewing session would avoid making the user repeat the
action on every document change. This changes presentation only: hiding the
pane must not change the active session's authorized document set or document
routes.

## 13. Modular Viewer Responsibilities

Split the viewer implementation along its existing capability boundaries while
preserving behavior and the public `lens::serve` path. The current viewer module
owns session state, document refresh, browser launching, HTTP routes, navigation
markup, PlantUML requests, JavaScript, CSS, and their tests. These concerns can
change independently and already have distinct types, dependencies, and test
scenarios.

Keep session and refresh state together, move route handlers and page
composition into cohesive modules, isolate public and local diagram rendering,
and place browser-launch construction with its platform tests. Store the
JavaScript and CSS as dedicated assets embedded into the binary at compile time.
Make the extraction mechanical: do not rename public APIs or redesign behavior,
and retain tests with the module that owns each behavior. Verify the split with
the complete Rust and browser suites.

## 14. Measured Large-Repository Scalability

Define performance budgets, meaning maximum acceptable time and resource use,
for repositories containing 1,000 and 10,000 discovered documents. Measure
startup discovery time, idle refresh work, memory use, and catalog-search
latency before selecting an optimization.

Current discovery reads every supported document eagerly, automatic refresh
rereads every known document every 500 milliseconds, and each catalog search
scans the complete identifier set. Candidate changes include checking file
metadata before reading content, retaining normalized search keys, completing a
search count and result page in one traversal, and using filesystem events.
Any event-based design must filter changes through the immutable set of
canonical document paths authorized when the session starts. Add repeatable
performance fixtures and record the accepted budgets as release evidence.

## 15. Bounded and Adversarial Input Handling

Define explicit limits for document size, discovered document count, directory
depth, and YAML frontmatter nesting. When a repository exceeds a limit, report
the affected resource and corrective action instead of allowing unbounded
startup memory or parsing work. Keep the existing query-size and diagram-output
limits consistent with this policy.

Add adversarial tests, meaning tests built from malicious or unusually extreme
input, for relative-path traversal, percent-encoded and Unicode identifiers,
deep or malformed YAML, oversized Markdown and PlantUML sources, deeply nested
directories, and partial document saves. Add generated-input tests for path
normalization and frontmatter parsing so that broad classes of inputs supplement
the existing hand-selected examples. Preserve the fixed session authorization
boundary and last-readable-document behavior in every failure case.

## 16. Explicit Public Diagram Rendering Consent

Make sending PlantUML source to a public rendering service an explicit user
choice. A future breaking release should either default to disabled rendering
or automatically select an available local renderer and otherwise remain
disabled. Public rendering would require `--renderer public`, with CLI and page
text explaining that diagram source is sent to the configured service.

Do not issue a public renderer request before that choice has been made. Retain
the current timeout, response-size limit, failure fallback, retry control, and
session disable behavior for users who select the public service. Document the
default change prominently in release notes and installation examples.

## 17. Headless and Automated Serving Controls

Support headless environments, meaning sessions without a desktop browser, and
scripted use without weakening the loopback-only default. Add `--no-open` to
suppress browser launching and `--port <PORT>` to select a predictable loopback
port, with port zero retaining the current operating-system-assigned behavior.
Provide a stable machine-readable way to obtain the serving URL.

Define actionable behavior for an unavailable requested port and keep printing
the manual URL for ordinary browser-launch failures. Add CLI tests for argument
parsing and browser suppression, plus an integration scenario that starts Lens
on a selected loopback port. External network binding should require a separate
security and product decision rather than being introduced by this proposal.

## 18. Reading-Context-Preserving Refresh

Preserve the reader's location and local page state when automatic refresh
detects a saved document change. Before reloading, retain the current fragment,
scroll position, focused element when practical, and the open state of document
disclosures. Restore that context after the refreshed page becomes readable.

Keep the revision endpoint small and retain the current fallback when revision
polling fails. Add a browser scenario that scrolls within a long document,
opens a disclosure, saves a change, and verifies that the refreshed content and
reading context are both preserved. Avoid a partial page-update design unless
measurement shows that a full reload with state restoration is insufficient.

## 19. Release and Dependency Maintenance

Make compatibility and supply-chain maintenance routine. Test both Rust 1.75,
the minimum supported Rust version (MSRV), and the current stable Rust release
in continuous integration. Add scheduled dependency advisory and license
checks, and configure automated dependency-update pull requests whose changes
must pass the existing locked Rust and browser suites.

Establish a post-V1 release record with a changelog and a package-version bump
before the next tag. Update package metadata and introductory documentation to
describe Linux, macOS, and Windows consistently. Keep completed proposals as
clearly marked historical evidence or move them to a dedicated history section
so that active proposals are immediately distinguishable.
