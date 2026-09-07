# PR 16 review: Mermaid rendering

Reviewed `723937f6f32085ecc648248dec5dcb7504608dc2` (merge base with
`main`) through `a630ad36ad879ae1fd24341570693f8b3fe1ff8a` on
`feat/mermaid-diagram-rendering`. Recommendation: address both findings before merging.

1. **[Medium] Prevent diagram configuration from styling the entire document** — [src/viewer/assets/app.js:23](../../src/viewer/assets/app.js#L23)

   Explanation and impact: The new initialization accepts diagram-level CSS configuration from the bundled Mermaid renderer. Its CSS scoping can be escaped, and the page now allows inline styles under its Content Security Policy (CSP). A repository Mermaid fence can therefore hide or alter unrelated document content without running a script. In Chromium, the following input made `getComputedStyle(document.body).display` equal `none`; both diagrams still reported successful rendering, so the error fallback never appeared. Before this change the fence was escaped code. This matches [Mermaid advisory GHSA-87f9-hvmw-gh4p](https://github.com/mermaid-js/mermaid/security/advisories/GHSA-87f9-hvmw-gh4p). External data theft was not demonstrated and is not claimed under Lens's CSP.

   ```text
   %%{init: {"fontFamily": "x;a{b} :not(&){display:none !important} c{d}"}}%%
   flowchart LR
   A-->B
   ```

   Proposed fix: Vendor a maintained Mermaid release containing the CSS-injection fixes and record its version and origin. Also prevent diagram overrides of `fontFamily`, `themeCSS`, `altFontFamily`, and `themeVariables` through Mermaid's `secure` configuration, preserving the existing protected keys. The cited advisory identifies 10.9.6 and 11.15.0 as its first patched releases; select a release covering subsequent advisories too. Merely changing `antiscript` to `strict` does not fix this configuration vulnerability.

   Test coverage: Add a real-browser regression using the shipped assets and CSP. Render this fence beside ordinary text and another diagram; verify both remain visible and their computed styles are unchanged.

   Reported behavior:

   ```plantuml
   @startuml
   participant "Repository diagram" as Source
   participant "Mermaid renderer" as Mermaid
   participant "Document page" as Page
   Source -> Mermaid: Override fontFamily with escaping CSS
   Mermaid -> Page: Insert SVG containing global CSS
   Page -> Page: Apply display:none to body
   note right of Page: Entire document disappears\nNo render rejection occurs
   @enduml
   ```

   Suggested solution:

   ```plantuml
   @startuml
   participant "Repository diagram" as Source
   participant "Patched Mermaid renderer" as Mermaid
   participant "Document page" as Page
   Source -> Mermaid: Request CSS configuration override
   Mermaid -> Mermaid: Keep protected application configuration
   Mermaid -> Page: Insert diagram with confined styles
   note right of Page: Surrounding content remains visible
   @enduml
   ```

2. **[Medium] Replace the Gantt renderer that loops forever on excluded dates** — [src/viewer/assets/app.js:39](../../src/viewer/assets/app.js#L39)

   Explanation and impact: Calling the vendored renderer on the input below enters its synchronous task-date calculation without a termination condition when every weekday is excluded. The rendering promise cannot settle, the catch handlers cannot reveal the source, and the browser main thread cannot process interactions or revision polling. In a fresh Chromium process, a read-only page evaluation remained unresponsive for a five-second deadline after a two-second startup allowance; ordinary diagrams completed within the same test setup. Closing the isolated browser was necessary to end the reproduction. This matches [Mermaid advisory GHSA-6m6c-36f7-fhxh](https://github.com/mermaid-js/mermaid/security/advisories/GHSA-6m6c-36f7-fhxh).

   ```text
   gantt
     excludes monday,tuesday,wednesday,thursday,friday,saturday,sunday
     Task :2025-01-01, 1d
   ```

   Proposed fix: Replace the vendored bundle with a maintained release containing the Gantt termination fix (first included in 10.9.6 and 11.15.0). A promise timeout on the same browser thread cannot interrupt this synchronous loop.

   Test coverage: Add an isolated browser test with an external timeout. Render the all-weekdays-excluded chart followed by a valid diagram, and verify that the page remains responsive, the invalid diagram shows readable source, and the subsequent diagram completes.

   Reported behavior:

   ```plantuml
   @startuml
   participant "Repository diagram" as Source
   participant "Browser main thread" as Browser
   participant "Mermaid Gantt renderer" as Mermaid
   Source -> Browser: Chart excludes all weekdays
   Browser -> Mermaid: render(id, source)
   loop Search for a day that is not excluded
     Mermaid -> Mermaid: Advance date; every date is excluded
   end
   note over Browser, Mermaid: Rendering never returns\nError handling and polling cannot run
   @enduml
   ```

   Suggested solution:

   ```plantuml
   @startuml
   participant "Repository diagram" as Source
   participant "Browser main thread" as Browser
   participant "Patched Gantt renderer" as Mermaid
   Source -> Browser: Chart excludes all weekdays
   Browser -> Mermaid: render(id, source)
   Mermaid -> Mermaid: Detect impossible date schedule
   Mermaid --> Browser: Reject rendering
   Browser -> Browser: Reveal source and continue rendering
   note right of Browser: Page remains responsive
   @enduml
   ```

## Validation and scope

- Inspected all changed files, Markdown escaping and diagram extraction, page asset embedding, route authentication, live-refresh polling, viewer state refresh, and the incidental Rust changes.
- `cargo test --locked`: passed, 128 library tests and 6 CLI tests.
- `cargo fmt --check`: passed.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: passed.
- `node --check src/viewer/assets/mermaid.min.js`: passed.
- Real-browser tests used Playwright with an isolated installed Chrome browser, a temporary loopback HTTP fixture, the exact committed JavaScript assets, UTF-8 responses, and the exact committed CSP. Flowchart, sequence, and class diagrams rendered; malformed syntax revealed source and did not prevent a later valid diagram from rendering. The two findings above were reproduced with these same assets and CSP.
- Browser fixture tests did not exercise the full running Rust server or file watcher. Rust route and state tests cover those layers; end-to-end file editing, Firefox, and Safari were not exercised. The bundled third-party code was examined around relevant rendering and configuration paths, not audited line by line in its entirety.
- All four PlantUML blocks validated against the configured default server, `https://www.plantuml.com/plantuml`: successful SVG responses with no `X-PlantUML-Diagram-Error` headers.
