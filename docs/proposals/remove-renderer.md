---
type: "Improvement Proposal"
title: "Remove CLI Renderer Selection"
description: "Removes renderer selection, local PlantUML command execution, and the in-page disable control, leaving server-based rendering configured by LENS_PLANTUML_SERVER."
status: "proposed"
tags: [proposal, plantuml, cli]
---

# Remove `--renderer` and Local PlantUML CLI Rendering

Status: proposed

## Summary

Remove the `--renderer` command-line option and stop launching the installed
`plantuml` command to render diagrams (local CLI rendering). Lens will always
use a network service that converts PlantUML source into an image (a PlantUML
server). The document page will no longer offer a control to disable rendering.

When `LENS_PLANTUML_SERVER` is unset or empty, Lens will use the default online
PlantUML server at `https://www.plantuml.com/plantuml`. A non-empty
`LENS_PLANTUML_SERVER` value will replace that base URL for the entire viewing
session.

## Motivation

The three renderer modes make the command-line interface, public Rust API,
viewer state, diagram request path, documentation, and tests carry behavior
that differs only at the final rendering step. Local CLI rendering also
requires process spawning, an independently installed PlantUML command, and
separate timeout, output, and error handling.

The installed PlantUML command is also often outdated, especially when it
comes from an operating-system package repository. This version drift can make
newer PlantUML syntax fail locally or cause the same diagram to render
differently across users' machines. A server-based path centralizes PlantUML
version management and gives every Lens session using that server consistent
rendering behavior.

The in-page disable control does not prevent the initial disclosure of diagram
source because the browser begins requesting diagrams while the page loads. A
control that can only stop later requests adds viewer state and user-interface
complexity without providing a useful privacy boundary.

Using one server-based path makes diagram behavior and failure handling
consistent. It also retains a deployment choice: a user can direct Lens to a
self-hosted or private PlantUML server without allowing a browser request to
choose a command or destination.

## Proposed Behavior

| Situation | Lens behavior |
|---|---|
| `LENS_PLANTUML_SERVER` is unset or contains only whitespace | Use `https://www.plantuml.com/plantuml`. |
| `LENS_PLANTUML_SERVER` contains a non-empty base URL | Send every diagram request in the viewing session to that server. |
| The configured server is invalid, unavailable, or rejects a diagram | Show the existing per-diagram failure result and keep the PlantUML source visible. Do not fall back to the default server. |
| The user passes `--renderer` | Exit with the command-line parser's unknown-argument error. |

Lens will continue to append `/svg/<encoded-source>` to the selected server
base URL. It will trim surrounding whitespace and trailing `/` characters from
the environment value as it does today. The server choice is read when the
viewing session starts and is not accepted from browser routes.

The page should describe the active path as PlantUML server rendering rather
than report a `public`, `local`, or `disabled` renderer mode. It must not expose
the configured server URL in a browser route or accept a replacement URL from
the page. It will not offer a control to disable diagram rendering.

## Scope

Implementation of this proposal will:

- remove `--renderer public|local|disabled` from CLI parsing and help;
- remove `RendererMode` from the public Rust API;
- change `serve(target, renderer_mode)` to a server-configured
  `serve(target)` entry point;
- remove the local `plantuml -tsvg -pipe` process path and its tests;
- remove the renderer enum variants and retain one server-backed diagram path;
- remove the in-page disable control, the `/renderer/disable` route, its
  session-state flag, disable-specific JavaScript and styling, and their tests;
- preserve the ten-second request timeout, 2 MiB response limit, SVG content
  checks, visible source fallback, and per-diagram retry control;
- preserve `LENS_PLANTUML_SERVER` for controlled browser tests and document it
  as the supported way to select another PlantUML server; and
- update CLI examples, requirements, use cases, glossary terms, risks,
  architecture decisions, iteration history, and release notes that describe
  renderer modes.

Historical records should not be rewritten to imply local CLI rendering never
existed. [ADR-009](../decisions/adr-009-selectable-plantuml-rendering.md) and
the [implemented local-rendering proposal](../improvement-proposals.md#1-local-plantuml-rendering)
should instead be marked as superseded by the decision that accepts this
proposal. [ADR-005](../decisions/adr-005-controlled-renderer-browser-tests.md)
should be updated or superseded because `LENS_PLANTUML_SERVER` will be a
supported server-selection mechanism rather than only an automated-test hook.
The session-disable portion of
[ADR-011](../decisions/adr-011-diagram-failure-controls.md) and
[Proposal 5](../improvement-proposals.md#5-diagram-failure-controls) should
also be marked as superseded while the per-diagram failure and retry behaviors
remain active.

## Compatibility and Migration

This is a breaking change to both the command-line interface and the exported
Rust library API.

Existing commands migrate as follows:

```text
lens --renderer public docs    -> lens docs
lens --renderer local docs     -> LENS_PLANTUML_SERVER=<server> lens docs
lens --renderer disabled docs  -> no startup-time equivalent
```

A user who previously selected `local` must run or obtain access to a PlantUML
server and set `LENS_PLANTUML_SERVER` to its base URL. Lens will not install,
start, supervise, or discover that server.

Removing the `disabled` CLI value and the in-page disable control means Lens no
longer offers a user-facing no-rendering mode. When Lens opens a document
containing PlantUML, it requests each diagram from the configured server. Users
who must keep diagram source within a controlled environment must configure a
server in that environment before starting Lens.

## Rejected Alternatives

### Keep `--renderer` with only `public` and `disabled`

This retains mode selection and its public API solely to support a no-rendering
startup path. It does not meet the proposal's goal of removing `--renderer`.

### Keep the in-page disable control

The control appears only after diagram requests can begin, so it cannot
guarantee that source stays off a server. Removing it avoids presenting a weak
privacy control and eliminates session state that is unrelated to the single
server-rendering path.

### Replace `--renderer` with `--plantuml-server`

The environment variable already selects the server without adding another
command-line configuration path. A second input would require precedence and
conflict rules without adding rendering capability.

### Fall back to the default server

If a configured server fails, silently sending the same source to the default
online server would violate the user's destination choice. Server failures
must remain visible and must not trigger failover.

## Acceptance Criteria

- `lens --help` contains no `--renderer` option or renderer-mode values.
- Passing any `--renderer` form exits before a viewing session starts.
- A session with no non-empty override sends PlantUML requests to
  `https://www.plantuml.com/plantuml`.
- A session with `LENS_PLANTUML_SERVER` set sends requests only to that server.
- A configured server failure does not contact the default server.
- Lens never looks up or starts a `plantuml` executable.
- Diagram failures keep their source visible and retain the retry control.
- Document pages contain no diagram-rendering disable control.
- The loopback viewer exposes no `/renderer/disable` route.
- The public `serve` entry point no longer requires a renderer argument.
- User-facing and durable design documentation consistently describes one
  server-based rendering path and the environment override.

## Verification

Automated coverage should include behavior-named scenarios such as:

- `renderer_argument_then_reports_unknown_argument`;
- `missing_plantuml_server_override_then_uses_default_server`;
- `empty_plantuml_server_override_then_uses_default_server`;
- `configured_plantuml_server_then_uses_only_that_server`;
- `unavailable_configured_server_then_does_not_contact_default_server`; and
- `document_page_then_omits_rendering_disable_control`.

Each new or changed test should use explicit setup, one primary action, and
verification sections.

### Manual end-to-end test

- **Setup:** Build Lens, prepare two documents with one valid PlantUML diagram
  in each, and start a controlled PlantUML server that records requests and
  returns valid SVG.
- **Actions:** Run Lens once with `LENS_PLANTUML_SERVER` unset and confirm a
  diagram through the default online service. Run it again with the environment
  variable pointing to the controlled server, inspect the page controls, and
  navigate to the second document. Attempt a POST request to
  `/renderer/disable`. Finally, run Lens with each former `--renderer` value.
- **Expected result:** The first session renders through the default online
  server. The second sends both diagram requests only to the controlled server,
  presents no rendering-disable control, and returns not found for
  `/renderer/disable`. Each former `--renderer` invocation exits with
  unknown-argument guidance and starts no browser session. No run invokes an
  installed `plantuml` command.

## Out of Scope

- Starting or managing a self-hosted PlantUML server.
- Adding automatic server discovery or failover.
- Adding another way to disable diagram rendering.
- Accepting a PlantUML server URL from a browser request.
- Changing document discovery, diagram encoding, or browser-session
  authorization.
