# Viewer UML Design Views

Status: target implementation design for proposal 13

These diagrams complement the black-box [SSD-01](ssd-01-open-markdown-target.md)
and [SSD-02](ssd-02-open-document-root.md). They show the runtime collaborators
and the Rust modules selected for proposal 13, including owned state. They do
not introduce additional behavior or abstractions.

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
Serve -> Render : render each discovered document
Render --> Serve : immutable HTML and diagram URLs
Serve -> State : viewer_state(rendered documents, initial document, client)
Serve -> Browser : xdg-open(loopback URL)

Browser -> Handler : GET / or /documents/{id}
Handler -> State : resolve known document ID
Handler --> Browser : rendered document

Browser -> Handler : GET /diagrams/{document ID}/{diagram ID}
Handler -> State : resolve known document ID
Handler -> State : resolve cached diagram URL
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
- `ViewerState` owns pre-rendered documents and identifier lookup tables.
- `markdown::render` is a stateless transformation; it rewrites only known
  document links and creates document-scoped diagram URLs.
- Diagram URLs are computed once at session creation and remain immutable for
  the session lifetime.

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
    +serve(target: MarkdownTarget, renderer_mode: RendererMode): Result<(), anyhow::Error>
  }
  class "browser module" as BrowserModule <<module>> {
    ~open_browser(url): Result<(), std::io::Error>
    -browser_command(platform, url): BrowserCommand
  }
  class "routes module" as RoutesModule <<module>> {
    ~router(state: Arc<ViewerState>): Router
    -document_view(state, document_id, query): Response
    -diagram(state, document_id, diagram_id): Response
  }
  class "page module" as PageModule <<module>> {
    ~page(title, document, navigation, controls): String
    ~navigation_pane(catalog_page, current_document, route): String
  }
  class "rendering module" as RenderingModule <<module>> {
    ~renderer_client(): Result<reqwest::Client, anyhow::Error>
    ~request_diagram(renderer, client, diagram): Result<Vec<u8>, anyhow::Error>
  }
  class "state module" as StateModule <<module>> {
    ~viewer_state(documents, initial_document, client, renderer): Arc<ViewerState>
    ~watch_documents(state): ()
  }
  class "catalog module" as CatalogModule <<module>> {
    ~search(request): CatalogPage
  }
  class ViewerState <<struct>> {
    ~documents: RwLock<Vec<ViewerDocument>>
    ~catalog: DocumentCatalog
    ~known_documents: BTreeSet<String>
    ~initial_document: usize
    ~client: reqwest::Client
    ~renderer: DiagramRenderer
    -rendering_disabled: AtomicBool
  }
  class ViewerDocument <<struct>> {
    ~canonical_path: PathBuf
    ~rendered: RenderedDocument
    ~revision: u64
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
ViewerModule ..> BrowserModule : opens loopback URL
ViewerModule ..> RenderingModule : creates HTTP client
ViewerModule ..> RoutesModule : serves router
ViewerModule ..> StateModule : creates session and starts refresh
RoutesModule ..> PageModule : composes responses
RoutesModule ..> RenderingModule : requests diagrams
RoutesModule --> ViewerState : reads session state
StateModule --> ViewerState : creates and refreshes
StateModule ..> MarkdownModule : renders documents
StateModule ..> CatalogModule : builds authorized catalog
ViewerState *-- "1..*" ViewerDocument : owns
ViewerDocument *-- "1" RenderedDocument : owns
ViewerState *-- "1" "reqwest::Client" : owns
PageModule ..> CatalogModule : renders catalog pages
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
- `ViewerState` remains one session-owned, cross-task value behind `Arc`; its
  document collection stays behind `RwLock`, and the session disable flag stays
  atomic. The split does not add locks or hold a lock across an `.await`.
- The viewer module is the composition root. Route functions coordinate Axum
  requests, while state, page, rendering, catalog, and browser modules own their
  existing specialized behavior and tests.
- JavaScript and CSS remain compile-time-owned data included by the page module
  from dedicated asset files; they do not become runtime filesystem inputs.
- There are no new traits because the renderer alternatives remain the existing
  closed `DiagramRenderer` enum, and the extracted modules introduce no new
  runtime variation point.
