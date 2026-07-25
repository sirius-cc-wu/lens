---
type: "Software Design"
title: "Server-Only PlantUML Rendering Design"
description: "Defines the Rust collaboration and ownership target for one session-fixed PlantUML server without renderer modes."
feature: "FEAT-01"
status: "implemented"
language: "Rust"
tags: [design, uml, plantuml]
---

# Server-Only PlantUML Rendering Design

Status: implemented in C7

This design realizes `UC-01` and `UC-10` from
[`FEAT-01`](use-cases.md), the diagram request in
[`SSD-01`](ssd-01-open-markdown-target.md), and the guarantees in
[`OC-05`](oc-05-request-diagram.md). It is the target replacement for renderer
selection and the session-disable portions of the current implementation
snapshot in [`uml-design.md`](uml-design.md).

## RZ-05: Start a Session and Request a Diagram from Its PlantUML Server

```plantuml
@startuml
actor Developer
participant "main::main\n<<function>>" as Main
participant "target::load_markdown_target\n<<function>>" as Target
participant "viewer::serve\n<<function>>" as Serve
participant "plantuml::server\n<<function>>" as Server
participant "Process environment" as Environment
participant "markdown::render\n<<function>>" as Render
participant "ViewerState\n<<struct>>" as State
actor "System browser" as Browser
participant "diagram handler\n<<function>>" as Handler
participant "plantuml::svg_url\n<<function>>" as SvgUrl
participant "reqwest::Client\n<<struct>>" as Client
participant "Configured PlantUML server" as PlantUML

Developer -> Main : lens(target?)
Main -> Target : load_markdown_target(target?)
Target --> Main : Result<MarkdownTarget, TargetError>
Main -> Serve : serve(target)
Serve -> Server : server()
Server -> Environment : read LENS_PLANTUML_SERVER
Environment --> Server : configured value or absence
Server --> Serve : normalized base URL or public default
loop each discovered document
  Serve -> Render : render(source)
  Render --> Serve : RenderedDocument
end
Serve -> State : create(documents, plantuml_server, client)
Serve -> Browser : open(loopback URL)

Browser -> Handler : GET /diagrams/{document ID}/{diagram ID}
Handler -> State : resolve known diagram
State --> Handler : authorized source and &plantuml_server
Handler -> SvgUrl : svg_url(&plantuml_server, source)
SvgUrl --> Handler : request URL
Handler -> Client : GET request URL with bounds
Client -> PlantUML : GET /svg/{encoded source}
PlantUML --> Client : SVG or failure
Client --> Handler : bounded SVG or failure
Handler --> Browser : SVG or per-diagram failure
@enduml
```

Failure realization:

- If a non-empty configured server value cannot produce a valid bounded SVG,
  the handler returns the existing per-diagram failure. It does not call
  `plantuml::server` again and cannot select the public default.
- A retry repeats the same Lens-owned diagram route and therefore derives its
  request URL from the same authorized source and session-owned destination.
- There is no renderer-disable system event or browser route.

Responsibility notes:

- `main::main` is the controller for the command invocation. Removing its
  renderer parameter lets the command parser reject every former `--renderer`
  form before it creates a target or viewing session.
- `plantuml::server` is the information expert for the environment-variable
  name, normalization rule, and public default.
- `viewer::serve` is the creator for `ViewerState` because it has the resolved
  target, normalized server, and HTTP client needed to establish the session.
- `ViewerState` owns the server string because its lifetime and immutability
  match the viewing session. It does not need an atomic rendering flag.
- `markdown::render` owns document parsing and authorized diagram-source
  discovery but knows nothing about server configuration. The diagram handler
  combines that stored source with the server borrowed from `ViewerState`; it
  never accepts either value from the browser.

## DCD-04: Rust Module and Type Target

```plantuml
@startuml
hide empty members
skinparam classAttributeIconSize 0

package "crate root" {
  class "main" as Main <<module>> {
    +main(): Result<(), anyhow::Error>
  }
  class "lib" as Lib <<module>> {
    +load_markdown_target(path): Result<MarkdownTarget, TargetError>
    +serve(target): Result<(), anyhow::Error>
  }
}

package "viewer" {
  class "viewer module" as ViewerModule <<module>> {
    +serve(target: MarkdownTarget): Result<(), anyhow::Error>
    -request_diagram(client, plantuml_server, diagram): Result<Vec<u8>, anyhow::Error>
  }
  class ViewerState <<struct>> {
    -documents: RwLock<Vec<ViewerDocument>>
    -plantuml_server: String
    -client: reqwest::Client
  }
}

package "markdown" {
  class "markdown module" as MarkdownModule <<module>> {
    +render(markdown, document_id, current_document, known_documents): RenderedDocument
  }
  class RenderedDocument <<struct>> {
    +html: String
    +diagrams: Vec<Diagram>
  }
  class Diagram <<struct>> {
    +source: String
  }
}

package "plantuml" {
  class "plantuml module" as PlantUmlModule <<module>> {
    ~server(): String
    +svg_url(server, source): String
  }
}

Main --> Lib : calls
Lib --> ViewerModule : re-exports serve
ViewerModule --> PlantUmlModule : reads server and builds URLs
ViewerModule --> ViewerState : creates
ViewerState *-- "1" "String\nPlantUML server" : owns
ViewerState *-- "1" "reqwest::Client" : owns
ViewerModule ..> MarkdownModule : renders documents
MarkdownModule --> RenderedDocument : creates
RenderedDocument *-- "0..*" Diagram : owns
@enduml
```

Rust adaptation:

- The single server path is a concrete value, not an open or closed family of
  rendering algorithms. A `String` owned by `ViewerState` plus cohesive
  `plantuml` module functions is the smallest native mechanism.
- `RendererMode` and the multi-variant `DiagramRenderer` enum are removed
  rather than replaced by a one-variant type or trait.
- `serve(target)` reads process configuration once at the composition root.
  The request function borrows its server and diagram values for temporary
  collaboration; no stored borrow or new lifetime parameter is required.
- Existing `RwLock` ownership for refreshable documents remains. Removing
  session disable also removes the renderer-specific `AtomicBool`; network I/O
  continues without holding the document lock.

## Construction Result

- CLI parsing, public re-exports, and call-site parameters for `RendererMode`
  were removed. `serve(target)` is enforced by a compile-time function bound.
- `plantuml` now owns only server normalization and source encoding.
- `ViewerState` owns the normalized server string and lends it only while
  requesting a diagram. Markdown rendering and refresh have no server
  dependency.
- The ten-second timeout, 2 MiB response limit, SVG checks, source fallback,
  and retry behavior remain covered by Rust and browser tests.
- The local-command branch, Tokio process features, disable route and state,
  disable markup, client script, styling, and mode-specific tests were removed.
- [`C7`](../../iterations/c7-server-only-plantuml-rendering.md) records the
  red-green evidence and full verification result.
