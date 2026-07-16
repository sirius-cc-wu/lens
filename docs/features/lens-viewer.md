# Lens Viewer

Status: E4 elaboration baseline

Lens is a CLI that starts a local browser-based viewer for source code and
Markdown documentation in a codebase. It renders PlantUML fenced blocks found
in Markdown without depending on Obsidian.

## System Boundary

Inside Lens:

- CLI argument handling and target resolution
- Local workspace server
- Browser viewer for source files and Markdown documents
- Markdown and PlantUML-block discovery
- Rendering requests and display of returned diagrams

Outside Lens:

- The developer's file system and target repository
- The default web browser
- A configured PlantUML rendering service
- The operating system's process and browser-launch facilities

## Actors

| Actor | Goal |
| --- | --- |
| Developer | Inspect code and documentation in one navigable browser workspace. |
| PlantUML renderer | Convert PlantUML source into a displayable diagram. |
| Operating system | Launch Lens and the default browser for the developer. |

## Use Cases

### UC-01 Start Workspace

Primary actor: Developer

The developer starts `lens` with no argument, a file path, or a directory
path. Lens resolves the target, starts the local viewer, and opens the default
browser. With no argument, the current repository is the target.

Main success scenario:

1. The developer invokes `lens` with an optional target path.
2. Lens resolves the target path and verifies that it is readable.
3. Lens starts the local workspace service.
4. Lens opens the workspace in the default browser.
5. The developer can browse the resolved codebase and documents.

Extensions:

- 1a. No argument: Lens uses the current working repository.
- 2a. The path does not exist or is unreadable: Lens reports a clear error and
  does not open a partial workspace.
- 3a. The requested local address is unavailable: Lens reports the startup
  failure and exits cleanly.

Special requirements:

- Startup must not modify the target codebase.
- A workspace must not expose files outside the resolved target unless the
  user explicitly requests them.
- The browser launch must be optional enough for headless or remote use to be
  testable, even though automatic opening is the normal behavior.

### UC-02 Browse Workspace Content

Primary actor: Developer

The developer navigates files and directories from the resolved workspace and
reads source code and Markdown documents in the browser.

Main success scenario:

1. The developer selects a file or directory in the workspace.
2. Lens returns the corresponding content or directory listing.
3. The browser displays the content with enough context to continue exploring.

The workspace lists directories on demand; Lens does not build a complete
repository index at startup.

Large files are read in bounded line chunks. The browser can request the next
chunk without asking Lens to serialize the entire file into one response.

Extensions:

- 1a. The path is outside the workspace: Lens rejects the request.
- 2a. The file cannot be read or is unsupported: Lens returns an actionable
  error rather than guessing at its contents.

### UC-03 Render PlantUML Block

Primary actor: Developer
Supporting actor: PlantUML renderer

The developer reads a Markdown document and sees its fenced PlantUML blocks as
diagrams while retaining access to the source text.

Main success scenario:

1. The developer opens a Markdown document containing a PlantUML fenced block.
2. Lens identifies the block and sends its PlantUML source to the configured
  renderer.
3. The renderer returns a diagram representation.
4. Lens displays the diagram in the document context and preserves the source
  block for inspection through a source/diagram toggle.

Extensions:

- 2a. The renderer is unavailable or rejects the source: Lens keeps the
  document readable and shows an actionable rendering error.
- 2b. The Markdown contains an empty or malformed block: Lens identifies the
  block issue without preventing the rest of the document from being viewed.
- 4a. Diagram rendering fails: Lens displays the source block and an actionable
  error in the document context.

## System Sequence Summaries

The following system events were exposed by the E1 implementation slice. The
Lens workspace is treated as one black-box system; its HTTP routes are concrete
operation names for the prototype, not a commitment to the final transport.

### SSD-01 Start Workspace

1. Developer -> Lens CLI: `startWorkspace(targetPath?)`
2. Lens CLI -> Operating system: `openBrowser(workspaceUrl)`
3. Lens CLI -> Developer: `workspaceReady(workspaceUrl)`

Extensions:

- `targetPath` is omitted: Lens resolves the current Git repository.
- Browser launch fails: Lens reports the URL and keeps the workspace available
  for manual opening.
- Target resolution fails: Lens returns an error and does not start the
  workspace.

### SSD-02 Browse Workspace Content

1. Developer -> Lens workspace: `listDirectory(path)`
2. Lens workspace -> Developer: `directoryEntries`
3. Developer -> Lens workspace: `readFile(path)`
4. Lens workspace -> Developer: `fileContent(plantumlBlockMetadata)`

Extensions:

- The path is outside the workspace: Lens returns `workspacePathRejected`.
- The path cannot be read: Lens returns `fileReadFailed`.

### SSD-03 Render PlantUML Block

1. Developer -> Lens workspace: `renderPlantUml(path, blockIndex)`
2. Lens workspace -> PlantUML renderer: `render(source)`
3. PlantUML renderer -> Lens workspace: `diagram` or `renderFailed`
4. Lens workspace -> Developer: `diagram` or `renderFailed`

## Operation Contracts

### C-01 `startWorkspace(targetPath?)`

Cross references: `UC-01`, `SSD-01`

Preconditions:

- The optional target path is supplied by the developer or the current
  directory is available.

Postconditions:

- A readable workspace target is established.
- A local workspace endpoint is listening for browser requests.
- The browser launch is attempted unless explicitly disabled.
- No file in the target repository is modified.

Open issues:

- Platform-specific process and signal behavior remains to be validated for
  release packaging.

### C-02 `renderPlantUml(path, blockIndex)`

Cross references: `UC-03`, `SSD-03`

Preconditions:

- `path` resolves to a readable file within the workspace.
- `blockIndex` identifies a non-empty PlantUML fence in that file.

Postconditions:

- The selected PlantUML source is sent to the configured renderer.
- A diagram representation is returned to the requesting workspace client, or
  an actionable render failure is returned.
- The source document remains unchanged and readable.

Open issues:

- Alternate diagram formats and renderer authentication remain outside the
  current SVG-only adapter contract.

## DM-01 Domain Model Notes

Conceptual classes:

- `Workspace`: the bounded set of codebase content selected for one Lens
  session.
- `Document`: readable source or Markdown content in a workspace.
- `PlantUML Block`: embedded PlantUML source located within a Markdown
  document.
- `Diagram`: a rendered representation produced from one PlantUML block.
- `PlantUML Renderer`: an external system that converts PlantUML source into a
  diagram.

Associations:

- A `Workspace` contains documents and applies the boundary rules for access.
- A `Document` may contain zero or more PlantUML blocks.
- A PlantUML block produces a diagram through a PlantUML renderer.
- A workspace delegates diagram production to a configured renderer.

Attributes:

- `Workspace.target`: selected file or directory path.
- `Document.path`: workspace-relative path.
- `PlantUML Block.source`: source text and its document line range.
- `Diagram.contentType`: validated representation type, currently SVG.

Rules and questions:

- A workspace must not expose content outside its selected target, including
  symlink targets.
- A renderer failure must not make the source document unreadable.
- The workspace applies built-in generated/vendor patterns plus optional
  `.lensignore` rules while preserving explicit reads.

## E3 Workspace Decisions

- Directory enumeration remains lazy: each directory is read only when the
  browser requests it.
- The default listing skips `.git`, `target`, `node_modules`, `.venv`,
  `__pycache__`, `dist`, and `build` directories. This policy is applied while
  listing and does not scan their contents.
- The browser client is a standalone static asset at `assets/index.html`,
  embedded at compile time instead of being maintained as a Rust string.
- Markdown documents are displayed in source order. PlantUML blocks become
  cards with diagram/source toggles, line ranges, and source-preserving render
  errors.
- `.lensignore` accepts gitignore-style glob patterns. Built-in generated and
  vendor patterns remain active, while explicit file reads are still allowed.
- Large files use bounded line chunks instead of a whole-file JSON response.

## Renderer Configuration And Privacy

- `--renderer-url URL` and `LENS_RENDERER_URL` select the PlantUML POST
  endpoint; the command-line option takes precedence.
- No remote renderer endpoint is selected by default. Remote rendering is
  explicit opt-in.
- Lens sends only the selected PlantUML block source to the configured endpoint
  and does not send repository listings, file paths, or telemetry.
- Use a local or private endpoint when diagram source must not leave the
  machine.

## E4 Document Fidelity Decisions

- Markdown headings, paragraphs, safe links, and non-PlantUML fenced code are
  rendered as document content without executing Markdown HTML.
- A PlantUML block remains a source-preserving card when rendering fails.
- Files larger than the whole-file response limit are exposed through bounded
  line ranges, with the browser requesting continuation explicitly.
- `.lensignore` is the first project-specific filtering mechanism; support for
  importing `.gitignore` rules remains deferred.

## MVP Boundary

### Workspace Startup

- Start from no argument, a file path, or a directory path.
- Resolve no argument to the current repository.
- Launch a local workspace and open the default browser.

### Codebase Browsing

- List directories within the target workspace.
- Display readable source files and Markdown documents.
- Reject traversal outside the target workspace.

### Markdown PlantUML Rendering

- Detect fenced PlantUML blocks in Markdown.
- Render blocks through a configured PlantUML service.
- Display rendered diagrams alongside or in place of the source block.
- Preserve readable source and report render failures.

### Operational Baseline

- Provide actionable startup, file-access, and render errors.
- Shut down the local workspace cleanly.
- Avoid modifying files in the target repository.

Out of scope for the inception baseline: Obsidian integration, Markdown
editing, Mermaid support, diagram export, collaborative access, and production
packaging details.

## Architectural Risks

| Risk | Why it matters | First evidence needed |
| --- | --- | --- |
| Local server and browser lifecycle | The CLI must remain alive while the browser uses the workspace and must stop predictably. | A thin end-to-end launcher serving one fixture repository. |
| Workspace file safety | A code browser must not become an unintended file server. | Path-containment tests covering symlinks, traversal, and target forms. |
| Markdown fence handling | Incorrect extraction will render the wrong source or corrupt document context. | Parser examples for valid, empty, malformed, nested, and multiple blocks. |
| PlantUML renderer integration | Network failures, encoding, response validation, and service choice affect the core value. | A renderer adapter exercised against a stub service and one real compatible endpoint. |
| Large repository responsiveness | Indexing or reading the whole repository at startup may make the CLI unusable. | Fixture and benchmark with ignored/generated directories. |
| Runtime packaging | Supported-platform packaging and bundled asset choices affect distribution. | A packaging smoke test on each supported platform. |

## Inception Decisions

- The product boundary is a standalone CLI plus browser workspace, not an
  Obsidian plugin.
- The first user-visible value is browsing a codebase and rendering PlantUML
  in Markdown.
- The first implementation slice should validate startup, safe file access,
  and one render path together.
- Rust is the MVP runtime recommendation; the evidence and tradeoffs are
  recorded in [ADR-001](../decisions/adr-001-rust-runtime.md).
- The renderer is accessed through a replaceable adapter. The current spike
  uses a native Rust HTTP client for a configurable POST endpoint returning
  validated SVG.

## E2 Elaboration Decisions

- HTTP requests are bounded, parsed as complete request lines, and given read
  and write timeouts.
- File responses are limited to 4 MiB and renderer responses to 8 MiB.
- The workspace accepts at most 32 concurrent connections and joins them on
  shutdown.
- Ctrl-C requests graceful server shutdown; the browser URL remains available
  when automatic browser launch is disabled or unavailable.

## Traceability

- `UC-01` -> startup lifecycle and target-resolution spike
- `UC-02` -> workspace file-access and path-containment spike
- `UC-03` -> Markdown parsing and PlantUML renderer spike
- `SSD-01` -> `C-01` -> CLI and workspace server
- `SSD-03` -> `C-02` -> `PlantUmlRenderer` adapter
- `DM-01` -> `Workspace`, `Document`, `PlantUML Block`, and `Diagram` concepts
- [E1: Lens inception](../iterations/e1-lens-inception.md) records the current
  inception objective; [E2: Runtime hardening](../iterations/e2-runtime-hardening.md)
  records runtime evidence; [E3: Workspace usability](../iterations/e3-workspace-usability.md)
  records scale and browser evidence; [E4: Document fidelity](../iterations/e4-document-fidelity.md)
  records file policy and privacy evidence.
