# V1 UML Design Views

Status: V1 implementation snapshot

These diagrams complement the black-box [SSD-01](ssd-01-open-markdown-target.md)
and [SSD-02](ssd-02-open-document-root.md). They show the implemented Rust
modules, owned state, and runtime collaborators for developer discussion. They
do not introduce additional behavior or abstractions.

## CMP-01: Component and Deployment View

```plantuml
@startuml
left to right direction
skinparam componentStyle rectangle

actor Developer
node "Linux workstation" {
  component "lens binary\n(main)" as CLI
  component "target module" as Target
  component "viewer module\n127.0.0.1 ephemeral port" as Viewer
  component "markdown module" as Markdown
  component "plantuml module" as Encoder
  component "System browser\nxdg-open" as Browser
  database "Document root\nMarkdown files" as Files
}
cloud "Public PlantUML server" as PlantUML

Developer --> CLI : lens [TARGET]
CLI --> Target : load target
Target --> Files : discover and read
CLI --> Viewer : serve document set
Viewer --> Browser : open loopback URL
Browser --> Viewer : GET documents and diagrams
Viewer --> Markdown : render document
Markdown --> Encoder : SVG URL for PlantUML blocks
Viewer --> PlantUML : HTTPS GET SVG
Viewer --> Browser : HTML or SVG
@enduml
```

The browser reaches only the local viewer. The viewer resolves document routes
only through its discovered document set, then retrieves PlantUML SVG through
the public renderer.

## RZ-01: Open and Navigate a Document Root

Use-case realization: `UC-02`, `UC-03`, and `UC-04`

```plantuml
@startuml
actor Developer
participant "main::main\n<<function>>" as Main
participant "target::load_markdown_target\n<<function>>" as Target
database "Filesystem" as Files
participant "viewer::serve\n<<function>>" as Serve
participant "ViewerState\n<<struct>>" as State
actor "System browser" as Browser
participant "Axum document handler\n<<function>>" as Handler
participant "markdown::render\n<<function>>" as Render
participant "reqwest::Client\n<<struct>>" as Client
participant "Public PlantUML server" as PlantUML

Developer -> Main : lens(target?)
Main -> Target : load_markdown_target(target?)
Target -> Files : canonicalize, discover, read
Files --> Target : Markdown documents
Target --> Main : Result<MarkdownTarget, TargetError>
Main -> Serve : serve(target)
Serve -> State : viewer_state(documents, initial_document, client)
Serve -> Browser : xdg-open(loopback URL)

Browser -> Handler : GET / or /documents/{id}
Handler -> State : resolve known document ID
Handler -> Render : render(source, document ID, known IDs)
Render --> Handler : HTML and diagram URLs
Handler --> Browser : rendered document

Browser -> Handler : GET /diagrams/{document ID}/{diagram ID}
Handler -> State : resolve known document ID
Handler -> Render : render(source, document ID, known IDs)
Render --> Handler : selected diagram URL
Handler -> Client : GET SVG with timeout and size limit
Client -> PlantUML : HTTPS GET encoded SVG URL
PlantUML --> Client : SVG or error
Client --> Handler : Result<SVG, error>
Handler --> Browser : SVG or 502
@enduml
```

Responsibility notes:

- `main` is the process boundary: it parses the CLI target once and delegates.
- `target` is the information expert for canonicalization and document discovery.
- `ViewerState` owns the in-memory document set and identifier lookup tables.
- `markdown::render` is a stateless transformation; it rewrites only known
  document links and creates document-scoped diagram URLs.
- The diagram handler re-renders the selected document to obtain a deterministic
  diagram URL rather than storing mutable diagram state.

## DCD-01: Rust Module and Type View

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

package "target" {
  class "target module" as TargetModule <<module>> {
    +load_markdown_target(path?): Result<MarkdownTarget, TargetError>
    -discover_documents(root): Result<Vec<MarkdownDocument>, TargetError>
  }
  class MarkdownTarget <<struct>> {
    -documents: Vec<MarkdownDocument>
    -initial_document: usize
    ~into_parts(self): (Vec<MarkdownDocument>, usize)
  }
  class MarkdownDocument <<struct>> {
    ~identifier: String
    ~canonical_path: PathBuf
    ~source: String
  }
  enum TargetError <<enum>> {
    Missing
    Unreadable
    UnsupportedTarget
    NoMarkdownDocuments
  }
}

package "viewer" {
  class "viewer module" as ViewerModule <<module>> {
    +serve(target: MarkdownTarget): Result<(), anyhow::Error>
    -router(state: Arc<ViewerState>): Router
    -request_diagram(client, diagram): Result<Vec<u8>, anyhow::Error>
  }
  class ViewerState <<struct>> {
    -documents: Vec<MarkdownDocument>
    -document_ids: BTreeMap<String, usize>
    -known_documents: BTreeSet<String>
    -initial_document: usize
    -client: reqwest::Client
  }
}

package "markdown" {
  class "markdown module" as MarkdownModule <<module>> {
    +render(markdown, document_id, current_document, known_documents): RenderedDocument
    +escape_html(value): String
  }
  class RenderedDocument <<struct>> {
    +html: String
    +diagrams: Vec<Diagram>
  }
  class Diagram <<struct>> {
    +url: String
  }
}

package "plantuml" {
  class "plantuml module" as PlantUmlModule <<module>> {
    +svg_url(source): String
  }
}

Main --> Lib : calls
Lib --> TargetModule : re-exports function
Lib --> ViewerModule : re-exports function
TargetModule --> MarkdownTarget : creates
MarkdownTarget *-- "1..*" MarkdownDocument : owns
ViewerModule --> MarkdownTarget : consumes
ViewerModule --> ViewerState : creates
ViewerState *-- "1..*" MarkdownDocument : owns
ViewerState *-- "1" "reqwest::Client" : owns
ViewerModule ..> MarkdownModule : renders documents
MarkdownModule --> RenderedDocument : creates
RenderedDocument *-- "0..*" Diagram : owns
MarkdownModule ..> PlantUmlModule : builds diagram URL
@enduml
```

Rust adaptation notes:

- The diagram uses `<<module>>` for cohesive free functions and `<<struct>>` or
  `<<enum>>` only for actual Rust types.
- Composition denotes owned fields. Dependencies denote parameter-only or
  function-call collaboration.
- `MarkdownTarget::into_parts(self)` consumes the target at the transition to
  the viewer, making the ownership transfer explicit.
- There are no traits because renderer, target, and viewer variation points are
  not open in V1.
