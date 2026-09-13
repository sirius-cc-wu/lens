---
type: "Architecture Decision"
title: "ADR-024: Provide a Dedicated Command to Stop the Background Viewer Service"
description: "Defines the 'lens stop' CLI command, typed IPC shutdown protocol framing, controller state transitions, graceful session draining, and idempotent termination."
id: "ADR-024"
status: "accepted"
date: "2026-09-13"
tags: [architecture, decision, process-lifecycle, ipc, stop-command]
---

# ADR-024: Provide a Dedicated Command to Stop the Background Viewer Service

Status: accepted

Date: 2026-09-13

## Context

The background Lens service runs detached to host isolated loopback viewing sessions across multiple commands. When documentation review finishes, the background service continues running indefinitely, retaining loopback TCP ports, open file descriptors, filesystem watchers, and memory. Terminating it previously required manual operating-system process management (`pkill`, `kill`, or Task Manager). Lens requires a first-class CLI command—`lens stop`—that cleanly terminates the background service for the current operating-system user, releases all retained viewing sessions and system resources, cleans up communication endpoints, and exits idempotently with code 0 while strictly preserving user isolation and zero HTTP shutdown exposure.

## Decision

Lens implements a first-class `lens stop` CLI command backed by typed local IPC protocol framing and a state-owning controller shutdown flow:

1. **CLI and Library Entrypoint:** A `stop` subcommand is parsed by `clap` alongside the existing optional target argument. It invokes `lens::stop()`, which delegates to the client coordinator without spawning a background service process.
2. **Client Discovery and Idempotency:** The client attempts to connect to the current user's authenticated local endpoint. If no service is reachable (or an orphaned stale socket exists from an abnormally terminated process), the client unlinks any dead socket file, informs the user that no service was running, and exits with code 0.
3. **Typed IPC Protocol Framing:** Communication uses a typed `ServiceRequest::Stop(StopRequest)` frame carrying `ProtocolVersion::CURRENT` and a unique `RequestId`. The server acknowledges with `ServiceResponse::Stopped { request_id }`.
4. **Controller State Machine & Lifecycle:**
   - The `ServiceController` implements an explicit lifecycle: `Running` $\rightarrow$ `Stopping` $\rightarrow$ `Stopped`.
   - Upon receiving `Stop`, the controller enters `Stopping` and immediately rejects any subsequent `Open` requests with a typed `Session` rejection.
   - Concurrent or repeated `Stop` requests return `ServiceResponse::Stopped` idempotently.
   - The service controller gracefully drains active in-flight HTTP requests across all retained `ViewerSession` instances, aborts document watcher tasks, and closes loopback TCP listeners.
5. **Endpoint Unlinking & Resource Reclamation:** The background service deletes its Unix-domain socket (or drops its Windows named pipe server handle) upon exiting. The client verifies completion within a bounded timeout (3 seconds) and prints confirmation.
6. **Security & Boundary Isolation:** Shutdown is exclusively authorized via the local OS-peer-authenticated IPC boundary. HTTP endpoints never expose shutdown routes. The command operates strictly on the current user's runtime endpoint.

## Consequences

- Running `lens stop` terminates the invoking user's background service cleanly, releasing all loopback TCP ports, filesystem watchers, and memory.
- The command is strictly idempotent: running it when no service is active reports that no service was running and exits successfully with code 0.
- Stale endpoint files left by abnormal process terminations are automatically detected, unlinked, and reported as inactive.
- HTTP endpoints remain completely isolated from process lifecycle controls; browser requests cannot terminate the service.
- Multi-user isolation is maintained; stopping one user's service never affects other users on the system.
- Existing `lens [TARGET]` commands, automatic refresh, and session isolation remain unaffected.

## Trace

- Use case: [`UC-12`](../features/background-viewer-service/use-cases.md)
- Requirements: [`FEAT-04-REQ-STOP`](../features/background-viewer-service/server-stop-requirements.md)
- Technical design: [`docs/features/background-viewer-service/server-stop-technical-design.md`](../features/background-viewer-service/server-stop-technical-design.md)
- Related decisions: [ADR-022](adr-022-per-user-background-service.md)
