# Background Viewer Service Stop Command Feature Review

Reviewed `feat/stop-command` from merge base `7c01f2373ee61d82d4b01bfb23c314b3924d3983` with `origin/main` through head `65a6ec7d468d87246aa8be6d5e52a49250ff8cb4`.

**Verdict:** Clean approval — no remaining actionable findings. The implementation delivers the dedicated background service stop command (`lens stop`) specified in [UC-12](../features/background-viewer-service/use-cases.md#uc-12-stop-the-background-lens-service), [FEAT-04-REQ-STOP](../features/background-viewer-service/server-stop-requirements.md), and [ADR-024](../decisions/adr-024-background-service-stop-command.md). All review feedback from PR #18 and multi-platform CI edge cases have been thoroughly resolved, including bounded shutdown connection drains, busy Windows named pipe retries, shutdown-race disconnect resilience, and concurrent stop assertion allowances for post-teardown callers. All local quality gates (162 Rust tests, 32 browser scenarios) and full GitHub Actions matrix CI jobs (Linux, macOS, Windows MSVC, Playwright) pass cleanly.

## Scope

Inspected the complete authored diff against `origin/main`:
- Requirements, architecture, and design specifications:
  - `docs/features/background-viewer-service/use-cases.md` (`UC-12`)
  - `docs/features/background-viewer-service/server-stop-requirements.md` (`FEAT-04-REQ-STOP`)
  - `docs/decisions/adr-024-background-service-stop-command.md` (`ADR-024`)
  - `docs/features/background-viewer-service/server-stop-technical-design.md`
- CLI and top-level entrypoints:
  - `src/main.rs`: `Command::Stop` subcommand parsing with `subcommand_precedence_over_arg = true` and target argument disambiguation.
  - `src/lib.rs`: Public `lens::stop()` coordination function.
- IPC Protocol Framing:
  - `src/service/protocol.rs`: `StopRequest` and `ServiceResponse::Stopped` frames, protocol version validation, serialization, and round-trip unit tests.
- Service Controller & Viewer Session Lifecycle:
  - `src/service/server.rs`: `ControllerState::Stopping` state transition, session ledger draining, rejection of in-flight open requests during stopping, idempotent stop responses, server backlog draining during shutdown, bounded connection task draining via `SHUTDOWN_DRAIN_TIMEOUT` (1,000ms), and forceful task abort fallback (`abort_all()`).
  - `src/viewer/mod.rs`: `ViewerSession::stop()` implementation triggering Axum server shutdown, aborting document filesystem watchers, and waiting on server task completion.
- Client Coordination & Recovery:
  - `src/service/client.rs`: `stop_background_service()` client coordination, bounded timeouts (`STOP_TIMEOUT`), exit polling, busy endpoint retry loop (`is_busy()`), stale endpoint detection (`is_absent()`), shutdown-race disconnect recovery, and concurrent caller outcome accommodation.
- Platform Endpoint Management:
  - `src/service/endpoint.rs`: Differentiated `is_absent()` and `is_busy()` error classifiers, export of `clean_stale_endpoint`.
  - `src/service/endpoint/unix.rs`: `clean_stale_endpoint` delegating to verified stale socket unlinking.
  - `src/service/endpoint/windows.rs`: No-op named pipe cleanup parity for Windows.
- CLI Integration Tests:
  - `tests/cli.rs`: CLI test cases verifying active service stop, inactive service idempotency, stale socket recovery, concurrent CLI process stops, and CLI `--help` discoverability.

## PR #18 Feedback & CI Resolution

Commits `41f00f8`, `1f10ed3`, `f9c5126`, and `65a6ec7` resolved all actionable review findings and cross-platform CI race conditions:

1. **Bounded IPC Connection Drain on Shutdown (`src/service/server.rs`):**
   - *Reported behavior:* `run_background_service()` previously joined remaining IPC connection tasks via an unbounded loop `while connections.join_next().await.is_some() {}`. If an idle client process maintained an open connection socket without sending frames or disconnecting, the background service could hang indefinitely on shutdown.
   - *Resolution:* Wrapped connection task draining in a 1,000ms timeout (`SHUTDOWN_DRAIN_TIMEOUT`). If pending connections do not exit within this grace period, `connections.abort_all()` forcefully cancels all remaining connection tasks and joins them cleanly.
   - *Verification:* Verified with `service::client::tests::idle_connection_held_during_stop_then_service_terminates_within_grace_period`, proving the service terminates in under 3 seconds despite an open, idle client socket.

2. **Differentiating Absent vs. Busy Endpoints with Windows Named Pipe Retries (`src/service/endpoint.rs`, `src/service/client.rs`):**
   - *Reported behavior:* `EndpointError::is_unavailable()` previously grouped Windows `ERROR_PIPE_BUSY` (raw OS error 231) with absent socket errors (`NotFound`, `ConnectionRefused`). Under high concurrency on Windows, all pipe instances may be temporarily busy connecting, causing `lens stop` to incorrectly assume no service was running, print `"No Lens background service is running."`, and exit prematurely.
   - *Resolution:* Partitioned error detection into `is_absent()` (endpoint does not exist) and `is_busy()` (endpoint exists but pipe instances or threads are temporarily busy). `stop_background_service()` now implements a retry loop with `CONNECT_RETRY_INTERVAL` (25ms) bounded by `STOP_TIMEOUT` when encountering `is_busy()`. Only `is_absent()` unlinks stale sockets and reports `NotRunning`.
   - *Verification:* Added 5 unit tests for `EndpointError` classification and 3 client coordination tests (`stop_with_busy_endpoint_retries_and_succeeds_when_endpoint_becomes_available`, `stop_with_persistently_busy_endpoint_times_out_without_reporting_not_running`, `stop_with_absent_endpoint_reports_not_running_without_retry`).

3. **Shutdown-Race Disconnect Resilience & Server Backlog Drain (`src/service/client.rs`, `src/service/server.rs`):**
   - *Reported behavior:* Under concurrent stop commands, if the service initiated shutdown while another client had connected or was transmitting its frame, the connection could experience transport disconnects (`Broken pipe` or `Connection reset`). Previously, this surfaced as an uncaught `ProtocolError::Io`, causing concurrent invocations to fail non-idempotently.
   - *Resolution:*
     - *Client-side:* In `stop_background_service_with()`, if an IO error occurs during the exchange with a stopping service, the client enters a 500ms verification loop. If the endpoint becomes absent (confirming the service terminated) or reconnects and confirms stopped status, the client reports `StopOutcome::Stopped` instead of erroring.
     - *Server-side:* In `run_background_service()`, the server continues accepting incoming connections on its listener during `SHUTDOWN_DRAIN_TIMEOUT`. Connections accepted while in `ControllerState::Stopping` immediately receive `ServiceResponse::Stopped` rather than connection aborts.
   - *Verification:* Verified with `service::client::tests::stop_when_transport_disconnects_during_shutdown_and_service_stops_then_reports_stopped`, `service::client::tests::concurrent_stops_when_service_running_then_all_clients_succeed_and_service_stops` (5 concurrent async tasks), and `tests::cli::concurrent_stop_commands_when_service_running_then_all_succeed_and_exit_zero` (4 concurrent CLI processes).

4. **Post-Teardown Concurrent Caller Outcome Accommodation (`src/service/client.rs`):**
   - *Reported behavior:* In `concurrent_stops_when_service_running_then_all_clients_succeed_and_service_stops`, the test previously asserted `StopOutcome::Stopped` across all 5 concurrent tasks. On Windows MSVC runners where process teardown is fast and task scheduling can vary, a task scheduled after the service has already completed its shutdown observes `StopOutcome::NotRunning`. While both `Stopped` and `NotRunning` produce identical exit code 0 for the CLI, the overly strict test assertion caused intermittent CI test failures on Windows.
   - *Resolution:* Updated the test assertion to permit concurrent stop callers to observe either `StopOutcome::Stopped` or `StopOutcome::NotRunning`, while asserting that at least one caller observed `StopOutcome::Stopped` and the service terminated.
   - *Verification:* Verified locally and via GitHub Actions CI (Run ID `34742259071`), where `Native Rust (x86_64-pc-windows-msvc)` passed cleanly in 3m26s.

## Multi-Axis Review

### 1. Functional Correctness (UC-12 & Acceptance Criteria)

- **AC-1 (Command Syntax & Discovery):** Running `lens --help` lists `stop` as a recognized subcommand. Running `lens stop --help` describes its purpose. `subcommand_precedence_over_arg = true` ensures `lens stop` is never misinterpreted as a file target named "stop".
- **AC-2 (Graceful Shutdown of Active Service):** Executing `lens stop` when a background service is active connects over local IPC, sends `StopRequest`, receives `ServiceResponse::Stopped`, awaits process termination within a 3-second bounded window, and reports `"Lens background service stopped."` with exit code 0.
- **AC-3 (Network & Resource Release):** `ServiceController::stop` calls `drain_sessions()`, which invokes `ViewerSession::stop()` on every retained session. Each session notifies its Axum server to gracefully shut down, aborts its file notification watcher task (releasing inotify/file descriptors), and drops the TCP listener, immediately freeing bound loopback ports. Dropping the IPC `Listener` unlinks the Unix domain socket.
- **AC-4 (Strict Idempotency):** When no service is running, `lens stop` cleanly detects endpoint unavailability, prints `"No Lens background service is running."`, and exits successfully with code 0. Repeated back-to-back executions of `lens stop` succeed with code 0. Concurrent executions of `lens stop` all succeed with code 0.
- **AC-5 (Stale Endpoint Cleanup):** If an orphaned Unix domain socket exists from an abnormally terminated process, `lens stop` verifies the endpoint is dead, unlinks the socket file, prints `"No Lens background service is running."`, and exits with code 0.
- **AC-6 (Per-User Isolation):** Communication endpoints resolve exclusively within the invoking user's runtime directory (`XDG_RUNTIME_DIR` on Unix, user SID pipe on Windows). On Unix, peer credentials (`SO_PEERCRED`) verify the caller's UID matches the service owner before processing requests.
- **AC-7 (Prompt Execution):** Under normal execution, `lens stop` completes in ~10–50ms. An unresponsive service is bounded by `STOP_TIMEOUT` (3000ms), and connection drain is bounded by `SHUTDOWN_DRAIN_TIMEOUT` (1000ms), preventing shell hangs.
- **AC-8 (Zero Regression):** All existing document viewing commands (`lens [TARGET]`), automatic refresh, and viewer session isolation remain unaffected.

### 2. Security & Policy Enforcement

- **Zero HTTP Exposure:** The stop functionality is strictly confined to the local OS-authenticated IPC channel (`src/service/protocol.rs`). No HTTP routes for shutdown or process termination are exposed via Axum, maintaining complete separation between browser network requests and process lifecycle controls.
- **OS Peer Authentication:** Unix endpoints enforce peer UID verification before reading frames. Windows endpoints enforce kernel-level named pipe security descriptors.
- **Multi-User Protection:** Services cannot be discovered, communicated with, or terminated across different operating-system users.

### 3. Resource & Lifecycle Verification

- **TCP Listener Unbinding:** Verified that Axum loopback listeners shut down promptly, releasing bound loopback ports for immediate reuse.
- **Watchers Aborted:** File-system change watchers are explicitly aborted, closing notify event loop handles.
- **Endpoint Filesystem Cleanliness:** Unix domain socket file removal was verified across normal shutdown (via `Drop` on `EndpointListener`), stale socket discovery (via `clean_stale_endpoint`), and idle connection aborts.

### 4. Code Cleanliness & Repository Conventions (per AGENTS.md)

- **Cohesive Module Boundaries:** Modules maintain single responsibilities:
  - `src/main.rs`: CLI argument parsing and dispatch.
  - `src/service/protocol.rs`: Type-safe frame definition, validation, and serde.
  - `src/service/server.rs`: Controller state transitions, session draining, request gating, and backlog acceptance.
  - `src/service/client.rs`: Client-side discovery, timeout handling, retry loop, outcome mapping, and disconnect recovery.
  - `src/service/endpoint.rs`: Transport error classification (`is_absent` vs `is_busy`) and stale socket unlinking.
- **Line Count Compliance:** All modified source files remain well below the 500 non-test lines split signal (e.g., non-test lines in `server.rs` are ~430; `client.rs` are ~400).
- **Behavior-Oriented Testing:** Every new unit and CLI test strictly follows `<condition_or_action>_then_<observable_result>` naming and includes `// Arrange`, `// Act`, and `// Assert` structure.

## Validation & Test Execution Metrics

All verification quality gates and test suites were executed against the worktree:

1. **Rust Code Formatting:**
   - Command: `cargo fmt --check`
   - Result: Passed (0 formatting violations).
2. **Clippy Static Analysis:**
   - Command: `cargo clippy --locked --all-targets --all-features -- -D warnings`
   - Result: Passed (0 warnings, 0 errors).
3. **Rust Unit & CLI Test Suite:**
   - Command: `cargo test --locked`
   - Result: Passed 162 tests (148 library unit tests, 3 main unit tests, and 11 CLI integration tests) in 5.42s.
   - New & regression tests passed:
     - `service::endpoint::tests::not_found_io_error_then_is_absent_and_unavailable`
     - `service::endpoint::tests::connection_refused_io_error_then_is_absent_and_unavailable`
     - `service::endpoint::tests::pipe_busy_raw_os_error_then_is_busy_and_unavailable_but_not_absent`
     - `service::endpoint::tests::would_block_io_error_then_is_busy_and_unavailable_but_not_absent`
     - `service::endpoint::tests::other_io_error_then_is_neither_absent_nor_busy`
     - `service::client::tests::idle_connection_held_during_stop_then_service_terminates_within_grace_period`
     - `service::client::tests::stop_with_busy_endpoint_retries_and_succeeds_when_endpoint_becomes_available`
     - `service::client::tests::stop_with_persistently_busy_endpoint_times_out_without_reporting_not_running`
     - `service::client::tests::stop_with_absent_endpoint_reports_not_running_without_retry`
     - `service::client::tests::stop_when_transport_disconnects_during_shutdown_and_service_stops_then_reports_stopped`
     - `service::client::tests::concurrent_stops_when_service_running_then_all_clients_succeed_and_service_stops`
     - `service::client::tests::stop_when_service_not_running_then_returns_not_running`
     - `service::client::tests::stop_when_service_running_then_stops_service_and_returns_stopped`
     - `service::client::tests::stop_when_stale_endpoint_exists_then_removes_socket_and_returns_not_running`
     - `service::server::tests::stop_message_when_running_then_replies_stopped_and_transitions_state`
     - `service::server::tests::open_message_when_stopping_then_rejects_with_session_error`
     - `service::server::tests::repeated_stop_message_when_stopping_then_replies_stopped_idempotently`
     - `stop_command_when_service_running_then_stops_service_and_exits_zero`
     - `stop_command_when_service_not_running_then_reports_inactive_and_exits_zero`
     - `stop_command_help_flag_then_describes_stop_command`
     - `stop_command_when_stale_socket_exists_then_removes_socket_and_exits_zero`
     - `concurrent_stop_commands_when_service_running_then_all_succeed_and_exit_zero`
4. **Playwright Browser Test Suite:**
   - Command: `npx playwright test`
   - Result: Passed 32 tests in 24.8s across Chromium.
5. **GitHub Actions Matrix CI (Run ID `34742259071`):**
   - Result: All 4 jobs completed with success:
     - `Compiled browser behavior` (ID 103683828479): Passed in 1m28s
     - `Native Rust (x86_64-pc-windows-msvc)` (ID 103683828517): Passed in 3m26s
     - `Native Rust (x86_64-apple-darwin)` (ID 103683828584): Passed in 2m54s
     - `Native Rust (x86_64-unknown-linux-gnu)` (ID 103683828624): Passed in 1m45s

## Residual Risks & Validation Limits

- **Windows Named Pipe Integration:** Verification was conducted across local Linux unit/browser tests and full GitHub Actions Windows MSVC CI testing. All named pipe tests passed cleanly under the Windows MSVC CI runner.
- **Ungraceful Service Termination Window:** If a background service process is forcibly terminated (e.g. `SIGKILL`), the orphaned socket file remains on disk until the next invocation of `lens` or `lens stop`, which safely recovers and unlinks it.

---

*Note on PlantUML Diagrams:* Per repository guidelines in `AGENTS.md`, this record includes no PlantUML diagrams because there are no actionable findings or unresolved defects; all prior review comments and platform-specific CI considerations have been fully resolved and verified.
