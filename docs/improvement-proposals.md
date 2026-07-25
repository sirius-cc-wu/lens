---
type: "Improvement Proposals"
title: Lens Improvement Proposals
description: "Tracks stable post-V1 improvement proposals, their rationale, and manual end-to-end verification."
status: "proposed"
tags: [planning, proposals]
---

# Lens Improvement Proposals

Status: proposed

These are candidate improvements after the V1 release. They are not release
commitments; a future iteration should select one based on user value, risk, and
implementation evidence. Implemented proposals are removed from this list; the
remaining numbers are stable and are not reused.

## Manual End-to-End Test Convention

Each **Manual end-to-end test** exercises Lens as a user would: start the built
command, interact with the browser or release service, and inspect the visible
result. For a proposed improvement, it defines the acceptance walkthrough that
must pass after implementation.

Build the current command with `cargo build --locked` before a local
walkthrough. Use a disposable document directory unless a proposal says
otherwise, and stop each Lens process before starting the next case. These
manual checks supplement, rather than replace, the automated checks in
[`docs/release-readiness.md`](release-readiness.md).

## 14. Measured Large-Repository Scalability

Define performance budgets, meaning maximum acceptable time and resource use,
for repositories containing 1,000 and 10,000 discovered documents. Measure
startup discovery time, initial-response time, idle refresh work, and memory
use before selecting an optimization.

Current discovery reads every supported document eagerly, automatic refresh
rereads every known document every 500 milliseconds, and session creation keeps
both rendered documents and fixed authorization identifiers in memory.
Candidate changes include checking file metadata before reading content,
rendering the initial document before the rest of the set when authorization
allows it, and using filesystem events. Any event-based design must filter
changes through the immutable set of canonical document paths authorized when
the session starts. Add repeatable performance fixtures and record the accepted
budgets as release evidence.

### Manual end-to-end test

This proposal is not implemented.

- **Setup:** Use the proposal's repeatable generator to create fixed 1,000- and
  10,000-document repositories. Record the fixture seed, Lens build, operating
  system, processor, memory, and storage. Close other Lens sessions.
- **Actions:** Start Lens at least 20 times on each repository and record time
  until the first page responds and peak memory use. For one run, record idle
  CPU and file activity for 60 seconds. Open several known document routes and
  save the displayed document repeatedly. Add and change files outside the
  document paths authorized at startup.
- **Expected result:** Report the median and the threshold met by 95 percent of
  samples (95th percentile) for startup and known-document response time, plus
  peak resident memory (RSS) and idle work. Every result stays within the
  recorded budgets, the browser remains responsive, and out-of-session changes
  never appear.

## 15. Bounded and Adversarial Input Handling

Define explicit limits for document size, discovered document count, directory
depth, and YAML frontmatter nesting. When a repository exceeds a limit, report
the affected resource and corrective action instead of allowing unbounded
startup memory or parsing work. Keep the existing diagram-output limit
consistent with this policy.

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

Status: superseded by ADR-017. This proposal is retained as a rejected
alternative: the accepted server-only design preserves the public default,
supports a session-fixed private server, and removes renderer and disable
choices.

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
describe Linux, macOS, and Windows consistently. Retire completed proposals
from the active list after linking accepted decisions to their construction
records so that implementation history remains discoverable.

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
