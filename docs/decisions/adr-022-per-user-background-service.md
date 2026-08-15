---
type: "Architecture Decision"
title: "ADR-022: Coordinate Isolated Viewing Sessions Through One Per-User Background Service"
description: "Selects authenticated local IPC, automatic service startup, client-side browser launch, and one fresh loopback viewing session per accepted Lens command."
id: "ADR-022"
status: "accepted"
date: "2026-08-05"
tags: [architecture, process-lifecycle, ipc, security]
---

# ADR-022: Coordinate Isolated Viewing Sessions Through One Per-User Background Service

Status: accepted

Date: 2026-08-05

## Context

Each current `lens <target>` process owns one loopback HTTP server and remains
in the foreground until the user stops it. Opening another target therefore
requires another terminal. `FEAT-04` requires ordinary commands to return after
a browser-ready URL is acknowledged while one background Lens process keeps
all accepted views available.

The process may be shared, but authority may not be. Existing decisions fix a
canonical root, discovered document set, source-link resolver, and PlantUML
server for every viewing session. Merely reaching a loopback port must not
allow a different operating-system user to make Lens accept a request to open
a filesystem target.

## Decision

Lens uses one automatically started background service for each
operating-system user. An ordinary command acts as a short-lived Lens client:
it captures its invocation directory, optional target, target scope, and
`LENS_PLANTUML_SERVER` value; sends one versioned open request; receives a
browser-ready loopback URL; launches the browser from the invoking desktop
environment; and returns.

The short-lived Lens command process and the background Lens service
communicate through local interprocess communication (local IPC), not a TCP
control port:

- Linux and macOS use a Unix-domain socket in a verified user-private runtime
  directory. The socket is user-only, and the service checks the connected
  peer's effective user identity.
- Windows uses a named pipe whose first instance is the single-owner claim. It
  is created with an explicit access-control list for the current user and
  LocalSystem rather than relying on the default pipe security descriptor.
- An internal endpoint module lets the rest of Lens connect to an existing
  service, claim the per-user endpoint, accept a connection, and authorize the
  connecting user. Rust selects the Unix-domain-socket or Windows-named-pipe
  implementation at compile time through `cfg`. Because Lens supports only
  these known platform implementations, it does not need a runtime abstraction
  based on traits and trait objects.

When several Lens commands simultaneously find no reachable service, each may
start a detached background-service process. Each new process tries to claim
the per-user endpoint. The process that succeeds becomes the service; the
others exit. A stale Unix socket is removed only after a connection fails and
the path is verified as an owned socket. Windows removes the pipe endpoint
when its owning process exits.

The byte-stream protocol is versioned, length-prefixed, and size-bounded. It
uses typed request and response variants, a lossless platform-native path
representation, and a request identifier. The background service retains a
request ledger for its process lifetime so an internal transport retry returns
the same in-flight or completed outcome instead of creating another viewing
session. A new CLI invocation always has a new request identifier and
intentionally opens a new view.

Every accepted command creates one new viewing session with its own ephemeral
loopback HTTP listener and existing fixed `ViewerState`. The background process
owns all session handles, but each session keeps its own document root,
discovered document set, source-link rules, and PlantUML server selection. One
session cannot access documents authorized only for another session. This
retains the current per-invocation discovery snapshot and preserves ADR-002 and
ADR-017.

Browser launching remains in the short-lived client. This preserves the
invoking desktop environment, reuses the existing platform launch commands,
and lets browser-launch failure report the already available manual URL. The
background service never needs a terminal or browser-launch environment.

The existing public `lens::serve(MarkdownTarget)` entry point remains a
foreground compatibility path. It and the background service share an internal
function that starts an isolated viewing session and returns its owned handle.
Ordinary CLI invocations use the background-service path.

The supported `lens stop` command uses the same authenticated per-user IPC
endpoint and never starts a missing service. Open and stop operations pass
through the state-owning service controller. Once it accepts stop, the
controller rejects unfinished and later opens as `ServiceStopping` and releases
every retained viewing session before acknowledging concurrent stop callers.
The service retains unclaimable per-user endpoint ownership throughout cleanup;
a Unix shutdown renames the verified socket to a private stopping barrier and a
racing claimant rechecks that barrier after binding, while accepted Windows
pipe handles retain first-instance ownership. Unix connections validate the
owned socket identity before and after connecting; Windows pipe-busy remains
retryable and does not itself mean the service is stopping. The ordinary
discoverable endpoint is removed before `Stopped`, and the ownership barrier is
released only after submitted response handlers finish. Graceful cleanup has
five seconds before remaining tasks are force-cancelled and joined. The existing
ten-second client acknowledgment bound includes teardown retries. A verified
owned stale Unix endpoint or stopping barrier is removed and reported as not
running; foreign or unverifiable endpoint state fails closed. Shutdown is not
available through browser-facing HTTP.

## Consequences

- A developer can open multiple repository views from one terminal while one
  process owns all browser-facing server tasks.
- A service crash makes all sessions in that process unavailable. A later
  command starts a new service, but already open browser URLs cannot be
  transparently transferred to it.
- Local IPC avoids fixed-port collisions and removes open and stop control
  operations from the browser-reachable HTTP surface.
- Platform endpoint creation and background-process detachment require narrow,
  security-sensitive Unix and Windows code plus native platform tests.
- The protocol must reject incompatible versions, oversized frames, malformed
  native paths, unknown variants, and peers outside the current user boundary
  before target resolution.
- Same-user processes remain inside this local trust boundary; they can already
  invoke the installed `lens` command with filesystem paths available to that
  user. The design prevents cross-user access rather than claiming isolation
  from other processes running as the same user.
- Each accepted `lens` command adds an isolated viewing session with its own
  in-memory state, document-refresh task, and loopback HTTP listener. The first
  implementation keeps these resources and each request's recorded outcome
  until the background service exits, even if the browser closes. Detecting
  closed browser views, retiring inactive sessions, and safely removing old
  request records are deferred until resource measurements can guide their
  policies, as tracked by
  [improvement 20](../improvement-proposals.md#20-measured-and-bounded-background-service-lifecycle).
- Compatibility between a Lens command and the background service is
  determined by the command-protocol version, not the Lens package version. If
  their protocol versions are incompatible, the command reports the
  incompatibility and leaves the running service unchanged so its existing
  browser views remain available.

## Alternatives Considered

### Detach every existing foreground server

Rejected because it preserves one operating-system process per command rather
than providing the confirmed single reusable process. It also leaves no common
startup, failure, or request-acknowledgment boundary.

### Use a fixed loopback TCP control port

Rejected because another program may own the port, multiple operating-system
users share the loopback network namespace, and reachability does not establish
the caller's user identity. It would also expose a filesystem-target operation
to browser-originated loopback requests unless additional authentication were
perfectly enforced.

### Publish an ephemeral TCP port and bearer token in a registry file

Rejected in favor of native local IPC. It adds token generation, protected
registry-file lifecycle, stale port records, and another TCP request surface
without reducing the platform-specific work needed to establish a secure
per-user location.

### Put every viewing session behind one global HTTP router

Rejected for the first implementation because it would rewrite all current
routes and asset URLs around a session identifier and concentrate every
authorization lookup in one mutable router. Separate ephemeral listeners
preserve the current router and make session isolation structural.

### Launch the browser from the background service

Rejected because a detached process may not retain the display environment of
the invoking terminal, and launch failure could no longer return the manual URL
through the ordinary command.

## Implementation Evidence

- The pinned Tokio 1.35.1 source provides `tokio::net::UnixListener`, Unix peer
  credentials, Windows named pipes, `ServerOptions::first_pipe_instance`, and
  explicit Windows security-attribute creation. Construction must enable the
  Tokio features used directly and add direct platform dependencies where user
  identity, security descriptors, or detachment require native APIs.
- The current viewer already binds an ephemeral `127.0.0.1` listener and owns
  one fixed `ViewerState`; extracting a session handle preserves that tested
  structure.

## Trace

- Requirement: [`FEAT-04`, `UC-11`](../features/background-viewer-service/use-cases.md)
- System sequence: [`SSD-07`](../features/background-viewer-service/ssd-07-request-target-view.md)
- Operation contract: [`OC-07`](../features/background-viewer-service/oc-07-request-target-view.md)
- Design: [`RZ-04` and `DCD-04`](../features/background-viewer-service/design.md)
- Risks: `R-06`, `R-09`, `R-11`, and `R-12` in the [risk list](../risk-list.md)
- Design iteration: [S3 background viewer service design](../iterations/s3-background-viewer-service-design.md)
