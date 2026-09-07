# PR 13: Current Head Review

Reviewed on 2026-09-07. Reviewed PR head
`c53ca2d03528c0af7a0a2adcdfd50966d7c5d175` against merge base
`1c6cd2d5e620991b3968dd39150c85e9ee1c4c7b` on
`feat/background-viewer-service`.

## Result

No remaining actionable findings were confirmed in the accumulated change.
The previous review records describe historical findings; their fixes are
included in this review's head. In particular, the CLI integration fixture now
uses a compact runtime directory, checks the complete Unix socket pathname,
and captures service stderr when readiness fails.

Reviewed the CLI and public foreground entry point, browser-launch extraction,
native process startup, Unix endpoint ownership and stale-socket recovery,
Windows named-pipe access policy and server identity checks, protocol framing
and native paths, request deduplication and session retention, target discovery,
source-link boundaries, viewer authorization, navigation, diagram retries,
refresh, tests, CI configuration, and the associated contracts and guidance.

## Validation

- `cargo fmt --check` passed.
- `cargo test --locked` passed: 123 library tests, six CLI integration tests,
  and zero documentation tests on Linux.
- `cargo +1.76.0 clippy --locked --all-targets --all-features -- -D warnings`
  passed. Rust 1.75, used by CI, is not installed locally.
- The same Clippy command with the default Rust 1.97 toolchain failed on
  `manual_div_ceil` in unchanged `src/plantuml.rs:32` and
  `suspicious_open_options` at `src/service/endpoint/unix.rs:147`. This review
  does not claim a clean default-toolchain lint run. The lock file is not used
  for storing content; explicit `.truncate(false)` would clarify its intent
  for the newer lint.
- `npm run test:browser -- --reporter=line` passed all 30 scenarios using
  isolated headless Chrome through Playwright. Chrome DevTools MCP was not
  available in this session.
- An additional temporary-fixture check exercised the compiled ordinary CLI
  with no existing endpoint. It cold-started the service, exited successfully,
  reused the same service PID on a second invocation, returned distinct working
  URLs, and refreshed both documents after the clients exited. Browser launch
  used a controlled `xdg-open` stub. The test service was terminated and the
  temporary fixture removed afterward.
- `git diff --check` passed for the reviewed range.
- In the [reviewed head's CI run](https://github.com/sirius-cc-wu/lens/actions/runs/34096830684),
  Linux, Windows, and compiled-browser jobs had passed; the macOS job was still
  pending when the review record was written. The prior macOS failure must not
  be considered verified as resolved until that native job passes.

## Residual Risks and Limits

Sessions and completed request outcomes remain retained for the service
lifetime, and a service crash invalidates all its URLs. These are documented,
accepted lifecycle limitations; large repositories and sustained session growth
were not newly benchmarked in this review.

Native macOS execution, adversarial Windows execution, actual desktop browser
handoff, and downloaded release archives were not exercised locally. Packaging
was not repeated locally; the native CI jobs include that check. This record
adds no PlantUML diagrams because no actionable findings were confirmed.
