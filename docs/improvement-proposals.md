---
type: "Improvement Proposals"
title: Lens Improvement Proposals
description: "Tracks stable post-V1 improvement proposals, their rationale, implementation status, and manual end-to-end verification."
status: "proposed"
tags: [planning, proposals]
---

# Lens Improvement Proposals

Status: proposed

These are candidate improvements after the V1 release. They are not release
commitments; a future iteration should select one based on user value, risk, and
implementation evidence. Implemented proposals remain as historical context
with their status recorded, and proposal numbers are stable and are not reused.

## Manual End-to-End Test Convention

Each **Manual end-to-end test** exercises Lens as a user would: start the built
command, interact with the browser or release service, and inspect the visible
result. For an implemented proposal, the walkthrough can be performed today.
For a proposed improvement, it defines the acceptance walkthrough that must
pass after implementation.

Build the current command with `cargo build --locked` before a local
walkthrough. Use a disposable document directory unless a proposal says
otherwise, and stop each Lens process before starting the next case. These
manual checks supplement, rather than replace, the automated checks in
[`docs/release-readiness.md`](release-readiness.md).

## 1. Local PlantUML Rendering

Add a `--renderer public|local|disabled` option. A local renderer would keep
diagram source on the user's machine and avoid dependence on the public
PlantUML service. This is the highest-value proposal because it addresses the
current privacy and renderer-availability risks.

Status: implemented in P1. `--renderer public|local|disabled` selects the
session renderer. Public remains the default; local invokes the installed
`plantuml` command without a renderer-service request, and disabled keeps the
diagram source visible without an image request.

### Manual end-to-end test

- **Setup:** Create a Markdown document containing one valid PlantUML block.
  Install the `plantuml` command, and disconnect the machine from the network
  after installation.
- **Actions:** Run `target/debug/lens --renderer local <document>`, then repeat
  with `--renderer disabled`. Reconnect the network and repeat with
  `--renderer public`.
- **Expected result:** The local session identifies the local renderer and
  displays the diagram while offline. The disabled session identifies
  rendering as disabled, displays no image, and keeps the source readable. The
  public session identifies the public renderer and displays the diagram after
  network access returns.

## 2. Prebuilt Linux Binaries

Publish Linux binaries and checksums with GitHub Releases. This would let users
install Lens without a Rust toolchain and reduce the friction of the current
source-install path.

Status: implemented in P2. The release-packaging command builds a target-bound
Linux archive containing Lens, the README, and license, and writes a SHA-256
checksum beside it. Proposal 8 will publish those verified artifacts from tags.

### Manual end-to-end test

- **Setup:** On Linux, download a release archive and its checksum into a clean
  environment where Rust and Cargo are not installed.
- **Actions:** Verify the checksum with `sha256sum --check`, extract the archive,
  and run the extracted `lens --help`. Start the extracted binary against a
  disposable Markdown directory.
- **Expected result:** The checksum passes; the archive contains one
  target-named directory with `lens`, `README.md`, and `LICENSE`; and the
  extracted binary opens the document without requiring a Rust toolchain.

## 5. Diagram Failure Controls

Expose renderer status, allow a failed diagram request to be retried, and let a
user disable diagram rendering for a session. These controls would make public
renderer failures clearer and give users a predictable fallback.

Status: implemented in P5. Each document exposes its renderer status, a failed
diagram presents a retry control, and a session disable control stops future
renderer requests while leaving PlantUML source readable.

### Manual end-to-end test

- **Setup:** Create two Markdown documents containing valid PlantUML blocks.
  Disconnect the machine from the network, then start Lens with the public
  renderer.
- **Actions:** Open the first document and observe its failures and source.
  Reconnect the network and select **Retry diagram rendering** on one diagram.
  Then select **Disable diagram rendering for this session** and navigate to the
  second document.
- **Expected result:** Each failure explains the renderer problem without
  hiding its source. Retry loads only the selected diagram after connectivity
  returns. Disabling updates the session status, prevents later diagrams from
  loading, and leaves every diagram source readable.

## 6. Standalone PlantUML Files

Allow Lens to open `.puml` files directly as well as PlantUML blocks embedded in
Markdown. This broadens diagram viewing without turning Lens into a general
source-code browser.

Status: implemented in P6. Lens discovers visible `.puml` files alongside
Markdown, accepts a direct `.puml` target, and serves each standalone source as
one diagram through the existing session-bound renderer route.

### Manual end-to-end test

- **Setup:** In one disposable directory, create `README.md`, a visible
  `architecture.puml`, and a hidden `.private.puml`.
- **Actions:** Start Lens on the directory, follow the navigation link to
  `architecture.puml`, and then start a new session by passing
  `architecture.puml` as the direct target.
- **Expected result:** The visible PlantUML file appears in navigation and both
  routes display one rendered diagram plus its source disclosure. The hidden
  file never appears, and requesting its path does not reveal its source.

## 7. Cross-Platform Support

Support macOS and Windows browser launch paths with platform-specific tests and
release artifacts. Linux remains the only supported V1 platform until this work
has evidence.

Status: implemented in P7. Lens selects the native Linux, macOS, or Windows
browser-launch command through platform-tested command construction, and the
target-aware release packager produces archives for each supported target on a
release runner using that operating system (native release runner).

### Manual end-to-end test

- **Setup:** Download the release archive and checksum built natively for
  Linux, macOS, or Windows. Repeat the walkthrough on all three operating
  systems.
- **Actions:** Verify the checksum, extract the archive, and run the native
  binary against a Markdown directory. Confirm that the default browser opens.
  Repeat in a shell where the platform launcher (`xdg-open`, `open`, or
  `cmd`) is intentionally unavailable.
- **Expected result:** Linux and macOS run `lens`, Windows runs `lens.exe`, and
  each opens its native default browser. When launching is unavailable, Lens
  remains running and prints a loopback URL that opens manually.

## 8. Release Automation

Add GitHub Actions checks for formatting, tests, Clippy, package verification,
tagged releases, and binary publishing. This would make each release
repeatable and reduce regression risk.

Status: implemented in P8. GitHub Actions verifies formatting, Rust tests,
Clippy, package metadata, and the browser suite on pull requests and `main`.
A `v<package-version>` tag starts native Linux, macOS, and Windows packaging,
then publishes the archives and SHA-256 checksums only after every matrix job
succeeds.

### Manual end-to-end test

- **Setup:** Use a test fork for failure cases and the main repository for an
  approved release candidate. Open a pull request with a harmless documentation
  change.
- **Actions:** Inspect the **Verify** workflow and confirm that formatting,
  Rust tests, Clippy, package metadata, and browser tests all run. In the fork,
  push a tag whose version differs from `Cargo.toml`. For the approved release,
  push the matching `v<package-version>` tag.
- **Expected result:** The pull request cannot present a fully green workflow
  until every check passes. The mismatched tag publishes nothing. The matching
  tag waits for Linux, macOS, and Windows package jobs, then creates one GitHub
  Release containing every native archive and checksum.

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

### Manual end-to-end test

- **Setup:** Create four Markdown files: valid metadata closed with `---`, valid
  metadata closed with `...`, malformed YAML, and an unclosed header. Include
  scalar, list, nested, unknown, and HTML-shaped values plus ordinary body text.
- **Actions:** Open the directory in Lens and visit all four files from the
  navigation pane.
- **Expected result:** Both valid headers appear as structured document
  metadata without delimiters in the body. Nested and unknown values remain
  visible, and HTML-shaped values display as text rather than execute. Both
  invalid files show corrective guidance while keeping the Markdown body
  readable.

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

### Manual end-to-end test

- **Setup:** Create 51 visible documents whose identifiers contain
  `reference`, plus one unrelated document and one hidden document. Start Lens,
  then create one more matching file after the session is already running.
- **Actions:** Search for mixed-case `ReFeReNcE`, confirm the first result page,
  and follow **Next results**. Disable JavaScript and repeat. Submit a query
  longer than 256 UTF-8 bytes, request an invalid page number, and try the URL
  of the file created after startup.
- **Expected result:** Search is case-insensitive, shows no more than 50
  authorized identifiers per page, and reaches the 51st result through native
  links with JavaScript disabled. The long query shows limit guidance, the
  invalid page returns the first valid result page, and neither the hidden nor
  post-start file becomes reachable.

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

### Manual end-to-end test

- **Setup:** Start Lens on a directory containing at least two documents. Use
  only the keyboard for the first hide and restore cycle.
- **Actions:** Focus **Hide documents**, activate it, and navigate directly to
  the second authorized document in the same tab. Confirm the pane remains
  hidden, then activate **Show documents**. Open the same Lens URL in a new tab.
- **Expected result:** The control is reachable and operable by keyboard, its
  label reflects the visible state, and hiding gives the document more width.
  The same tab preserves the hidden state across document navigation and
  restores the pane with the current document marked. The new tab starts with
  its own visible pane, and every authorized route works in either state.

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

### Manual end-to-end test

This proposal is not implemented.

- **Setup:** Before the split, keep a copy of the current Lens binary. Prepare a
  directory that exercises Markdown navigation, catalog search, automatic
  refresh, standalone and embedded PlantUML, renderer failure, and YAML
  frontmatter. Also keep a small external program that starts Lens through
  `lens::serve`. Build the post-split binary separately.
- **Actions:** Perform the same walkthrough with both binaries: open every
  document type, search and paginate, save a visible change, retry a failed
  diagram, disable rendering, and request an unauthorized path. Compare the
  page text, controls, routes, and browser network responses. Point the external
  program at the post-split Lens library, build it, and repeat the walkthrough.
- **Expected result:** A user cannot distinguish the two builds. The
  external program still starts Lens through `lens::serve`; page assets and
  browser restrictions are still served; and only source module locations have
  changed.

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

### Manual end-to-end test

This proposal is not implemented.

- **Setup:** Use the proposal's repeatable generator to create fixed 1,000- and
  10,000-document repositories. Record the fixture seed, Lens build, operating
  system, processor, memory, and storage. Close other Lens sessions.
- **Actions:** Start Lens at least 20 times on each repository and record time
  until the first page responds and peak memory use. For one run, record idle
  CPU and file activity for 60 seconds. Submit each matching, non-matching,
  first-page, and last-page search at least 20 times. Add and change files
  outside the document paths authorized at startup.
- **Expected result:** Report the median and the threshold met by 95 percent of
  samples (95th percentile) for startup and search time, plus peak resident
  memory (RSS) and idle work. Every result stays within the recorded budgets,
  the browser remains responsive, and out-of-session changes never appear.

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

### Manual end-to-end test

This proposal is not implemented.

- **Setup:** For each document-size, document-count, directory-depth, and YAML
  nesting limit, prepare repositories immediately below, exactly at, and
  immediately above the limit. Also prepare encoded traversal URLs, equivalent
  Unicode spellings, HTML-shaped metadata, oversized PlantUML, and a document
  that can be saved in partial stages.
- **Actions:** Start Lens on every boundary repository. Request each adversarial
  URL, open each unusual document, and repeatedly truncate, remove, and restore
  the staged-save document while its page is visible.
- **Expected result:** Below-limit and at-limit repositories open normally.
  Above-limit cases stop promptly and identify the affected resource and
  corrective action. No path reveals an unauthorized file, no metadata executes
  as HTML, oversized input stays bounded, and partial saves retain the last
  readable page until a complete save is available.

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

### Manual end-to-end test

This proposal is not implemented.

- **Setup:** Create a document with a unique sentence inside a PlantUML block.
  Start a local HTTP stand-in for the public renderer that records requests and
  returns a valid SVG. Point `LENS_PLANTUML_SERVER` to it.
- **Actions:** Start Lens with no renderer option and inspect both the page and
  request log. Stop it, clear the log, and repeat with `--renderer public`.
  Disconnect the stand-in to exercise failure, retry, and session disable.
- **Expected result:** The default session sends no request, explains that
  public rendering needs consent, and keeps the unique source visible. Explicit
  public mode explains where source is sent and produces one diagram request at
  the stand-in. Existing timeout, size-limit, retry, failure, and disable
  behavior remains available. Release notes and usage examples call out the
  changed default.

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

### Manual end-to-end test

This proposal is not implemented.

- **Setup:** Use a headless shell with no desktop browser. Choose one available
  loopback port and keep a second terminal ready.
- **Actions:** Start Lens with `--no-open --port <available-port>`, capture the
  machine-readable URL, and request it with `curl`. Start a second Lens process
  on the occupied port. Then start Lens with `--no-open --port 0` and request
  its reported URL.
- **Expected result:** No browser-launch attempt occurs. The first URL uses
  `127.0.0.1` and the selected port and serves the document. The second process
  exits with corrective port-in-use guidance. Port zero reports a usable
  operating-system-assigned port. No option exposes a non-loopback listener.

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

### Manual end-to-end test

This proposal is not implemented.

- **Setup:** Create a long Markdown document with a fragment target and a
  PlantUML block. Make its renderer unavailable so the retry control appears.
  Use that control as the focusable element and the PlantUML source disclosure
  as the closed disclosure. Open browser developer tools so revision requests
  can be blocked temporarily.
- **Actions:** Navigate to the fragment, scroll farther, focus the retry control
  without activating it, open the source disclosure, and save visible body
  changes. Repeat after removing the PlantUML block so the focused control no
  longer exists. Finally, block revision requests, make another save, and later
  unblock them.
- **Expected result:** Each successful refresh shows new content while
  preserving the fragment and approximately the same scroll position. Focus and
  disclosure state are restored when their elements still exist; removing the
  focused element causes no error. Failed polling leaves the current page
  readable, and recovery performs at most one required refresh rather than a
  reload loop.

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

### Manual end-to-end test

This proposal is not implemented.

- **Setup:** Open an automated dependency-update pull request. In a test fork,
  prepare one branch with a controlled vulnerable dependency or disallowed
  license. Prepare the next release candidate with its changelog, version bump,
  and cross-platform documentation.
- **Actions:** Inspect continuous integration for the dependency pull request
  and confirm separate Rust 1.75 and current-stable jobs. Run the scheduled
  dependency and license workflow on the controlled failure branch. Review the
  release notes, push the approved matching tag, download every native archive,
  and verify its checksum and contents.
- **Expected result:** Dependency updates cannot merge unless both Rust
  versions and the complete baseline pass. The controlled advisory or license
  violation fails with actionable output. The changelog, `Cargo.toml`, tag, and
  platform documentation agree on the release, and all published archives pass
  the release-readiness walkthrough.
