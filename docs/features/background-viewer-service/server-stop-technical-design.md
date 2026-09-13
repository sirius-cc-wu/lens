---
type: "Technical Design"
title: "Background Service Stop Command Technical Design Specification"
description: "Architectural specification, IPC protocol extensions, controller lifecycle state machine, and resource cleanup for the 'lens stop' CLI command."
id: "FEAT-04-DESIGN-STOP"
feature: "FEAT-04"
status: "accepted"
language: "Rust"
tags: [design, specification, process-lifecycle, ipc, stop-command]
---

# Background Service Stop Command Technical Design Specification

## 1. Overview & System Context

The Lens background service runs as a detached, per-user background process coordinating ephemeral loopback HTTP servers and filesystem watchers across viewing sessions. While `lens [TARGET]` requests target views from this background service, Lens previously lacked a dedicated CLI command to stop the service when documentation review completes.

This technical design specifies the realization of `lens stop` (`UC-12`, `FEAT-04-REQ-STOP`, and `ADR-024`). The command provides a clean, prompt, and idempotent mechanism to:
1. Connect to the running background service via authenticated local IPC.
2. Signal shutdown using a typed, versioned protocol frame.
3. Drain in-flight HTTP requests, abort file-system watchers, and close loopback listeners.
4. Cleanly unlink the IPC endpoint file and terminate the background process.
5. Recover from stale orphaned sockets and exit with status code 0 when no service is active.

```
+-------------------------------------------------------------------------------+
|                               Invoking Terminal                               |
|                                                                               |
|  $ lens stop                                                                  |
|        |                                                                      |
|        v                                                                      |
|  +-------------+   IPC Connect     +---------------------------------------+  |
|  | Lens Client | ----------------> |        Background Lens Service        |  |
|  |             |                   |                                       |  |
|  |             |   StopRequest     |  +---------------------------------+  |  |
|  |             | ----------------> |  |        ServiceController        |  |  |
|  |             |                   |  | (Running -> Stopping -> Stopped)|  |  |
|  |             |   Stopped Ack     |  +---------------------------------+  |  |
|  |             | <---------------- |                  |                    |  |
|  |             |                   |                  v                    |  |
|  |             |                   |  +---------------------------------+  |  |
|  |  Exit(0)    |                   |  |      RequestLedger Sessions     |  |  |
|  |  "Stopped"  |                   |  |  - Drain in-flight HTTP        |  |  |
|  +-------------+                   |  |  - Abort file watchers         |  |  |
|                                    |  |  - Close TCP listeners         |  |  |
|                                    |  +---------------------------------+  |  |
|                                    |                  |                    |  |
|                                    |                  v                    |  |
|                                    |  +---------------------------------+  |  |
|                                    |  |        Endpoint Unlink          |  |  |
|                                    |  |  - Delete service-v1.sock      |  |  |
|                                    |  |  - Process Exit(0)             |  |  |
|                                    |  +---------------------------------+  |  |
|                                    +---------------------------------------+  |
+-------------------------------------------------------------------------------+
```

---

## 2. System & Component Architecture

The stop capability spans five cohesive modules in the `lens` crate:

```text
src/
├── main.rs                  # Parses CLI arguments; routes 'stop' subcommand
├── lib.rs                   # Public library entrypoint: lens::stop()
├── service/
│   ├── protocol.rs          # StopRequest and Stopped protocol frame definitions
│   ├── client.rs            # Client coordinator: discovery, stop request, timeout
│   ├── server.rs            # Service listener loop, controller state, session drain
│   └── endpoint/
│       ├── mod.rs           # Stale endpoint recovery interface
│       ├── unix.rs          # Unix domain socket unlinking and stale recovery
│       └── windows.rs       # Windows named pipe lifecycle
└── viewer/
    └── mod.rs               # ViewerSession graceful shutdown and resource cleanup
```

### Component Roles & Responsibilities

| Component | Responsibility |
| :--- | :--- |
| **CLI (`src/main.rs`)** | Distinguishes `lens stop` from `lens [TARGET]` using `clap` subcommand parsing; dispatches to `lens::stop()`. |
| **Public API (`src/lib.rs`)** | Exposes `pub async fn stop() -> anyhow::Result<()>`, formatting output messages and mapping client outcomes to process exit codes. |
| **Client Coordinator (`src/service/client.rs`)** | Connects to the local endpoint; does **not** auto-start a service; detects inactive/stale endpoints; dispatches `ServiceRequest::Stop`; enforces bounded timeouts. |
| **Protocol Framing (`src/service/protocol.rs`)** | Encodes and decodes `StopRequest` and `ServiceResponse::Stopped` frames bounded by `MAX_FRAME_BYTES` (64 KiB). |
| **Service Controller (`src/service/server.rs`)** | Owns lifecycle state (`Running` $\rightarrow$ `Stopping` $\rightarrow$ `Stopped`); rejects late `Open` requests; orchestrates session draining and process exit. |
| **Endpoint Platform (`src/service/endpoint/`)** | Manages socket/pipe ownership; deletes socket file on normal termination; provides verified unlinking for dead sockets. |
| **Viewer Session (`src/viewer/mod.rs`)** | Triggers graceful Axum HTTP shutdown via `shutdown_sender`; aborts document watcher tasks; closes loopback TCP ports. |

---

## 3. IPC Protocol Extensions

The local IPC protocol between the Lens client and background service is extended with typed stop framing:

### Protocol Framing Structure

All frames use the existing Lens binary length-prefix format:
- **Prefix:** 4-byte big-endian unsigned integer declaring payload byte count (maximum `MAX_FRAME_BYTES` = 65,536 bytes).
- **Payload:** UTF-8 encoded JSON object matching the `ServiceRequest` or `ServiceResponse` schema.

### Data Contracts (`src/service/protocol.rs`)

```rust
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct StopRequest {
    pub(crate) protocol_version: ProtocolVersion,
    pub(crate) request_id: RequestId,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", content = "body", rename_all = "snake_case")]
pub(crate) enum ServiceRequest {
    Open(OpenRequest),
    Stop(StopRequest),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", content = "body", rename_all = "snake_case")]
pub(crate) enum ServiceResponse {
    Ready {
        request_id: RequestId,
        view_url: String,
    },
    Rejected {
        request_id: RequestId,
        error: OpenError,
    },
    Stopped {
        request_id: RequestId,
    },
    Incompatible {
        supported_version: ProtocolVersion,
    },
}
```

### Version Validation Invariants

```rust
impl ServiceRequest {
    pub(crate) fn validate_version(&self) -> Result<(), ProtocolError> {
        let received = match self {
            Self::Open(request) => request.protocol_version,
            Self::Stop(request) => request.protocol_version,
        };
        if received == ProtocolVersion::CURRENT {
            Ok(())
        } else {
            Err(ProtocolError::IncompatibleVersion {
                received,
                supported: ProtocolVersion::CURRENT,
            })
        }
    }
}
```

---

## 4. Controller Lifecycle & State Machine

The `ServiceController` in `src/service/server.rs` acts as the single authoritative state owner for the background process.

### State Transition Diagram

```
                +-------------------------------------------------+
                |                     Running                     |
                | - Accepts Open requests                         |
                | - Tracks active sessions in RequestLedger       |
                +-------------------------------------------------+
                                         |
                                         | Receive StopRequest
                                         v
                +-------------------------------------------------+
                |                    Stopping                     |
                | - Rejects subsequent Open requests              |
                | - Returns Stopped to duplicate/concurrent Stop  |
                | - Triggers graceful drain of ViewerSessions     |
                | - Signals listener loop to cease accepting      |
                +-------------------------------------------------+
                                         |
                                         | Drain complete & listener dropped
                                         v
                +-------------------------------------------------+
                |                     Stopped                     |
                | - All TCP loopback ports closed                 |
                | - All notify watchers aborted                   |
                | - Endpoint unlinked                             |
                | - Process exits (status 0)                      |
                +-------------------------------------------------+
```

### Controller Implementation Design

```rust
enum ControllerState {
    Running,
    Stopping,
}

enum ControllerMessage {
    Open {
        request: OpenRequest,
        reply: oneshot::Sender<ServiceResponse>,
    },
    Stop {
        request: StopRequest,
        reply: oneshot::Sender<ServiceResponse>,
    },
    Complete {
        request_id: RequestId,
        completion: SessionCompletion,
    },
    #[cfg(test)]
    Stats {
        reply: oneshot::Sender<ControllerStats>,
    },
}
```

### State-Dependent Behavior Matrix

| Incoming Message | Controller State: `Running` | Controller State: `Stopping` |
| :--- | :--- | :--- |
| **`Open(request)`** | Enters `RequestLedger`; spawns `create_session()`; replies with `Ready` or `Rejected`. | Immediately replies with `ServiceResponse::Rejected { error: OpenError { code: OpenErrorCode::Session, message: "Lens background service is stopping" } }`. |
| **`Stop(request)`** | Transitions to `Stopping`; responds with `ServiceResponse::Stopped { request_id }`; initiates session shutdown and listener termination. | Idempotently responds with `ServiceResponse::Stopped { request_id }`. Does not duplicate drain tasks. |
| **`Complete`** | Completes in-flight session and stores `ViewerSession` handle in ledger. | If a session completes during stopping, it is immediately closed/dropped. |

### Graceful Session Draining Flow

When transitioning to `Stopping`:
1. **Acknowledge Stop:** Send `ServiceResponse::Stopped { request_id }` back over the active client connection so the invoking `lens stop` command receives confirmation promptly.
2. **Signal Axum Graceful Shutdown:** Iterate over all retained `ViewerSession` instances in `RequestLedger`. For each session:
   - Call `session.shutdown_sender.take().unwrap().send(())`.
   - Axum's `.with_graceful_shutdown()` drains active HTTP requests within a brief bounded grace window (1,000 ms).
   - Abort the associated `watcher_task` (`notify` file-system watcher).
3. **Close Listener & Release Ports:** When Axum tasks complete, the underlying `TcpListener` instances are dropped, returning ephemeral ports to the operating system immediately.
4. **Terminate Server Loop:** Signal the `run_background_service()` loop via a cancellation token or channel. The loop exits and drops `endpoint::Listener`, unlinking the socket file before process termination.

---

## 5. Endpoint Lifecycle & Stale Socket Recovery

### Normal Termination Cleanup

On Unix systems, `src/service/endpoint/unix.rs` implements `Drop` for `Listener`:

```rust
impl Drop for Listener {
    fn drop(&mut self) {
        let Ok(metadata) = fs::symlink_metadata(&self.path) else {
            return;
        };
        if metadata.file_type().is_socket()
            && metadata.uid() == effective_user_id()
            && metadata.dev() == self.device
            && metadata.ino() == self.inode
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}
```
When `run_background_service()` returns, `Listener` is dropped, removing `service-v1.sock` atomically.

On Windows, named pipes (`\\.\pipe\lens-{user_sid}\service-v1`) are operating-system kernel objects. Closing the server handle or terminating the process automatically removes the pipe instance from the kernel namespace with no filesystem artifact.

### Client-Side Stale Endpoint Detection & Recovery

If the background service process was terminated abruptly (e.g. `SIGKILL`, hardware power loss, or operating system crash), an orphaned socket file may remain on disk on Unix systems.

`src/service/client.rs` implements resilient recovery during `lens stop`:

```rust
pub(crate) enum StopOutcome {
    Stopped,
    NotRunning,
}

pub(crate) async fn stop_background_service() -> Result<StopOutcome, ClientError> {
    match endpoint::connect().await {
        Ok(mut connection) => {
            let request_id = new_request_id()?;
            let request = ServiceRequest::Stop(StopRequest {
                protocol_version: ProtocolVersion::CURRENT,
                request_id,
            });
            let response = exchange_with_timeout(&mut connection, &request, STOP_TIMEOUT).await?;
            match response {
                ServiceResponse::Stopped { request_id: resp_id } => {
                    verify_request_id(request_id, resp_id)?;
                    Ok(StopOutcome::Stopped)
                }
                ServiceResponse::Incompatible { .. } => Err(ClientError::IncompatibleService),
                _ => Err(ClientError::MismatchedResponse),
            }
        }
        Err(error) if error.is_unavailable() => {
            // Attempt cleanup of orphaned socket file if process is dead
            endpoint::clean_stale_endpoint()?;
            Ok(StopOutcome::NotRunning)
        }
        Err(error) => Err(ClientError::Endpoint(error)),
    }
}
```

#### Stale Cleanup Verification (`clean_stale_endpoint` on Unix):
1. Locate `endpoint_path()`. If `NotFound`, return `Ok(())`.
2. Verify metadata: must be a socket file owned by the current user's effective UID inside the verified private runtime directory (`0700`).
3. Verify that connecting to the socket fails with `ECONNREFUSED` (proving no process is listening).
4. Unlink the orphaned socket file with `fs::remove_file()`.

---

## 6. Adversarial Doubt Cycle (Doubt-Driven Development)

During architectural specification, five high-stakes failure modes were analyzed and stress-tested:

### 1. Race Condition Between Concurrent `open` and `stop`
- *Doubt:* What happens if a user runs `lens doc.md` and `lens stop` simultaneously in different terminals?
- *Design Resolution:*
  - If `Open` arrives at the controller first: the session is created and registered. Then `Stop` arrives, transitions controller to `Stopping`, and terminates all registered sessions including the newly opened one.
  - If `Stop` arrives at the controller first: the controller enters `Stopping`. The subsequent `Open` is immediately rejected with `OpenErrorCode::Session` ("Lens background service is stopping"). No orphaned viewing session or listener is created.

### 2. Idempotency of Repeated `lens stop` Commands
- *Doubt:* What happens if a user or automated test runs `lens stop` twice in rapid succession?
- *Design Resolution:*
  - Both invocations must exit with status code 0.
  - If the second command connects while the service is in `Stopping` state, the controller responds with `ServiceResponse::Stopped { request_id }` and the client reports `Stopped` (exit 0).
  - If the second command runs after the service has fully exited, `endpoint::connect()` returns an unavailable error; the client detects no active service, reports `No Lens background service is running.`, and exits code 0.

### 3. HTTP Route Isolation & Security Boundary
- *Doubt:* Could an unauthorized entity or rogue script in the browser send an HTTP request to terminate the background service?
- *Design Resolution:*
  - HTTP loopback listeners **never** expose a shutdown endpoint. The Axum router contains only document, revision, asset, and diagram routes.
  - Shutdown authority is restricted strictly to the local OS IPC boundary (`service-v1.sock` or Windows named pipe).
  - The service verifies the connecting peer's OS credentials (`SO_PEERCRED` on Linux, `getpeereid` on BSD/macOS, pipe DACL on Windows) matching the service owner's identity before decoding any frame.

### 4. Bounded Execution Time & Hung Service Defense
- *Doubt:* What happens if the background service is deadlocked or unresponsive?
- *Design Resolution:*
  - `lens stop` enforces a strict client-side timeout (`STOP_TIMEOUT = 3,000 ms`).
  - If the background service accepts the connection but fails to reply within 3 seconds, the client aborts and returns an actionable error: `"Lens background service did not acknowledge stop within three seconds"`. It never hangs the terminal indefinitely.

### 5. Cross-Platform Parity (Linux, macOS, Windows)
- *Doubt:* Do Unix domain sockets and Windows named pipes behave identically during abrupt client or server termination?
- *Design Resolution:*
  - Unix: Uses flock-based lock files and inode-verified unlinking. Stale socket cleanup explicitly handles dead sockets.
  - Windows: Named pipe instances automatically clean up when all handles are closed. First pipe instance semantics prevent competing ownership.
  - User-facing semantics (CLI syntax, exit codes, output text) are 100% identical across all platforms.

---

## 7. Implementation Plan & File Modifications

### 1. `src/main.rs`
- Add `#[command(subcommand)] command: Option<Command>` to `Arguments`.
- Define `enum Command { Stop }`.
- In `main()`, match `command`: if `Some(Command::Stop)`, call `lens::stop()`.

### 2. `src/lib.rs`
- Add public entrypoint:
  ```rust
  #[doc(hidden)]
  pub async fn stop() -> anyhow::Result<()> {
      match service::client::stop_background_service().await? {
          service::client::StopOutcome::Stopped => {
              println!("Lens background service stopped.");
          }
          service::client::StopOutcome::NotRunning => {
              println!("No Lens background service is running.");
          }
      }
      Ok(())
  }
  ```

### 3. `src/service/protocol.rs`
- Define `StopRequest` with `protocol_version: ProtocolVersion` and `request_id: RequestId`.
- Add `ServiceRequest::Stop(StopRequest)`.
- Add `ServiceResponse::Stopped { request_id: RequestId }`.
- Update `validate_version()` to handle `ServiceRequest::Stop`.

### 4. `src/service/client.rs`
- Implement `stop_background_service()` with `StopOutcome { Stopped, NotRunning }`.
- Add bounded `STOP_TIMEOUT` (3 seconds).

### 5. `src/service/server.rs`
- Extend `ControllerMessage` with `Stop { request: StopRequest, reply: oneshot::Sender<ServiceResponse> }`.
- Add `ControllerState::Stopping` state to `ServiceController`.
- Implement graceful drain of `ViewerSession` instances and listener termination.

### 6. `src/service/endpoint/mod.rs` & `unix.rs`
- Expose `clean_stale_endpoint()` helper on Unix to unlink orphaned socket files when connection is refused.

### 7. `src/viewer/mod.rs`
- Expose `pub(crate) async fn stop(mut self)` on `ViewerSession` to trigger graceful HTTP shutdown and abort watcher tasks.

---

## 8. Testing Strategy

All automated tests adhere to `<condition_or_action>_then_<observable_result>` naming and Arrange / Act / Assert (3A) structure.

### 1. Protocol Unit Tests (`src/service/protocol.rs`)
- `stop_request_frame_then_serializes_and_deserializes_correctly`
  - `// Arrange`: Construct `ServiceRequest::Stop` with current protocol version and known request ID.
  - `// Act`: Write frame and read frame through in-memory buffer.
  - `// Assert`: Verify deserialized frame matches original request.
- `stopped_response_frame_then_preserves_request_id`
  - `// Arrange`: Construct `ServiceResponse::Stopped { request_id }`.
  - `// Act`: Encode and decode frame.
  - `// Assert`: Verify matching `request_id`.

### 2. Server Controller Unit Tests (`src/service/server.rs`)
- `stop_message_when_running_then_replies_stopped_and_transitions_state`
  - `// Arrange`: Start controller with mock session.
  - `// Act`: Send `ControllerMessage::Stop`.
  - `// Assert`: Verify `ServiceResponse::Stopped` returned and active sessions drained.
- `open_message_when_stopping_then_rejects_with_session_error`
  - `// Arrange`: Transition controller to stopping.
  - `// Act`: Send `ControllerMessage::Open`.
  - `// Assert`: Verify immediate `ServiceResponse::Rejected`.
- `repeated_stop_message_when_stopping_then_replies_stopped_idempotently`
  - `// Arrange`: Controller already in stopping state.
  - `// Act`: Send second `Stop` message.
  - `// Assert`: Verify `ServiceResponse::Stopped` returned without panic.

### 3. CLI Integration Tests (`tests/cli.rs`)
- `stop_command_when_service_running_then_stops_service_and_exits_zero`
  - `// Arrange`: Start background service via `BackgroundService::start(...)`. Verify it is active.
  - `// Act`: Run `lens stop`.
  - `// Assert`: Verify exit code 0, stdout contains `"Lens background service stopped."`, and background process terminates.
- `stop_command_when_service_not_running_then_reports_inactive_and_exits_zero`
  - `// Arrange`: Ensure no background service is running.
  - `// Act`: Run `lens stop`.
  - `// Assert`: Verify exit code 0 and stdout contains `"No Lens background service is running."`.
- `stop_command_help_flag_then_describes_stop_command`
  - `// Arrange`: Prepare command `lens stop --help`.
  - `// Act`: Execute command.
  - `// Assert`: Verify exit code 0 and stdout describes command purpose.
- `stop_command_when_stale_socket_exists_then_removes_socket_and_exits_zero`
  - `// Arrange`: Create an inactive Unix domain socket file at the endpoint path.
  - `// Act`: Run `lens stop`.
  - `// Assert`: Verify exit code 0 and socket file is removed.

---

## 9. Boundaries & Acceptance Criteria

### Boundaries
- **Always do:**
  - Enforce strict idempotency: `lens stop` must exit 0 whether the service is running or inactive.
  - Ensure all bound loopback TCP ports and notify file watchers are completely released before process termination.
  - Delete Unix domain socket files on clean exit.
  - Isolate shutdown to the authenticated local IPC channel; never expose a shutdown HTTP route.
  - Format with `cargo fmt --check` and pass `cargo clippy --locked --all-targets --all-features -- -D warnings`.
  - Structure all tests with `// Arrange`, `// Act`, and `// Assert`.
- **Ask first:**
  - Changing the IPC frame size limit (`MAX_FRAME_BYTES`).
  - Adding external dependencies to `Cargo.toml`.
- **Never do:**
  - Attempt to stop background services belonging to other operating-system users.
  - Block the terminal indefinitely on an unresponsive service.
  - Auto-start a new background service when executing `lens stop`.
  - Leave orphaned loopback listeners or file watchers running after service stop.

### Traceability to Acceptance Criteria

- **AC-1 (Command Syntax & Discovery):** Realized in Section 2 & 7 (`main.rs` clap subcommands).
- **AC-2 (Graceful Shutdown of Active Service):** Realized in Section 4 (`ServiceController` lifecycle).
- **AC-3 (Network & Resource Release):** Realized in Section 4 (`ViewerSession` draining and listener closure).
- **AC-4 (Strict Idempotency):** Realized in Section 4 & 5 (Exit 0 for inactive/repeated stops).
- **AC-5 (Stale Endpoint Cleanup):** Realized in Section 5 (`clean_stale_endpoint`).
- **AC-6 (Per-User Isolation):** Realized in Section 2 & 6 (Endpoint runtime directory and peer authorization).
- **AC-7 (Prompt Execution):** Realized in Section 6 (`STOP_TIMEOUT` bound).
- **AC-8 (Zero Regression):** Verified by test suite and isolated module extensions.
