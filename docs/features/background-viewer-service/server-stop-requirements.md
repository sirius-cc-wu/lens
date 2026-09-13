---
type: "Feature Specification"
title: "Background Service Stop Command Requirements"
description: "Specifies functional requirements, user interactions, business rules, edge cases, and acceptance criteria for the 'lens stop' command."
id: "FEAT-04-REQ-STOP"
status: "active"
scope: "Lens"
tags: [requirements, specification, process-lifecycle, stop-command]
---

# Background Service Stop Command Requirements

## 1. Problem Statement & User Value

When a developer or technical writer uses Lens (`lens [TARGET]`), Lens transparently initiates a background service for the invoking operating-system user. This background service manages HTTP loopback listeners, file-system change watchers for automatic refresh, and memory state across one or more viewing sessions.

Currently, there is no dedicated CLI mechanism to stop the background service. When documentation review is complete, the background service continues running indefinitely in the background, retaining system resources (loopback TCP ports, open file descriptors, file-system watchers, and memory). Users wishing to clean up must identify the background process manually via operating-system utilities (`ps`, `pkill`, `kill`, or Task Manager) and terminate it forcefully.

### Product Goal

Provide a first-class CLI command—`lens stop`—that cleanly terminates the running background Lens service for the current user, releases all retained viewing sessions, closes network listeners, halts file watchers, cleans up IPC endpoints, and returns control to the shell with clear confirmation.

---

## 2. User Story

**As a** developer or technical writer using Lens,  
**I want to** run `lens stop` from my terminal,  
**So that** the background Lens service terminates cleanly and immediately releases all system resources (network ports, file watchers, memory, and communication sockets) without requiring manual process management.

---

## 3. Actor, Triggers, and Command Syntax

### Primary Actor
Developer or technical writer.

### Command Syntax
```bash
lens stop
```
- **Arguments:** None.
- **Flags:** Standard help flags (`-h`, `--help`) describing the command.
- **Context:** Can be executed from any directory; operates on the invoking user's background service.

### Preconditions
- The `lens` executable is available in PATH or invoked directly.
- The command executes under the privileges of the operating-system user whose service is to be stopped.

### Trigger
The user runs `lens stop` in a shell.

---

## 4. Example Mapping (Three-Amigos Discovery)

### User Story
> *Stop the background Lens service via `lens stop` to immediately release all retained sessions and system resources.*

```
                       ┌────────────────────────────────────────────────────────┐
                       │                       User Story                       │
                       │           Stop background service via lens stop.       │
                       └──────────────────────────┬─────────────────────────────┘
                                                  │
         ┌───────────────────┬────────────────────┼───────────────────┬───────────────────┐
         ▼                   ▼                    ▼                   ▼                   ▼
    ┌──────────┐       ┌───────────┐        ┌───────────┐       ┌───────────┐       ┌───────────┐
    │  Rule 1  │       │  Rule 2   │        │  Rule 3   │       │  Rule 4   │       │  Rule 5   │
    │ Graceful │       │ Strict    │        │ Total     │       │ Stale     │       │ User      │
    │ Stop     │       │ Idempotent│        │ Resource  │       │ Endpoint  │       │ Isolated  │
    │ & Exit   │       │ Success   │        │ Release   │       │ Recovery  │       │ Scope     │
    └────┬─────┘       └─────┬─────┘        └─────┬─────┘       └─────┬─────┘       └─────┬─────┘
         │                   │                    │                   │                   │
         ├─────────┐         ├─────────┐          ├─────────┐         │                   │
         ▼         ▼         ▼         ▼          ▼         ▼         ▼                   ▼
     [Ex 1.1]  [Ex 1.2]  [Ex 2.1]  [Ex 2.2]   [Ex 3.1]  [Ex 3.2]  [Ex 4.1]            [Ex 5.1]
     Running   In-flight  Not       Repeated   Ports &   IPC       Orphaned            Only
     service   drained    running   stop       watchers  socket    socket file         current
     stops     before     reports   exits 0    freed     removed   cleaned up          user's
     cleanly   exit       stopped   both                 on exit                       service
```

### Business Rules & Concrete Examples

#### Rule 1: Clean Graceful Shutdown
*When the background service is running, `lens stop` must request and confirm clean service termination.*
- **Example 1.1 (Standard stop):** The background service is active with one open repository session. The user runs `lens stop`. Lens connects to the service, requests shutdown, receives confirmation, and prints `Lens background service stopped.` The background process exits cleanly with exit code 0.
- **Example 1.2 (In-flight request draining):** A browser view is in the middle of loading a document when `lens stop` is issued. The service drains active in-flight requests within a brief bounded grace period before terminating listeners and exiting.

#### Rule 2: Strict Idempotency
*If no background service is currently running, `lens stop` must report that no service was active and exit successfully (code 0).*
- **Example 2.1 (Service not running):** No background service is running. The user runs `lens stop`. Lens detects no active endpoint and prints `No Lens background service is running.` The command exits with code 0 (not an error).
- **Example 2.2 (Back-to-back stop):** The user runs `lens stop`, which stops the service and exits code 0. Immediately after, the user runs `lens stop` again. The second command prints `No Lens background service is running.` and exits code 0.

#### Rule 3: Total Resource Reclamation
*Stopping the service must completely free all system resources held by the background process.*
- **Example 3.1 (Network ports and watchers):** The background service has 3 viewing sessions with 3 bound loopback HTTP ports and active filesystem watchers. Upon `lens stop`, all 3 ports are closed and immediately available for re-binding; all filesystem watcher handles are dropped.
- **Example 3.2 (IPC endpoint cleanup):** The service communication endpoint (Unix-domain socket file or Windows named pipe) is deleted from the filesystem so no stale artifact remains.

#### Rule 4: Stale Endpoint Recovery
*If an endpoint exists on disk but the owning process is dead (e.g. from an ungraceful crash or forced SIGKILL), `lens stop` must clean up the orphaned endpoint safely.*
- **Example 4.1 (Orphaned socket):** An endpoint socket exists in the runtime directory, but connecting to it fails because the process no longer exists. `lens stop` detects the dead endpoint, removes it, reports that the inactive endpoint was cleaned up or that no active service was running, and exits with code 0.

#### Rule 5: User and Process Isolation
*`lens stop` must operate strictly on the invoking user's background service and never interfere with services belonging to other users.*
- **Example 5.1 (Multi-user system):** User `alice` and User `bob` are logged into the same Linux machine, each running Lens. When `alice` runs `lens stop`, only `alice`'s service in `alice`'s runtime directory is stopped. `bob`'s service continues running undisturbed.

#### Rule 6: Bounded Execution Time
*The stop command must be prompt and never hang the terminal indefinitely.*
- **Example 6.1 (Fast completion):** Under normal conditions, `lens stop` completes in under 250 milliseconds.
- **Example 6.2 (Unresponsive service timeout):** If the background service is hung or fails to acknowledge within a bounded timeout (e.g., 3 seconds), the client reports the timeout error with an actionable exit status rather than blocking the shell.

---

## 5. Use Case Detailing (`UC-12`)

### Use Case Identifier: `UC-12: Stop the Background Lens Service`

- **Primary Actor:** Developer or technical writer.
- **Goal:** Terminate the background Lens service for the current user and release all associated system resources.
- **Preconditions:**
  1. The `lens` command is executable in the current environment.
- **Trigger:** The user runs `lens stop`.

### Main Success Scenario

1. The user executes `lens stop`.
2. Lens locates the background service endpoint for the current operating-system user.
3. Lens connects to the background service and transmits a stop request.
4. The background service acknowledges the stop request.
5. The background service terminates all active viewing sessions:
   - Closes all loopback HTTP listeners.
   - Halts all filesystem watchers.
   - Clears retained session state.
6. The background service unbinds and removes its IPC communication endpoint.
7. The background service process terminates cleanly.
8. The invoking `lens stop` command prints confirmation to standard output and exits with code 0.

### Extensions & Alternative Scenarios

- **2a. Service not running (No endpoint found):**
  - Lens reports that no background service is currently running.
  - The command exits with code 0.
- **2b. Stale endpoint found (Process dead):**
  - Lens attempts connection, recognizes the endpoint is dead/unresponsive, removes the stale endpoint entry, reports that no active service was running, and exits with code 0.
- **3a. Communication failure or timeout:**
  - If the service accepts connection but fails to complete shutdown within a bounded deadline (e.g. 3 seconds), Lens informs the user of the timeout and exits with an actionable non-zero error code.
- **4a. In-flight requests pending:**
  - The background service allows a brief grace period (e.g. up to 1 second) to complete pending responses before closing listeners and exiting.
- **1a. Concurrent stop commands:**
  - If multiple `lens stop` commands run simultaneously, the first initiates shutdown; subsequent callers observe the shutdown in progress or completed state and exit cleanly with code 0.

### Postconditions on Success
- The background Lens service process has terminated.
- All loopback HTTP listeners opened by the service are closed.
- All file-system notification handles / watchers opened by the service are closed.
- The IPC communication endpoint is removed.
- The terminal receives a success exit code (0) and a concise status message.

---

## 6. Observable Acceptance Criteria

| ID | Criterion | Verification Method |
|:---|:---|:---|
| **AC-1** | **Command Syntax & Discovery:** `lens --help` lists the `stop` command; `lens stop --help` describes its purpose. | Run `lens --help` and `lens stop --help`; verify concise documentation output and exit code 0. |
| **AC-2** | **Graceful Shutdown of Active Service:** Running `lens stop` when a background service is active terminates the service process cleanly and reports success. | Start a viewing session (`lens <doc>`), verify background process is running, run `lens stop`; verify process exits and CLI outputs confirmation with exit code 0. |
| **AC-3** | **Network & Resource Release:** All loopback HTTP ports and filesystem watchers held by viewing sessions are freed upon service stop. | Start multiple viewing sessions, record bound ports, run `lens stop`; assert all recorded ports are completely closed and immediately reusable. |
| **AC-4** | **Strict Idempotency:** Running `lens stop` when no service is active reports that no service is running and exits with code 0. | Ensure no Lens process is running, execute `lens stop`; verify output indicates service is not running and exit code is 0. |
| **AC-5** | **Stale Endpoint Cleanup:** If an orphaned socket/endpoint exists without an active process, `lens stop` removes it and exits with code 0. | Create a fake/orphaned socket at the endpoint path, run `lens stop`; verify socket file is removed and exit code is 0. |
| **AC-6** | **Per-User Isolation:** `lens stop` only terminates the invoking user's service and does not affect other operating-system users. | Verify endpoint path resolution is strictly scoped to the current user's runtime directory. |
| **AC-7** | **Prompt Execution:** `lens stop` completes and returns terminal control promptly (investigation threshold < 250ms for local shutdown; bounded hard timeout). | Time execution of `lens stop` against active and inactive services. |
| **AC-8** | **Zero Regression:** Existing document viewing commands (`lens [TARGET]`), automatic refresh, and viewer session isolation remain unaffected. | Run full test suite (`cargo test --locked` and browser integration tests). |

---

## 7. Architectural Handoff Signals

*Guidance for the Software Architect when authoring the Technical Design Specification (`docs/features/background-viewer-service/server-stop-design.md` or similar):*

1. **IPC Protocol Frame:** Define a typed `Stop` or `Shutdown` request and corresponding acknowledgment response in the background service protocol (`src/service/protocol.rs`).
2. **Controller Shutdown Flow:** Extend `ServiceController` to handle the stop signal:
   - Cease accepting new connection frames.
   - Trigger cancellation / shutdown signals across all retained `ViewerSession` instances.
   - Allow a brief drain window for in-flight HTTP requests.
   - Close Axum listeners and notify file watchers to terminate.
3. **Endpoint Unlink:** Ensure the service deletes its Unix-domain socket (or closes its named pipe handle) during clean exit, while the client also incorporates fallback unlink logic for dead sockets.
4. **Client-Side Timeout Bounds:** Enforce a strict timeout (e.g. 3000ms) on waiting for service shutdown acknowledgment before falling back to an actionable error.
5. **Cross-Platform Parity:** Ensure identical user-facing behavior across Linux (Unix domain sockets), macOS, and Windows (named pipes).
