# PR 13: Background Viewer Service Revalidation

Reviewed on 2026-09-07. Scope: merge base
`1c6cd2d5e620991b3968dd39150c85e9ee1c4c7b` through PR head
`70e5a36b39454bbccf75dbc1f17527fa9bf387b2` on
`feat/background-viewer-service`. This review covers the accumulated change
against `main`, including the fixes recorded in the two previous reviews.

Resolution status: The finding was resolved after review and passed the
resolution validation recorded below.

## Finding

1. [Medium] Shorten the CLI service fixture's socket path as well — [`tests/cli.rs:127`](../../tests/cli.rs#L127)

   Explanation and impact: `BackgroundService::start` still uses
   `unique_path(name)` for `XDG_RUNTIME_DIR`, retaining the verbose
   `lens-cli-<pid>-empty-directory-service` suffix. The service appends
   `/lens/service-v1.sock`. With a representative macOS temporary directory,
   `/var/folders/zz/zyxvpxvq6csfxvn_n0000000000000/T`, and a five-digit PID,
   the resulting pathname is 108 bytes and exceeds the native socket pathname
   capacity. The compact naming fix in the client unit-test fixture does not
   cover this independent integration-test fixture.

   The [reviewed head's macOS job](https://github.com/sirius-cc-wu/lens/actions/runs/34095586040/job/101658406510)
   passes all 123 library tests, then fails
   `empty_current_directory_then_reports_no_documents_error`: the child exits
   without a readiness line, so the assertion at line 157 sees an empty string.
   The poisoned mutex subsequently fails
   `missing_target_then_reports_actionable_error`. Native Clippy and package
   verification are consequently skipped. The previous review's resolution
   statement does not establish a passing macOS integration suite.

   Reproduction: Launching the actual compiled service in a temporary private
   runtime directory with a 124-byte final socket pathname on Linux exits with
   status 1, no stdout, and `Could not claim the Lens command endpoint: path
   must be shorter than libc::sockaddr_un.sun_path`. This confirms the failure
   mechanism; the linked native job supplies the macOS execution evidence.

   Reported behavior and impact:

   ```plantuml
   @startuml
   title CLI fixture still exceeds the native socket path limit
   participant "CLI integration fixture" as Fixture
   participant "Background service" as Service
   participant "Unix socket API" as Socket
   participant "macOS CI" as CI
   Fixture -> Service : XDG_RUNTIME_DIR with verbose scenario suffix
   Service -> Socket : bind runtime/lens/service-v1.sock
   Socket --> Service : pathname too long
   Service --> Fixture : exit before readiness; empty stdout
   Fixture --> CI : assertion fails and mutex is poisoned
   CI -> CI : second test fails; later checks are skipped
   @enduml
   ```

   Proposed fix: Give the service runtime fixture a compact unique name,
   independent of the descriptive document fixture names. Validate the complete
   encoded socket pathname before spawning, using the native platform's limit
   and allowing room for its terminator. Keep private-directory permissions and
   ownership checks. Include captured child stderr in startup failures so a
   native endpoint error is not reduced to an empty-readiness assertion.

   Suggested solution:

   ```plantuml
   @startuml
   title Bound the CLI fixture socket pathname before spawning
   participant "CLI integration fixture" as Fixture
   participant "Background service" as Service
   participant "Unix socket API" as Socket
   participant "macOS CI" as CI
   Fixture -> Fixture : choose compact unique runtime directory
   Fixture -> Fixture : validate complete encoded socket pathname
   Fixture -> Service : spawn with private runtime directory
   Service -> Socket : bind pathname within native limit
   Socket --> Service : listener ready
   Service --> Fixture : readiness line
   Fixture --> CI : execute target-error assertions
   @enduml
   ```

   Test coverage: Exercise the actual CLI runtime-path helper with a normal
   macOS temporary prefix and the longest scenario label. Assert that the final
   encoded socket pathname fits, then re-run the full native macOS job,
   including `tests/cli.rs`, Clippy, and packaging. The current simulated-path
   unit test only verifies the separate `lc-<pid>-<sequence>` naming scheme.

   Resolution: Resolved after review. Updated `BackgroundService::start` in
   [`tests/cli.rs`](../../tests/cli.rs) to use a compact `lcr-{pid}-{seq}`
   runtime directory scheme, independent of document fixture names. Added
   pre-spawn sockaddr path length assertions and concurrent child stderr capture
   on readiness failure. Added `cli_fixture_socket_path_then_fits_within_unix_sun_path_limit`
   exercising simulated deep macOS temporary directories.

## Verification and Limits

- `cargo fmt --check` passed.
- `cargo test --locked` passed: 123 library tests, five CLI tests, and zero
  documentation tests on Linux.
- `cargo +1.76.0 clippy --locked --all-targets --all-features -- -D warnings`
  passed. Local validation used the installed toolchains; the native CI jobs
  use Rust 1.75.
- `npm run test:browser -- --reporter=line` passed all 30 scenarios in isolated
  headless Chrome through the repository's Playwright configuration. This
  includes shared-context independent navigation and refresh, rejected
  cross-session credentials, and absence of cookies on another local port.
- At the reviewed head, Linux, Windows, and compiled-browser CI passed;
  macOS failed as described above. Native desktop handoff and adversarial
  Windows execution were not performed locally.
- Traced the client, protocol, native endpoint ownership and authorization,
  process detachment, request ledger, retained viewer lifecycle, target
  resolution, rendering, source links, authentication, navigation, refresh,
  public foreground API, and their tests. No additional actionable finding
  was confirmed. Process-lifetime resource retention and service-crash session
  loss remain explicitly accepted lifecycle risks.
- The fixture reproduction used temporary directories and the actual compiled
  binary; the test process exited and its directories were removed. No
  production code was changed.
- `git diff --check` passed for the reviewed range.
- Both PlantUML blocks returned HTTP 200 and SVG from the configured default
  server, `https://www.plantuml.com/plantuml`, with no
  `x-plantuml-diagram-error` header. The first compressed-URL request received
  HTTP 403; validation succeeded using the server's hexadecimal source format.
  The review's local source link and referenced line were checked.

## Resolution Validation

- The finding was resolved and verified across the codebase.
- `cargo fmt --check` passed cleanly.
- `cargo test --locked` passed all 123 library tests and 6 CLI integration tests.
- `cargo +1.76.0 clippy --locked --all-targets --all-features -- -D warnings` passed with 0 warnings.
- `cargo package --locked --allow-dirty` built and verified the package cleanly.
- `npm run test:browser -- --reporter=line` passed all 30 browser scenarios.
