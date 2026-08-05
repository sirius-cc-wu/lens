---
type: "Code Review"
title: "PR #13 Review: Reusable Background Viewer Service"
description: "Reviews the per-user command endpoints, client and service lifecycle, retained viewing sessions, public Rust API, native platform behavior, documentation, and verification coverage."
pull_request: 13
status: "completed"
date: "2026-08-05"
tags: [review, background-service, ipc, security, cross-platform]
---

# PR #13 Review: Reusable Background Viewer Service

Review range:
`1c6cd2d5e620991b3968dd39150c85e9ee1c4c7b..b9c62144a717720aa3c27e26f4c2f9ca2c5022bd`.
The base and head are the pull request's reported commits. The local
`feat/background-viewer-service` branch matched the writable remote head before
this review record was added.

## Findings

1. **[High] The Windows client does not authenticate the named-pipe server
   — [`src/service/endpoint/windows.rs:43`](../../src/service/endpoint/windows.rs#L43)**

   Explanation and impact: The user-SID-specific name and the DACL in
   `create_server` restrict clients of a legitimate Lens pipe, but they do not
   prove to a client that the process which created an existing pipe is Lens or
   even belongs to the current user. Windows permits an unrelated process to
   act as a named-pipe server, and that process chooses the security descriptor
   for the pipe it creates. A different local user can therefore create
   `\\.\pipe\lens-<victim-sid>-v1` first with a DACL that admits the victim.
   `ClientOptions::open` connects without checking the server process or token,
   sends the invocation directory, target, PlantUML configuration, and request
   identifier, and trusts any matching response.

   The attacker can copy the observed request identifier into a `Ready`
   response and supply an arbitrary `view_url`. The client passes that value to
   [`cmd /C start`](../../src/browser.rs#L34) without first requiring an HTTP
   loopback URL. Rust's
   [Windows `Command` guidance](https://doc.rust-lang.org/stable/std/process/struct.Command.html#method.arg)
   warns that attacker-controlled `cmd.exe` arguments can run arbitrary shell
   commands. This turns a predictable endpoint-name race into cross-user data
   disclosure and potentially code execution in the victim's account. The
   Windows DACL test proves only the policy produced by the legitimate server;
   it does not exercise a pipe created by an adversarial server. Microsoft also
   documents that [any process can act as a named-pipe server](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipes)
   and that the [pipe creator's security descriptor controls client access](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights).

   Reported behavior and impact:

   ```plantuml
   @startuml
   title Predictable Windows pipe can be claimed by another user
   actor Attacker
   actor Victim
   participant "Attacker pipe server" as FakePipe
   participant "Lens client" as Client
   participant "cmd /C start" as Cmd

   Attacker -> FakePipe : create victim's predictable pipe name\nwith a DACL admitting the victim
   Victim -> Client : lens confidential-docs
   Client -> FakePipe : connect and send path, configuration, request_id
   FakePipe --> Client : Ready(same request_id, hostile view_url)
   Client -> Cmd : start hostile view_url
   Cmd --> Victim : attacker-selected command or destination
   @enduml
   ```

   Proposed fix: After opening a Windows pipe, identify its server process with
   `GetNamedPipeServerProcessId`, inspect that process token, and require the
   server SID to match the current user before writing any request. Keep the
   legitimate pipe DACL for server-side client authorization. Independently
   parse every `Ready` destination and accept only the exact Lens URL shape,
   `http://127.0.0.1:<valid-port>`, before browser launch; use a Windows launch
   API that does not route an untrusted value through `cmd.exe` where practical.

   Suggested solution:

   ```plantuml
   @startuml
   title Authenticate the server and constrain the returned destination
   actor User
   participant "Lens client" as Client
   participant "Windows pipe" as Pipe
   participant "Server process token" as Token
   participant Browser

   User -> Client : request target view
   Client -> Pipe : open named-pipe instance
   Client -> Token : resolve server PID and token SID
   alt server SID differs from current user
     Token --> Client : identity mismatch
     Client --> User : reject before sending request
   else server SID matches current user
     Token --> Client : authenticated
     Client -> Pipe : send bounded request
     Pipe --> Client : Ready(request_id, view_url)
     Client -> Client : require http://127.0.0.1:<port>
     Client -> Browser : open validated loopback URL
   end
   @enduml
   ```

   Test coverage: Add a Windows endpoint test in which a pre-existing pipe is
   owned by a deliberately mismatched server identity and verify that the
   client rejects it before transmitting a frame. Add client-response tests
   for non-loopback, non-HTTP, user-info, path-bearing, malformed-port, and
   shell-metacharacter destinations, plus one accepted IPv4 loopback URL.

2. **[Medium] Process-lifetime viewing sessions remain readable by other local
   users — [`src/service/server.rs:223`](../../src/service/server.rs#L223)**

   Explanation and impact: Every successful request is now retained for the
   background process lifetime, but the associated router still serves the
   initial document, every discovered Markdown document, revision values, and
   diagrams to any connection that reaches its loopback port. There is no
   session token or other request authentication in
   [`viewer::routes::router`](../../src/viewer/routes.rs#L17). Loopback limits
   network reachability, not operating-system identity; ADR-022 itself notes
   that multiple users share this namespace. Another local user can scan the
   small TCP port space, recognize a Lens response, and read repository
   documentation long after the invoking client has exited. The foreground
   viewer already had an unauthenticated listener, but this pull request makes
   that exposure materially worse by retaining every listener invisibly, with
   no idle retirement or supported stop command, for the service lifetime.

   Reported behavior and impact:

   ```plantuml
   @startuml
   title Retained loopback session has no per-user authorization
   actor Developer
   actor "Different local user" as OtherUser
   participant "Lens client" as Client
   participant "Background controller" as Controller
   participant "Unauthenticated loopback router" as Router

   Developer -> Client : lens private-repository
   Client -> Controller : Open(target)
   Controller -> Router : create and retain listener
   Client --> Developer : exit after Ready(URL)
   OtherUser -> Router : scan 127.0.0.1 ports and GET /
   Router --> OtherUser : rendered repository document
   note right of Controller
     Session remains for the
     service process lifetime.
   end note
   @enduml
   ```

   Proposed fix: Generate a high-entropy capability for every viewing session
   and require it before serving any document, asset, revision, or diagram.
   Bootstrap an HttpOnly, SameSite cookie from the capability-bearing ready URL
   or consistently retain the capability in every route, then avoid logging or
   exposing it outside the invoking client and browser. Process-lifetime
   retention is then separated from cross-user authorization.

   Suggested solution:

   ```plantuml
   @startuml
   title Require a per-session browser capability
   actor Developer
   actor "Different local user" as OtherUser
   participant "Lens client" as Client
   participant "Protected viewer router" as Router
   participant Browser

   Client -> Router : create session with random capability
   Router --> Client : Ready(loopback URL + capability)
   Client -> Browser : open capability-bearing URL
   Browser -> Router : bootstrap request with capability
   Router --> Browser : set protected session cookie and serve document
   OtherUser -> Router : GET / without capability
   Router --> OtherUser : 401 or indistinguishable 404
   @enduml
   ```

   Test coverage: Add router and compiled-browser checks showing that a request
   without the session capability cannot read the initial document, discovered
   documents, revisions, assets, or diagrams, while the exact ready URL can
   bootstrap a complete page and automatic refresh. Exercise two sessions to
   prove that one capability cannot cross into the other session.

3. **[Medium] The public `lens::open` API cannot cold-start from a library
   consumer — [`src/lib.rs:15`](../../src/lib.rs#L15)**

   Explanation and impact: The crate publishes both a library and a binary, and
   this new function is documented and public. Its missing-service path reaches
   [`spawn_detached_service`](../../src/service/process.rs#L19), which executes
   `std::env::current_exe()` with Lens's hidden service flag. For the repository
   CLI that executable is `lens`, but for an ordinary library caller it is the
   caller's application. Unless that unrelated application happens to parse
   `--lens-background-service` and delegate to Lens, the child exits or runs
   the wrong behavior and `lens::open` reports a startup timeout. It works only
   when some separately installed Lens CLI already owns the endpoint, making a
   public API's behavior depend on unrelated prior process state. Existing tests
   inject an in-process service spawner or invoke the Lens binary, so none
   exercise a second executable that links the library.

   Reported behavior and impact:

   ```plantuml
   @startuml
   title Public library call re-executes the consuming application
   actor "Library user" as User
   participant "consumer-app" as App
   participant "lens::open" as Open
   participant "spawn_detached_service" as Spawn
   participant "consumer-app child" as Child

   User -> App : invoke feature using Lens library
   App -> Open : open(target, scope)
   Open -> Spawn : no service is reachable
   Spawn -> Child : current_exe --lens-background-service
   Child --> Spawn : unknown flag or unrelated behavior
   Open --> App : StartupTimeout after three seconds
   @enduml
   ```

   Proposed fix: Choose and document one supported boundary. If background
   opening is CLI-only, keep the orchestration out of the public Rust API and
   expose only an explicitly internal CLI entry point while retaining
   `lens::serve` for library callers. If library background opening is required,
   accept a verified service executable or launcher from the caller, or install
   a dedicated service binary whose location is resolved independently of the
   consuming process. Do not silently assume `current_exe` implements Lens's
   CLI protocol.

   Suggested solution:

   ```plantuml
   @startuml
   title Make the executable boundary explicit
   actor "Library user" as User
   participant "Public Lens API" as API
   participant "Verified Lens service executable" as Service

   alt background opening is CLI-only
     User -> API : serve(MarkdownTarget)
     API --> User : foreground supported behavior
     note right of API
       Background orchestration remains
       an internal Lens CLI operation.
     end note
   else library background opening is supported
     User -> API : open_with_service(target, executable)
     API -> Service : executable --lens-background-service
     Service --> API : authenticated ready response
     API --> User : ready URL
   end
   @enduml
   ```

   Test coverage: Build a tiny second binary that links the `lens` library and
   calls the supported public API with no existing service. For a public
   background API, verify that the designated Lens service starts and the page
   remains available after the consumer exits. For a CLI-only decision, add a
   compile-time API assertion that background orchestration is not exposed as a
   normal public library function.

4. **[Medium] Lens claims a generic socket name in the shared XDG runtime root
   — [`src/service/endpoint/unix.rs:66`](../../src/service/endpoint/unix.rs#L66)**

   Explanation and impact: When `XDG_RUNTIME_DIR` is set, `runtime_directory`
   returns that per-user directory itself and `endpoint_path` appends only
   `service-v1.sock`. XDG runtime directories are shared by all applications
   for the user; on a normal Linux login this becomes
   `/run/user/<uid>/service-v1.sock`. An unrelated application can plausibly
   use the same generic name. If its listener is active, Lens either treats it
   as the Lens owner or exchanges Lens JSON with it and fails. If its owned
   socket is stale, `remove_verified_stale_socket` verifies only socket type,
   owner, device, and inode, then deletes that other application's endpoint.
   The fallback path is Lens-specific, but the ordinary XDG path is not.

   Reported behavior and impact:

   ```plantuml
   @startuml
   title Generic XDG socket name collides across applications
   participant "Other application" as Other
   participant "$XDG_RUNTIME_DIR" as Runtime
   participant Lens

   Other -> Runtime : create service-v1.sock
   alt other listener is active
     Lens -> Runtime : connect to service-v1.sock
     Runtime --> Lens : unrelated protocol or accepted connection
     Lens --> Lens : fail startup or protocol exchange
   else other socket is stale
     Lens -> Runtime : verify user-owned socket
     Lens -> Runtime : delete service-v1.sock
     Lens -> Runtime : bind Lens endpoint at same name
   end
   @enduml
   ```

   Proposed fix: Create and verify an application-specific private directory,
   such as `$XDG_RUNTIME_DIR/lens`, and place `service-v1.sock` inside it; an
   unambiguous prefixed socket name is a weaker fallback if a subdirectory is
   not viable. Apply the existing ownership, mode, stale-socket, and drop-time
   inode checks within that Lens-owned namespace.

   Suggested solution:

   ```plantuml
   @startuml
   title Isolate the Lens endpoint namespace
   participant Lens
   participant "$XDG_RUNTIME_DIR/lens" as LensRuntime
   participant "Other application endpoint" as Other

   Lens -> LensRuntime : create directory with mode 0700
   Lens -> LensRuntime : verify owner, type, and permissions
   Lens -> LensRuntime : recover only lens/service-v1.sock
   Lens -> LensRuntime : bind user-only Lens socket
   Other --> LensRuntime : separate namespace; remains untouched
   @enduml
   ```

   Test coverage: Set `XDG_RUNTIME_DIR` to a private fixture containing an
   unrelated root-level `service-v1.sock`; claim the Lens endpoint and verify
   that the unrelated socket remains untouched while the endpoint appears only
   below the Lens subdirectory. Retain active-owner, stale recovery, unsafe
   file, concurrent claim, mode, and peer-identity cases for the new path.

5. **[Low] The detached service retains the first invoking directory
   — [`src/service/process.rs:29`](../../src/service/process.rs#L29)**

   Explanation and impact: `service_command` detaches standard streams and the
   process session but does not set a working directory, so the long-lived
   child inherits the first client's directory. The service never needs that
   directory because every request carries its own `invocation_directory`.
   Starting Lens from a removable or separately mounted repository can
   therefore keep that filesystem busy after the command and browser tab are
   gone. With process-lifetime retention and no user-facing stop command, an
   ordinary unmount or eject can continue failing until the user discovers and
   terminates the hidden service.

   Reported behavior and impact:

   ```plantuml
   @startuml
   title Detached service pins the first working filesystem
   actor Developer
   participant "Lens client on mounted repository" as Client
   participant "Detached Lens service" as Service
   participant "Operating system" as OS

   Developer -> Client : lens from /media/repository
   Client -> Service : spawn; inherit /media/repository as cwd
   Client --> Developer : exit after Ready
   Developer -> OS : unmount /media/repository
   OS --> Developer : resource busy while service retains cwd
   @enduml
   ```

   Proposed fix: Configure the child with a stable, non-removable working
   directory before spawning, while continuing to resolve every target from the
   invocation directory carried in the request. Make failure to select or enter
   that directory a typed process-start error rather than silently inheriting
   the caller's directory.

   Suggested solution:

   ```plantuml
   @startuml
   title Detach service lifetime from target filesystem lifetime
   participant "Lens client" as Client
   participant "Service command" as Command
   participant "Detached Lens service" as Service

   Client -> Client : capture invocation_directory in OpenRequest
   Client -> Command : build hidden service command
   Command -> Command : set stable service cwd
   Command -> Service : spawn detached process
   Client -> Service : send captured invocation_directory
   Service -> Service : resolve target from request, not process cwd
   @enduml
   ```

   Test coverage: Assert that `service_command` sets the selected stable current
   directory and still carries only the hidden service argument. Add a native
   lifecycle check that launches the client from a temporary mounted fixture,
   waits for readiness and client exit, and verifies the mount can be released
   while the service remains alive.

6. **[Low] Browser-module documentation links still target the removed path
   — [`docs/iterations/s2-background-viewer-command-contract.md:85`](../iterations/s2-background-viewer-command-contract.md#L85)**

   Explanation and impact: This pull request mechanically moves
   `src/viewer/browser.rs` to `src/browser.rs`, but the new S2 command-contract
   record, the new
   [`S3 design record`](../iterations/s3-background-viewer-service-design.md#L95),
   and the affected pre-existing
   [`M2 browser-module record`](../iterations/m2-browser-launch-module.md#L29)
   still link to the deleted source path. All three links now fail. Readers
   following the background-service analysis or the browser extraction history
   cannot reach the implementation that those records cite, and repository
   link checks cannot distinguish intentional historical wording from an
   accidentally stale source target.

   Reported behavior and impact:

   ```plantuml
   @startuml
   title Browser documentation points to a removed source path
   actor Maintainer
   artifact "S2 command contract" as S2
   artifact "S3 design record" as S3
   artifact "M2 browser module record" as M2
   artifact "src/viewer/browser.rs\nremoved by PR #13" as Removed

   Maintainer -> S2 : follow browser source link
   Maintainer -> S3 : follow browser source link
   Maintainer -> M2 : follow browser source link
   S2 -[#red,dashed]-> Removed : missing target
   S3 -[#red,dashed]-> Removed : missing target
   M2 -[#red,dashed]-> Removed : missing target
   @enduml
   ```

   Proposed fix: Point all three Markdown links at `src/browser.rs`. Historical
   prose may continue to say that the implementation originally lived in
   `viewer::browser`, but clickable source references should resolve to the
   current file unless the repository provides a stable historical permalink.

   Suggested solution:

   ```plantuml
   @startuml
   title Keep historical explanation while linking current source
   actor Maintainer
   artifact "S2, S3, and M2 records" as Docs
   artifact "src/browser.rs\ncurrent implementation" as Browser

   Docs -> Docs : retain historical viewer::browser wording where relevant
   Maintainer -> Docs : follow browser source link
   Docs -> Browser : resolve ../../src/browser.rs
   Browser --> Maintainer : current command construction and tests
   @enduml
   ```

   Test coverage: Update the three links and run the repository's local
   Markdown-target checker across all documentation. The changed-file check
   found the two new broken links; a repository-wide check additionally found
   the M2 link made stale by this move. Keep unrelated pre-existing broken links
   outside this pull request's fix unless their owning work is brought into
   scope.

## Validation

- GitHub's compiled-browser and native Linux, Intel macOS, and Windows jobs all
  passed for PR #13 at `b9c6214`.
- `git diff --check
  1c6cd2d5e620991b3968dd39150c85e9ee1c4c7b...b9c62144a717720aa3c27e26f4c2f9ca2c5022bd`
  passed.
- `cargo fmt --check` passed.
- `cargo test --locked` passed 105 library tests and 5 CLI integration tests.
- `cargo clippy --locked --all-targets --all-features -- -D warnings` passed.
- `cargo package --locked --allow-dirty` built and verified the package.
- `npm run test:browser -- --reporter=line` passed all 26 compiled-browser
  scenarios.
- The complete production diff and its caller, callee, protocol, endpoint,
  process, session-lifetime, target-resolution, browser-launch, and test paths
  were inspected. Microsoft named-pipe security documentation, Tokio 1.35.1's
  client defaults, and Rust's Windows process guidance were checked for the
  Windows finding.
- The worktree contained no unrelated tracked changes before this review
  record was added.
- Local Markdown-target validation checked 197 links in the changed Markdown
  and this review record. The two failures are the new S2 and S3 browser-source
  links reported in finding 6. A repository-wide 870-link pass also identified
  the M2 link made stale by this pull request, plus five unrelated pre-existing
  failures.

## Residual Risks and Validation Gaps

- Native CI proves that platform-specific code compiles and its current tests
  pass, but this review environment is Linux; it did not execute the proposed
  adversarial Windows user-identity scenario or a native browser handoff.
- The intentionally unbounded request ledger, session memory, listener count,
  and polling work remain documented product risks. This review accepted the
  explicit process-lifetime policy and did not repeat the C16 50-session
  measurements.
- Automatic detached startup is covered through an injected in-process service
  spawner, while compiled browser fixtures start their service explicitly. No
  existing automated test performs a cold production re-exec and then proves
  that the detached process has discarded all unneeded process context.
- The background-service documentation contains three new PlantUML blocks;
  their validation status, along with all twelve review diagrams, is recorded
  after server validation below.

## Diagram and Link Validation

- All three background-service diagrams and all twelve finding diagrams
  returned non-empty `image/svg+xml` responses with HTTP 200 from the
  configured default PlantUML server.
- Every source location linked from this review record exists and contains the
  referenced line. The three broken browser-module links are retained as
  findings in their owning documents rather than repeated by this record.
