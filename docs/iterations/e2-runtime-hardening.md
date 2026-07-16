# E2: Runtime Hardening

Status: completed

## Goal

Harden the E1 vertical slice into a production-shaped local workspace boundary
without expanding the browser feature set.

## Risks Addressed

- Incomplete or oversized HTTP requests
- Unbounded file and renderer responses
- Connection starvation and shutdown hangs
- External `curl` runtime dependency
- Unvalidated renderer response content

## Artifacts To Start

- Domain model `DM-01`: canonical notes in
  [`docs/features/lens-viewer.md`](../features/lens-viewer.md) - establishes
  shared vocabulary for workspace, document, block, and diagram concepts.

## Artifacts To Refine

- Feature brief: [`docs/features/lens-viewer.md`](../features/lens-viewer.md) -
  refined with E2 limits, domain model notes, and implementation decisions.
- ADR-001: [`docs/decisions/adr-001-rust-runtime.md`](../decisions/adr-001-rust-runtime.md)
  - refined with the native HTTP renderer evidence.

## Artifacts Consulted

- `SSD-01` through `SSD-03` and `C-01` through `C-02` in the feature brief.
- E1 results: [`e1-lens-inception.md`](e1-lens-inception.md).

## Decisions To Record

- Use `ureq` with Rustls for renderer HTTP requests; pin the dependency graph
  for the supported Rust 1.75 toolchain.
- Bound HTTP headers, file responses, renderer responses, and concurrent
  connections.
- Use connection-level read/write timeouts and a Ctrl-C shutdown channel.

## Trace

- `UC-02` -> path and file limits -> bounded workspace responses
- `UC-03` -> `C-02` -> `HttpRenderer` -> validated SVG response
- `SSD-01` -> `C-01` -> Ctrl-C -> joined workspace shutdown
- `DM-01` -> workspace/document/block/diagram vocabulary

## Exit Criteria

- Native Rust HTTP renderer exchanges a valid SVG response with a local stub.
- Non-SVG, oversized, and failed renderer responses are rejected.
- Malformed and oversized HTTP requests return bounded client errors.
- Oversized files are rejected before their contents are serialized.
- Concurrent workspace requests complete without starving one another.
- Explicit shutdown joins the listener and connection workers.
- Ctrl-C exits the CLI cleanly.

## Results

All exit criteria passed. The server now uses bounded request parsing, 5-second
socket read/write timeouts, a 32-connection cap, 4 MiB file limit, 8 MiB
renderer-response limit, concurrent connection workers, and joined shutdown.
The renderer uses `ureq` with Rustls and validates SVG content type and body.

Verification:

- `cargo test`: 10 passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo fmt --check`: passed.
- CLI startup and Ctrl-C shutdown smoke test: passed.

Residual risks are production HTTP protocol breadth, large-repository indexing
policy, browser asset packaging, and renderer authentication/configuration.

## Artifact Outcomes

- started: `DM-01` - canonical domain model notes in the feature brief.
- refined: `Lens Viewer` - current source of truth for E2 constraints and
  design decisions.
- refined: `ADR-001` - Rustls-based native renderer and Cargo compatibility
  pins recorded.
- started: `E2: Runtime Hardening` - this file - closed with implementation and
  verification evidence.
- deferred: production HTTP framework choice, full browser asset pipeline,
  ignored-file indexing policy, and renderer authentication design.
