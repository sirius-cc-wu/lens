---
type: "Improvement Proposal"
title: "Host a Pi Agent Workspace in Lens"
description: "Adds an optional browser-based Pi coding-agent workspace to Lens through a local process and Pi's line-delimited RPC protocol."
id: "PROP-PI-AGENT-WORKSPACE"
status: "proposed"
tags: [proposal, pi, coding-agent, local-ui, rpc, security]
---

# Host a Pi Agent Workspace in Lens

Status: proposed

## Decision Sought

Expand Lens with an optional **agent workspace**: a browser view that lets a
user run the local Pi coding agent in a selected workspace. Lens would provide
the browser page, local process supervision, and presentation of streamed
agent activity. Pi would remain responsible for model access, tools, workspace
trust, approvals, agent state, and session persistence.

This is a product-scope expansion. Lens is currently a focused documentation
viewer; the agent workspace must be an explicitly separate surface, not an
additional behavior hidden in ordinary document views or the Markdown renderer.

The recommended integration is a Pi child process using `pi --mode rpc` and
its documented line-delimited JSON protocol. Lens must not link Pi as a Rust
library or recreate Pi's provider, tool, session, or authorization logic.

## Summary

A new command with the intended shape below opens an isolated, loopback-only
agent workspace for a directory:

```text
lens agent [WORKSPACE]
```

The short-lived Lens command asks the existing authenticated per-user Lens
service to create an agent workspace. That service starts one Pi child process
for the new workspace with its current directory set to the selected directory
and with standard input and output connected to an RPC adapter. The service
then creates an isolated, token-protected browser session and returns its URL
to the command for normal browser launch.

The browser page sends narrowly defined user actions to Lens. Lens translates
them into Pi RPC commands and converts Pi's streamed events into browser
updates. It never gives the browser a general stdin, shell, filesystem, or
provider API. Pi remains the authoritative owner of conversation history,
tool execution, workspace trust, and saved sessions.

The first user-visible slice is a streamed conversation and a tool-activity
timeline. A complete agent workspace additionally requires a safe, structured
way to present and answer Pi's ordinary tool-approval requests. Pi already
documents an RPC flow for extension UI requests; accepting this proposal does
not authorize Lens to infer, bypass, or silently grant ordinary tool approvals.
That protocol prerequisite must be resolved before a GUI release can offer
normal tool-using agent sessions.

## Motivation

Pi already has a suitable frontend boundary: its RPC mode accepts commands
such as `prompt`, `steer`, `follow_up`, `abort`, `get_state`, and `compact`, and
emits streaming message, tool-execution, lifecycle, and error events. Lens
already has the complementary local presentation capabilities:

- an authenticated, loopback-only browser session;
- a per-user background service reached through user-authenticated local IPC;
- browser launch from the invoking desktop environment; and
- a safe, local repository-review workflow with validated VS Code handoff.

Joining those responsibilities at the process boundary avoids two expensive
and fragile alternatives: converting Pi into a browser server, or embedding a
second asynchronous runtime and the full agent implementation into Lens.

The result should make Pi easier to observe during coding work. A user can see
streaming output, tool calls, pending decisions, and errors in one local page,
while Pi continues to work from the same workspace and retain the same command
line and terminal interfaces.

## User Scenario

Primary actor: developer

Goal: work with Pi in a local browser workspace while retaining Pi's existing
security and session behavior.

Preconditions:

- Lens and a compatible `pi` executable are installed for the same operating
  system user.
- The user selects a readable directory to become Pi's working directory.
- Pi can load its own configuration and has any credentials required by the
  selected model.
- The workspace satisfies Pi's own trust policy. Lens does not pre-approve a
  workspace for Pi.

Main success scenario:

1. The user runs `lens agent .` from a repository.
2. The Lens command submits an agent-workspace request through the existing
   per-user local IPC service.
3. The service validates the directory, starts a fresh Pi child in that
   directory with `--mode rpc`, and starts an isolated loopback browser view.
4. Lens launches the browser with the authenticated view URL and returns the
   URL to the terminal.
5. The user enters a prompt in the browser page.
6. Lens sends a correlated Pi `prompt` command and displays Pi's streamed text
   and tool-execution events as they arrive.
7. When Pi completes the turn, Lens shows the final outcome and leaves the
   workspace ready for the user's next prompt.
8. The user may issue a supported steering, follow-up, abort, state, or
   compaction action. Lens translates only that defined action rather than
   relaying arbitrary protocol input.

Extensions:

- If `pi` cannot be found, cannot start, or emits a startup error, Lens renders
  the diagnostic in the agent view and does not accept prompts.
- If the Pi child exits, Lens preserves the bounded visible event history and
  marks the workspace disconnected. It must not automatically replay a prompt
  or tool call into a replacement process.
- If Pi asks for an extension UI response, Lens renders the request and sends
  the response with Pi's required request identifier and generation value.
- If Pi requires an ordinary tool approval that the RPC contract cannot
  represent, the agent workspace fails closed with an actionable message; it
  must not turn on a permissive Pi mode or auto-approve the call.
- The user can explicitly stop the agent workspace. Stopping terminates its Pi
  child and closes its agent-specific browser routes without affecting Lens
  document-view sessions or other agent workspaces.

## Goals

- Provide an opt-in local GUI for Pi's supported RPC workflow.
- Preserve Pi as the owner of provider credentials, model selection, tools,
  workspace trust, approvals, agent-loop behavior, and session persistence.
- Reuse Lens's per-user service and isolated loopback-session pattern without
  weakening its local IPC or browser authentication boundaries.
- Present streaming assistant text, lifecycle status, tool activity, errors,
  and explicitly supported user decisions clearly and accessibly.
- Support one independent Pi process per agent workspace so two repositories
  cannot share a working directory, Pi input stream, event stream, or visible
  transcript by accident.
- Retain Lens's existing document-review behavior unchanged.
- Keep all agent prompts, responses, tool arguments, and tool results local to
  Lens and Pi except for network operations Pi itself performs under its
  existing configuration and policy.

## Non-Goals

This proposal does not:

- turn Lens into a general source-code browser, editor, terminal emulator, or
  HTTP API for arbitrary local programs;
- embed Pi as a Lens crate dependency or make Pi adopt Tokio, Axum, or Lens's
  process lifecycle;
- replace Pi's terminal user interface, print mode, JSON mode, RPC mode, or
  Agent Client Protocol (ACP) support;
- create a hosted, multi-user, LAN-reachable, or cloud-synchronized agent
  service;
- expose Pi's stdin, stdout, configuration, credentials, or workspace files to
  arbitrary browser JavaScript;
- infer approval from a tool name, a model response, or a previous decision;
- enable permissive execution flags such as a global "approve everything"
  option merely because the frontend is graphical;
- render assistant messages through Lens's document renderer, send them to a
  PlantUML server, or allow agent output to change the document set; or
- add code editing, patch application, merge conflict resolution, or a source
  diff viewer in the first release.

A later proposal may add a reviewed diff or editor-handoff workflow once it has
a dedicated authorization model. It must not be smuggled in through tool-event
rendering.

## Proposed Command and Workspace Selection

The public command should be a named subcommand rather than an overloaded
ordinary `lens [TARGET]` invocation:

```text
lens agent [WORKSPACE]
```

`WORKSPACE` defaults to the invoking directory. It must name a readable
physical directory after canonicalization; a Markdown file, PlantUML file,
symlink, or nonexistent path is rejected. The directory becomes Pi's current
working directory exactly as selected. Lens does not use document discovery or
`--scope` to broaden it to a repository root, because Pi's working directory
is execution authority rather than a document-view selection.

The command parser currently reserves the ordinary positional argument for a
document target. Its eventual subcommand design must retain existing `lens`,
`lens [TARGET]`, and `lens stop` behavior. `agent` needs its own target type,
validation error messages, help text, and tests rather than reusing a
Markdown-oriented target type by coincidence.

An initial release resolves `pi` through the invoking service's inherited
`PATH`, reports the resolved program in diagnostics, and invokes it without a
shell. A later explicit executable-selection setting can be considered only
with validation and a clear ownership model; it is not needed to prove the
integration.

## Architecture

### Process and authority boundary

```text
Lens command
    | authenticated native local IPC
    v
Lens per-user background service
    | owns one isolated agent-workspace controller
    | starts a Pi child with cwd = validated workspace
    v
Pi --mode rpc <---- line-delimited JSON ----> Lens RPC adapter
    ^                                             |
    |                                             | token-protected loopback HTTP + event stream
    |                                             v
Pi configuration, tools, sessions, trust       Browser agent workspace
```

The background service owns the agent-workspace controller, its Pi child, the
bounded in-memory browser projection, and the loopback listener. It must use
one controller and one Pi process per accepted `lens agent` request. It may
share the current service process with document-view sessions, but no state
from a Pi controller may be stored in a document `ViewerState` or exposed by a
document route.

The child invocation is built with a platform process API equivalent to:

```text
pi --mode rpc
```

with its current directory set to the validated workspace and its standard
input, output, and error separately piped. The implementation must use direct
argument passing, not `sh -c`, `cmd /C`, string interpolation, or a shell
command prefix. Environment inheritance must be deliberate and minimal: Pi
needs its normal configuration and credentials, while Lens-specific session
secrets and browser token values must never be inherited by the child.

Pi's exit status and a bounded tail of stderr are diagnostics, not browser
markup. Lens must escape and size-limit all child-supplied text before display.

### RPC adapter

The adapter is a typed Lens component that owns Pi's stdin writer and stdout
reader. It serializes writes so two browser actions cannot interleave JSON
lines. It parses each stdout line as a size-bounded JSON value, classifies
known event types, and maps them to Lens-owned browser events.

The initial mapping is:

| Pi event or command | Lens agent-workspace behavior |
|---|---|
| `prompt` | Accept a typed prompt action and associate it with a Lens-generated request identifier. |
| `steer`, `follow_up`, `abort`, `get_state`, `compact` | Accept only a dedicated, validated browser action and send the corresponding documented Pi command. |
| `agent_start`, `agent_end` | Update turn lifecycle and display a concise status. |
| `message_update` | Append a safely rendered streaming text delta to the current assistant message. |
| `tool_execution_start`, `tool_execution_end` | Add a tool-timeline item with escaped, size-bounded structured details. |
| `response` | Resolve the matching browser action and show success or failure. |
| `error` | Mark the session or action failed with Pi's stable code and human-readable message. |
| `extension_ui_request` | Render a typed decision surface and send the documented correlated UI response. |

Lens must retain unknown Pi records as a bounded diagnostic event and display a
protocol-compatibility warning. It must not treat an unknown record as a text
delta, a successful completion, an approval, or an executable browser command.
A malformed, oversized, or semantically invalid record ends the Pi controller
rather than letting an unbounded or ambiguous stream reach the browser.

The adapter is an integration boundary, not a second implementation of Pi's
agent state machine. Pi's messages and persisted session are authoritative.
Lens holds only enough in-memory projection to reconnect a page, render the
current turn, and diagnose process failure.

### Browser transport and routes

An agent workspace has its own loopback listener and fresh high-entropy session
token, following the current Lens viewer-session model. It must use a distinct
agent route namespace and state type. A document session must never be able to
reach an agent endpoint by guessing a path, and an agent session must never
serve an arbitrary document or filesystem path.

The browser protocol should be deliberately small:

- a state endpoint returns a sanitized snapshot needed to render or reconnect;
- an event-stream endpoint delivers ordered, Lens-defined server-sent events
  (SSE) with monotonically increasing event identifiers;
- command endpoints accept typed JSON actions such as prompt, steer,
  follow-up, abort, state refresh, compact, stop, and supported UI responses;
  and
- no endpoint accepts a raw Pi JSON line, a command string, a pathname, a shell
  fragment, or a provider credential.

SSE is preferred for the first implementation because Pi's event stream is
server-to-browser and commands are independent HTTP requests. The controller
keeps a bounded event ring so a browser that reconnects with its last event
identifier can either receive missed events or receive a fresh state snapshot
when it fell behind. The implementation must define a maximum retained event
count and byte size before construction.

All browser routes require the per-session secret used by Lens's existing
loopback views. They must preserve the no-referrer policy, use no CORS support,
and keep the content security policy restricted to same-origin resources and
connections. State-changing requests must additionally require a same-origin
`Origin` check, a JSON content type, request-body limits, and a valid session
secret. The token must never be written to logs, echoed in an error page,
passed to Pi, or sent to an external URL.

### Rendering and accessibility

Agent text and tool output are untrusted data. They must be escaped before
insertion into HTML. The first release should use a small, local, deliberately
safe text/Markdown subset or plain text with code blocks; it must not reuse the
document-rendering pipeline, execute authored HTML, fetch remote media, or
process PlantUML/Mermaid fences from agent output.

The primary page has these semantic regions:

- workspace identity and Pi connection status;
- the scrollable conversation transcript;
- an accessible live status region for streamed progress;
- a tool-activity timeline with expandable, escaped details;
- a prompt form and explicit turn controls; and
- a modal or equivalent focus-managed surface for structured Pi UI requests.

Keyboard operation, visible focus, submission prevention while an action is
invalid, clear busy/error state, and readable long command output are required.
A screen reader must receive concise status changes without having every token
delta announced. Tool results can contain secrets, so collapsed details should
not be copied into page titles, URLs, browser notifications, or log messages.

### Approval and trust prerequisite

Pi must remain the policy decision point. Lens may display a decision Pi has
asked it to present, but it must not decide whether a tool needs approval,
remember a broader permission than Pi granted, or convert a GUI click into an
unbounded trust setting.

Before a tool-using agent workspace can ship, the integration needs a stable
Pi RPC contract for ordinary approval requests with at least:

- a unique request identifier and generation/correlation value;
- the requested operation, enough safe context to explain its impact, and the
  choices Pi permits;
- an explicit response shape that Pi validates;
- cancellation, timeout, duplicate-response, and stale-response behavior; and
- a fail-closed result when the frontend disconnects or cannot render the
  request.

Pi's documented extension UI events demonstrate the appropriate correlated
pattern, but they do not prove that ordinary tool approval can be represented
by the same event. The first Lens integration must audit Pi's actual RPC
behavior. If the contract is absent, this proposal's first executable slice
must either restrict itself to a Pi configuration that cannot issue tool calls
or stop before the tool would execute and direct the user to Pi's terminal
surface. It must never launch Pi with an approval-bypassing flag as a
workaround.

Pi's workspace trust-on-first-use behavior is likewise Pi-owned. Lens can show
an error or a structured Pi request, but it must not write trust records or
pretend that opening a Lens page constitutes workspace trust.

### Lifecycle and persistence

Pi continues to write and resume its own session records. Lens does not invent
a parallel conversation store, copy Pi's sessions into its service protocol,
or mutate Pi session files directly. A future session picker may ask Pi through
a documented interface or launch Pi with a documented session-selection option;
that work needs a separate compatibility and privacy review.

The service stops the Pi child when the user selects **Stop agent workspace**,
when a fatal protocol violation occurs, or when the Lens background service is
stopped. Child shutdown must first request normal termination with a bounded
wait and then use a platform process termination mechanism only if necessary.
It must never kill a process group selected by a user-provided string.

Closing a browser tab alone is not proof that the user wants to abort a turn:
the page may reload or reconnect. The first implementation therefore keeps the
controller while its Lens service remains alive and exposes explicit stop. It
must measure retained-memory, process-count, and provider-cost behavior under
abandoned tabs before adding an idle-retirement policy. Such a policy must
abort a running turn before terminating its child and must be independently
reviewed.

## Privacy and Security

The integration handles more sensitive material than a document viewer. Prompts
can contain source code or credentials, and tool output can contain repository
contents, command output, and access tokens. The following invariants apply:

- Agent pages bind only to loopback and retain per-session authentication; Lens
  does not open a LAN port or a shared browser session.
- Lens's native local IPC remains the only way to ask the background service to
  create an agent workspace. Browser requests cannot create one or select its
  directory.
- The workspace directory is supplied through authenticated local IPC, is
  validated before child launch, and is never selected from a browser request.
- Lens invokes a fixed Pi executable with fixed arguments and a validated
  current directory. It never evaluates a browser-derived command line.
- The browser receives only Lens-defined projected events. It does not receive
  a bidirectional byte stream to the child process.
- Lens does not persist raw prompts, transcripts, tool arguments, tool results,
  or child stderr beyond bounded in-memory state needed for the active view.
  Pi's own documented session behavior remains visible to the user and is not
  changed by this proposal.
- Agent output never enters the PlantUML request path. Lens must not transmit
  prompts, replies, or tool results to Lens's configured PlantUML server.
- Logs use event category, request identifiers that are not browser tokens, and
  byte counts where possible. They must not record prompt or tool-result bodies
  by default.
- A page refresh or a replayed browser request cannot cause Lens to replay a
  Pi prompt or approve a decision. Commands require a one-time Lens action
  identifier and have defined duplicate behavior.

The current viewer uses a URL session token with a no-referrer policy. The
agent implementation should reuse only the vetted authentication mechanism or
strengthen it in one shared, separately reviewed change; it must not copy
ad-hoc token parsing into new routes. Any change to the shared token mechanism
must preserve document-view security and receive dedicated route tests.

## Compatibility

The agent workspace is opt-in. Existing invocations retain their meaning:

```text
lens
lens [TARGET]
lens --scope target [TARGET]
lens stop
```

Installing Pi is an additional requirement only for `lens agent`; ordinary
Lens document viewing must not try to discover, start, or require Pi. A missing
or incompatible Pi executable is a contained agent-workspace failure, not a
Lens startup failure.

Lens and Pi are separately versioned projects. The integration is compatible
with the documented Pi RPC records it uses, not merely with an executable name
or semantic version string. Construction must use a synthetic Pi fixture to
prove the boundary and must pin an integration test against a known compatible
Pi build before release. A protocol change should produce a visible, safe
incompatibility state rather than silent truncation or an automatic downgrade.

## Delivery Plan

Each step should be a small, independently reviewable iteration. Pi changes
are a cross-repository prerequisite and must land with Pi's own quality and
release process; Lens must not carry a hidden compatibility shim for an absent
Pi feature.

1. **Elaboration: verify the Pi UI contract.** Document the exact Pi RPC event
   schema, lifecycle guarantees, and ordinary tool-approval behavior using a
   versioned fixture. Decide whether Pi must add an approval event family.
   Produce a joint compatibility matrix and fail-closed behavior for unsupported
   Pi versions.
2. **Lens agent-session foundation.** Add a separately typed agent-workspace
   request to the authenticated Lens service, directory validation, child
   process supervision, bounded stdout/stderr handling, and a deterministic
   fake-Pi test program. Do not yet add browser input that can cause a real tool
   call.
3. **Read-only streamed browser projection.** Add the isolated agent listener,
   authenticated state/SSE routes, safe transcript rendering, process/error
   states, and browser reconnection tests against the fake Pi.
4. **Typed conversational controls.** Add prompt, steering, follow-up, abort,
   state, and compaction actions with request correlation, duplicate handling,
   and turn-state rules. Validate against a compatible real Pi invocation that
   cannot make unapproved tool calls.
5. **Approval and trust integration.** Only after Pi provides the verified
   contract, add structured extension and ordinary approval decisions with
   focus management, stale-response rejection, disconnect behavior, and
   end-to-end fail-closed tests.
6. **Transition evidence.** Measure startup, idle resource retention, event
   backpressure, shutdown, browser recovery, and cross-platform process
   behavior. Update user guidance, security documentation, risks, release
   readiness, and the public command reference.

## Alternatives Considered

### Link Pi into Lens as a Rust crate

Rejected. Lens is a Tokio/Axum application while Pi uses asupersync and has a
large independent runtime, configuration, provider, tool, and session surface.
In-process linking would couple release cadence and runtime assumptions, make
cancellation and process recovery harder, and turn a clean frontend protocol
into internal API coupling.

### Add a network server directly to Pi

Rejected for the initial integration. Pi already provides a process-local RPC
frontend boundary. Adding browser authentication, HTTP routing, static assets,
origin handling, and multi-view lifecycle to Pi would duplicate Lens's local
viewer responsibilities and enlarge Pi's attack surface.

### Let browser JavaScript spawn or directly control Pi

Rejected. Browsers cannot safely own local process authority. A direct bridge
would expose a high-value arbitrary-command surface, complicate origin and
session protection, and bypass Lens's authenticated service boundary.

### Treat the agent page as another Markdown document

Rejected. An agent workspace has streaming state, mutations, private output,
and user-decision controls that do not belong in the fixed authorized document
set. Reusing the document renderer would also risk sending agent-provided
PlantUML to an external renderer.

### Use an embedded terminal or an iframe of the existing Pi TUI

Rejected. It would reproduce terminal emulation problems in a browser and
would not expose structured tool, lifecycle, approval, or session events.
Pi's RPC protocol is the correct integration boundary.

### Build a Tauri, Electron, or hosted web application first

Rejected. Lens already supplies a cross-platform local browser presentation
path. A new desktop shell or hosted service adds packaging, update,
authentication, and multi-user concerns before the local agent workflow has
been proven.

### Use Pi ACP instead of Pi RPC

Deferred. ACP is valuable for editor clients, but the documented Pi RPC
protocol directly supplies the agent controls and events required here. Lens
should begin with RPC and reconsider ACP only if a concrete missing contract or
an ecosystem integration requires it.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| A browser action bypasses Pi approval or workspace trust. | Make Pi the policy authority; require a correlated approval protocol; fail closed when unsupported. |
| Browser code gains arbitrary child-process or filesystem authority. | Use typed endpoints and a Lens-owned adapter; never relay raw stdin, shell fragments, or browser paths. |
| Prompt, source, or tool output leaks through logs, referrers, or PlantUML. | Use no-referrer authenticated sessions, redacted logging, escaped local rendering, and prohibit agent content from document/diagram pipelines. |
| Lens and Pi update independently and silently misinterpret events. | Use a typed, versioned fixture, strict event validation, bounded unknown-event diagnostics, and an explicit incompatible state. |
| A Pi child outlives the UI or an abandoned page consumes provider resources. | Provide explicit stop, tie children to the service lifecycle, measure abandoned-session behavior, and design any idle policy separately. |
| Streaming output exhausts memory or blocks a page. | Bound line sizes, event history, tool-detail sizes, request bodies, and stderr retention; define overload termination behavior. |
| A new route weakens document-view authentication. | Keep agent state and routes separate; reuse shared auth only through an audited common component; run existing viewer security tests unchanged. |
| Rendering model output creates cross-site scripting or unsafe external requests. | Escape all output, use a small local renderer, restrictive CSP, and no remote-content/diagram processing for agent messages. |

## Acceptance Criteria

The proposal is ready for implementation when its Pi approval prerequisite has
an explicit, tested answer. A production agent workspace is complete only when:

- `lens agent [WORKSPACE]` opens an isolated authenticated loopback view while
  existing Lens command behavior remains unchanged;
- the selected workspace is a validated directory and is the Pi child's exact
  working directory;
- Lens starts Pi with direct process arguments and `--mode rpc`, never through
  a shell, and reports missing, incompatible, malformed, and exited child
  states without exposing secrets;
- the browser can submit typed prompts and receive streamed assistant and tool
  lifecycle updates in order;
- browser actions cannot send arbitrary Pi protocol records, select a path,
  execute a command, or obtain credentials;
- every agent browser route requires the session secret, rejects cross-origin
  mutation attempts, applies body and event-size limits, and emits no external
  network request merely to render a conversation;
- agent output is safely rendered and is never passed to the Markdown document,
  PlantUML, or Mermaid pipelines;
- Lens forwards supported extension UI interactions with Pi's required
  correlation data;
- ordinary tool approval is represented through a documented Pi contract or
  fails closed before execution; no implicit approval or permissive fallback is
  used;
- stopping an agent terminates only its own Pi child and leaves other Lens
  document/agent sessions available;
- browser reconnection receives a correct bounded projection without replaying
  a prompt, tool call, or decision;
- Pi remains the sole writer of Pi session state; and
- Linux, macOS, and Windows evidence covers child launch, termination,
  loopback access, local IPC authorization, and a compatible Pi fixture.

## Verification Approach

The implementation should include focused unit, service, and browser tests
with setup, one primary action, and observable verification. Representative
scenarios include:

- `agent_workspace_directory_then_starts_pi_with_that_current_directory`;
- `agent_workspace_file_target_then_rejects_before_child_launch`;
- `missing_pi_executable_then_renders_contained_startup_error`;
- `pi_text_delta_then_emits_ordered_browser_event`;
- `malformed_pi_record_then_stops_controller_without_rendering_record`;
- `browser_raw_rpc_payload_then_rejects_without_writing_to_pi`;
- `duplicate_prompt_action_then_writes_one_pi_prompt`;
- `browser_reconnect_after_retained_event_then_receives_missing_projection`;
- `browser_reconnect_after_history_expiry_then_receives_fresh_snapshot`;
- `cross_origin_prompt_then_rejects_without_agent_action`;
- `tool_result_with_html_then_renders_as_text_without_script_execution`;
- `agent_output_with_plantuml_fence_then_makes_no_renderer_request`;
- `unsupported_ordinary_approval_then_fails_closed_before_tool_execution`;
- `stop_one_agent_workspace_then_terminates_only_its_pi_child`; and
- `pi_child_exit_then_does_not_replay_pending_prompt`.

A compiled-browser test should use a fake Pi process that emits controlled
records and should inspect visual states, keyboard focus, accessible names,
streaming/reconnection behavior, and the absence of unsafe browser requests.
Real-Pi integration tests should be opt-in or use a controlled compatible
provider fixture so they do not consume credentials or perform an uncontrolled
agent action during ordinary Lens tests.

## Open Questions

1. What exact ordinary tool-approval records and response semantics can Pi
   guarantee over RPC, and which Pi release first carries them?
2. Should `lens agent` accept only directories as proposed, or should a future
   explicit `--workspace` option allow a document page to open the directory
   that contains it? The first version should avoid implicit path conversion.
3. What is the smallest Pi configuration that proves streaming conversation
   with no tool-call authority for the early Lens browser slice?
4. Which portion of Pi's structured tool result is safe and useful to show by
   default, and what detail-size/redaction rules preserve diagnostics without
   spreading secrets across the browser page?
5. When measurements exist, what idle timeout, reconnection window, and
   service-restart policy best bound abandoned agent processes without
   unexpectedly interrupting an active user?
6. Should Pi publish a formal machine-readable RPC capability/version handshake
   rather than requiring Lens to infer compatibility from observed events?

## Out of Scope for This Proposal

- A final visual design system for the agent page.
- Patch, diff, or source-file content rendering in Lens.
- An editor plugin, VS Code extension, or replacement for Pi ACP.
- Multi-user access, remote browser access, or sharing agent sessions.
- Persisting Lens-owned transcripts, analytics, cloud backups, or telemetry.
- Configurable arbitrary command runners or providers.
- Automatic approval, workspace-trust modification, or a graphical equivalent
  of Pi's permissive execution modes.
- Changing Lens's normal Markdown, PlantUML, Mermaid, source-link, or
  background-service behavior except where a separately typed agent-workspace
  request shares reviewed service infrastructure.
