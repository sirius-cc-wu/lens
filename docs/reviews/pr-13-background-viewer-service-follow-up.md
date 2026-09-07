---
type: "Code Review"
title: "PR #13 Follow-up Review: Browser Authorization and Startup"
description: "Records four confirmed findings on the revised background service, with browser, concurrency, and native CI evidence."
pull_request: 13
status: "completed"
date: "2026-09-07"
tags: [review, background-service, security, concurrency]
---

# PR #13 Follow-up Review

Review range: `1c6cd2d5e620991b3968dd39150c85e9ee1c4c7b..c3bc76b953c2235aae2787a1b9ccaa0488487df6`.
The local `feat/background-viewer-service` branch matched the pull request head.
This review covers the full PR and the fixes recorded in the
[earlier review](pr-13-background-viewer-service.md). That historical record's
resolution status does not establish approval of this revised head.

Resolution status: All four findings were resolved after review and passed the
resolution validation recorded below.

## Findings

1. [High] Host-wide cookies break session isolation and disclose the browser credential — [`src/viewer/routes.rs:72`](../../src/viewer/routes.rs#L72)

   Explanation and impact: Every session sets the same `lens-session` cookie
   for `127.0.0.1` with `Path=/`, although each listener uses a different port.
   Cookies do not isolate ports, as specified in
   [RFC 6265, section 8.5](https://www.rfc-editor.org/rfc/rfc6265#section-8.5).
   Opening session B therefore replaces session A's cookie. A's authored
   document links and revision polling omit the URL token and start receiving
   HTTP 401; reloading A's original ready URL restores A but breaks B.

   The same scope also sends the credential to an unrelated HTTP server on
   another `127.0.0.1` port when the browser visits it. `HttpOnly` prevents
   JavaScript cookie reads, not receipt by that server, and `SameSite=Strict`
   does not distinguish these ports. A server operated by a different local
   user can replay the received cookie against the Lens listener to read its
   retained documents. This defeats the new cross-user browser protection.

   Reproduction: In one isolated Chrome context, open two URLs returned by the
   same test-owned service. A's revision endpoint changes from 200 to 401 and
   its linked-document endpoint returns 401. Navigate B to a separate local
   test HTTP server; it receives the session cookie. Replaying that header
   against B returns 200 and the synthetic fixture document. No real documents
   or credentials were used or logged.

   Reported behavior and impact:

   ```plantuml
   @startuml
   title One host-wide cookie crosses viewer ports
   participant Browser
   participant "Session A" as A
   participant "Session B" as B
   participant "Other local HTTP server" as Other
   Browser -> A : open ready URL
   A --> Browser : set lens-session=A
   Browser -> B : open ready URL
   B --> Browser : replace cookie with lens-session=B
   Browser -> A : poll revision with cookie B
   A --> Browser : 401; refresh stops
   Browser -> Other : visit another port; cookie B included
   Other -> B : replay cookie B
   B --> Other : retained document
   @enduml
   ```

   Proposed fix: Carry the random access credential (session capability) only
   in requests explicitly addressed to that viewer origin, rather than a
   host-wide cookie. For example, consistently include it in generated local
   document, asset, diagram, and revision URLs, with `Referrer-Policy:
   no-referrer` to prevent navigation from disclosing it. An origin-scoped
   browser bootstrap with explicitly authenticated requests is another option.
   Merely making cookie names unique fixes replacement but does not prevent
   another local HTTP server from receiving them.

   Suggested solution:

   ```plantuml
   @startuml
   title Keep capabilities in viewer-specific requests
   participant Browser
   participant "Session A" as A
   participant "Session B" as B
   participant "Other local HTTP server" as Other
   Browser -> A : local routes explicitly carry capability A
   A --> Browser : document and revision; no-referrer policy
   Browser -> B : local routes explicitly carry capability B
   B --> Browser : independent document and revision
   Browser -> Other : external navigation without credential or referrer
   Other -> B : unauthenticated request
   B --> Other : 401
   @enduml
   ```

   Test coverage: Add a shared-browser-context test that opens two sessions,
   navigates in both, saves each document, and verifies both refresh. Add a
   second-port capture test proving no capability is transmitted and no
   credential replay can read the viewer. The current cross-session browser
   test sends a mismatched query token without bootstrapping either session,
   so it cannot detect cookie replacement or disclosure.

   Resolution: Resolved after review. Replaced host-wide cookies with URL
   capability parameterization (`?token=<token>`). Injected `?token` into
   generated local document, diagram, stylesheet, script, and revision request
   URLs, and added `Referrer-Policy: no-referrer` to all responses and HTML
   metadata. Added shared-browser-context multi-session tests and cross-port
   request capture tests proving isolation and preventing credential disclosure.

2. [Medium] The expanded endpoint path breaks the native macOS test suite — [`src/service/client.rs:573`](../../src/service/client.rs#L573)

   Explanation and impact: `TestRuntime::new` combines the platform temporary
   directory with a long scenario name and then supplies it as
   `XDG_RUNTIME_DIR`. The new endpoint layout appends `/lens/service-v1.sock`.
   On macOS the resulting Unix socket pathname exceeds the supported limit.
   The [current head's native macOS job](https://github.com/sirius-cc-wu/lens/actions/runs/34093432686/job/101651636504)
   fails in `concurrent_first_commands_then_one_service_accepts_both_requests`
   with `path must be shorter than libc::sockaddr_un.sun_path`. The resulting
   poisoned test mutex causes five additional failures: 111 tests pass and six
   fail, and the later native validation steps cannot run. The older successful
   CI run quoted in the PR description does not validate this head.

   Reported behavior and impact:

   ```plantuml
   @startuml
   title macOS fixture exceeds Unix socket pathname capacity
   participant "TestRuntime" as Fixture
   participant "Endpoint path builder" as Endpoint
   participant "Unix socket API" as Socket
   participant "Native macOS CI" as CI
   Fixture -> Endpoint : long temporary path plus scenario name
   Endpoint -> Socket : append /lens/service-v1.sock and connect
   Socket --> Fixture : InvalidInput; pathname too long
   Fixture --> CI : first test panics; mutex becomes poisoned
   CI -> CI : five more tests fail; verification stops
   @enduml
   ```

   Proposed fix: Use a compact unique runtime directory name, keeping verbose
   scenario labels out of the socket pathname. Check the complete encoded
   socket path against the platform limit when constructing test fixtures;
   retain the private-directory permission and ownership checks.

   Suggested solution:

   ```plantuml
   @startuml
   title Keep test socket paths within native limits
   participant "TestRuntime" as Fixture
   participant "Endpoint path builder" as Endpoint
   participant "Unix socket API" as Socket
   Fixture -> Fixture : choose compact unique runtime name
   Fixture -> Endpoint : build complete socket pathname
   Endpoint --> Fixture : validate native pathname length
   Fixture -> Socket : connect using bounded pathname
   Socket --> Fixture : connection succeeds
   @enduml
   ```

   Test coverage: Re-run the native macOS job at the corrected head. Exercise
   fixture construction with a normal long macOS temporary-directory prefix
   and the longest scenario label, asserting the final pathname fits before
   starting any service.

   Resolution: Resolved after review. Updated `TestRuntime::new` in
   [`src/service/client.rs`](../../src/service/client.rs) to use a compact
   `lc-{pid}-{seq}` naming convention, keeping verbose scenario labels out of
   the runtime directory path. Added native sockaddr length assertions and a
   simulated deep macOS temporary path unit test verifying socket paths fit
   under the 104-byte limit.

3. [Medium] Concurrent first commands can fail while creating the runtime directory — [`src/service/endpoint/unix.rs:91`](../../src/service/endpoint/unix.rs#L91)

   Explanation and impact: Directory preparation first observes that the path
   is missing, then treats every `create` error as fatal. Two first commands
   can both observe absence; one creates the private `lens` directory and the
   other receives `AlreadyExists`. This helper also runs on the client's first
   `connect`, and `connect_or_start` does not classify `AlreadyExists` as
   retryable, so an otherwise valid invocation exits without a view while the
   other invocation starts normally. This affects ordinary XDG startup as well
   as first creation of the fallback directory.

   Reproduction: A temporary Rust harness imports the actual endpoint module
   and synchronizes 16 claims against an absent application directory. Across
   100 rounds it observed 434 directory-creation `AlreadyExists` errors. No
   production implementation was copied or modified.

   Reported behavior and impact:

   ```plantuml
   @startuml
   title Concurrent directory creation rejects a valid command
   participant "Client A" as A
   participant "Client B" as B
   participant Filesystem
   A -> Filesystem : inspect runtime directory
   Filesystem --> A : missing
   B -> Filesystem : inspect runtime directory
   Filesystem --> B : missing
   A -> Filesystem : create private directory
   B -> Filesystem : create same directory
   Filesystem --> B : AlreadyExists
   B -> B : abort command before service handoff
   @enduml
   ```

   Proposed fix: Accept `AlreadyExists` from this specific directory-creation
   operation and continue through the existing type, owner, and mode checks.
   Preserve rejection of a symlink, foreign-owned directory, or unsafe modes;
   do not simply return success when a competing creation wins.

   Suggested solution:

   ```plantuml
   @startuml
   title Validate a directory created by another contender
   participant Client
   participant Filesystem
   Client -> Filesystem : create private runtime directory
   Filesystem --> Client : created or AlreadyExists
   Client -> Filesystem : inspect type, owner, and permissions
   alt directory is safe
     Client -> Client : continue connection and service startup
   else directory is unsafe
     Client -> Client : return typed security error
   end
   @enduml
   ```

   Test coverage: Synchronize directory creation from independent threads or
   processes before the application directory exists. Require every safe
   contender to proceed to endpoint election, and retain negative tests for
   unsafe competing entries. The existing concurrent client test uses a
   single-thread runtime and does not interleave the synchronous directory
   inspection and creation operations.

   Resolution: Resolved after review. Updated `prepare_runtime_directory` in
   [`src/service/endpoint/unix.rs`](../../src/service/endpoint/unix.rs) to accept
   `std::io::ErrorKind::AlreadyExists` on directory creation and continue
   safely through the existing directory type, UID ownership, and `0700`
   permission validation. Added a concurrent 16-contender directory creation test.

4. [Medium] A starting Unix listener can be mistaken for a stale socket — [`src/service/endpoint/unix.rs:182`](../../src/service/endpoint/unix.rs#L182)

   Explanation and impact: A failed connection plus an unchanged owned socket
   inode does not establish that the owner has exited. Tokio's pinned Mio
   implementation calls `bind` and `listen` separately. If candidate A is
   descheduled after `bind`, candidate B sees its socket, gets connection
   refused, passes both metadata checks, unlinks A's live socket, and binds a
   replacement. A resumes and also returns a successful listener. Both service
   processes report readiness, but only B's socket is discoverable. A also
   records B's pathname inode at the end of `claim_at`, so its later cleanup can
   remove B's endpoint. This violates single-owner startup and can leave an
   undiscoverable background process.

   Reproduction: Launch two actual compiled service candidates in a dedicated
   private runtime directory. A test-only native interposer delays Unix
   `listen` by 300 ms after the actual bind; launch B once A's socket appears.
   Both candidates remain alive and print readiness with no errors. The delay
   exposes a valid scheduling interleaving; it does not change bind, connect,
   metadata, or unlink results. An uncontrolled 500-round stale-claim stress
   test did not reproduce duplicate ownership, so this is controlled ordering
   evidence, not a frequency estimate.

   Reported behavior and impact:

   ```plantuml
   @startuml
   title Starting listener is removed as if its owner had exited
   participant "Candidate A" as A
   participant "Socket pathname" as Path
   participant "Candidate B" as B
   A -> Path : bind; pause before listen
   B -> Path : connect
   Path --> B : connection refused
   B -> Path : verify inode; unlink; bind replacement
   B -> B : listen and report ready
   A -> A : resume listen on unlinked socket
   A -> Path : record replacement inode
   A -> A : also report ready
   @enduml
   ```

   Proposed fix: Establish exclusive ownership with a separate stable,
   user-private OS lock before stale-socket inspection, unlinking, binding, and
   listening. Hold the lock for the listener's lifetime and release it only
   after endpoint cleanup. Losing candidates should return `AlreadyOwned`;
   process exit should release ownership so a later winner can safely recover
   stale state. Further inode checks alone cannot distinguish the bind/listen
   interval from a dead owner.

   Suggested solution:

   ```plantuml
   @startuml
   title Protect startup and cleanup with lifetime ownership
   participant "Candidate A" as A
   participant "Private OS ownership lock" as Lock
   participant "Candidate B" as B
   participant "Socket pathname" as Path
   A -> Lock : acquire exclusive lifetime lock
   A -> Path : inspect stale state; bind; listen
   B -> Lock : try exclusive acquisition
   Lock --> B : AlreadyOwned
   B -> B : exit without touching socket
   A -> Path : remove owned socket on orderly shutdown
   A -> Lock : release ownership
   @enduml
   ```

   Test coverage: Control the interval between bind and listen and assert
   exactly one successful owner and continued discovery after the loser exits.
   Also test simultaneous recovery after a crash. Update the client stale
   fixture at [`src/service/client.rs:422`](../../src/service/client.rs#L422):
   it still puts `service-v1.sock` in the runtime root, while production now
   connects below `lens/`, so that test currently exercises ordinary startup
   rather than stale recovery.

   Resolution: Resolved after review. Implemented user-private advisory file locking
   (`libc::flock` with `LOCK_EX | LOCK_NB` on a companion `.lock` file) in
   [`src/service/endpoint/unix.rs`](../../src/service/endpoint/unix.rs) to serialize
   stale-socket recovery and socket binding. The listener retains the lock file
   for its process lifetime, safely unlinking the socket file before releasing
   ownership on shutdown, and releasing lock ownership upon process crash.
   Contenders receive `EndpointError::AlreadyOwned` before touching socket paths.
   Corrected the stale test fixture path in `src/service/client.rs`.

## Verification and Limits

- `cargo fmt --check` passed.
- `cargo test --locked` passed 117 library tests and five CLI tests.
- `npm run test:browser -- --reporter=line` passed all 28 existing scenarios.
- `cargo +1.76.0 clippy --locked --all-targets --all-features -- -D warnings`
  passed. The same command on installed Rust 1.97.1 fails on the pre-existing
  `manual_div_ceil` warning in `src/plantuml.rs:32`; the PR does not change that
  expression. Rust 1.75 was not installed locally.
- The current head's [CI run](https://github.com/sirius-cc-wu/lens/actions/runs/34093432686)
  passes Linux, Windows, and compiled-browser jobs and fails macOS as described
  in finding 2. No CI rerun or PR comment was requested or posted.
- Browser evidence uses Playwright's isolated Chrome context because the
  Chrome DevTools connector is unavailable. Browser and race fixtures were
  temporary; all test-owned service processes were stopped afterward.
- Reviewed the changed production modules and their target resolution,
  rendering, refresh, browser-launch, protocol, process, native endpoint,
  controller, public-library, test, and documentation dependencies. The
  existing public foreground API remains supported; the background API is now
  explicitly documented as internal.
- Process-lifetime session and request retention are explicit accepted scope,
  tracked by improvement 20; their unbounded resource growth is not repeated
  as a new finding. Native adversarial Windows execution and macOS desktop
  browser handoff were not performed in this Linux environment.
- `git diff --check` passed for the reviewed range and the staged review record.
- All eight new PlantUML blocks returned HTTP 200 and non-empty SVG from the
  configured default server, `https://www.plantuml.com/plantuml`, with no
  `x-plantuml-diagram-error` response header. All six local review links resolve,
  including their referenced source lines.

## Resolution Validation

- All four findings were resolved and verified across the codebase.
- `cargo fmt --check` passed cleanly.
- `cargo test --locked` passed all 123 library tests and 5 CLI integration tests.
- `cargo +1.76.0 clippy --locked --all-targets --all-features -- -D warnings` passed with 0 warnings.
- `cargo package --locked --allow-dirty` built and verified the package cleanly.
- `npm run test:browser -- --reporter=line` passed all 30 browser scenarios.
