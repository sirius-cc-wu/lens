---
type: "Code Review"
title: "PR #12 Review: Open Validated Repository Files in VS Code"
description: "Reviews source-link filesystem authorization, VS Code URL serialization, Markdown link rewriting, refresh behavior, browser security boundaries, and test coverage."
pull_request: 12
status: "completed"
date: "2026-07-26"
tags: [review, source-link, vscode, security]
---

# PR #12 Review: Open Validated Repository Files in VS Code

Review range:
`81de3b220130f16716d3ade4ed35f055b03e9ad5..39c743f25661b5623209c397f38e8f14ce658bff`.
The base is the pull request's fetched `origin/main`; the head is the pull
request's reported head commit.

## Findings

1. **[Medium] Numeric colon segments can make VS Code open the wrong file
   — [`src/source_link.rs:154`](../../src/source_link.rs#L154)**

   Explanation and impact: `vscode_url` emits every colon in a canonical path
   unchanged. VS Code reserves `:line:column` at the end of a
   `vscode://file/` URL, percent-decodes the URI path, and passes that decoded
   path through its line-and-column-aware parser. A validated POSIX file such
   as `report:33` therefore produces `vscode://file/.../report:33`, which VS
   Code interprets as the different file `report` at line 33. If that prefix
   file exists, the user sees the wrong file; otherwise VS Code reports that
   the path does not exist. This violates the requirement that a generated
   editor destination open the exact canonical file Lens validated.

   The syntax is documented under
   [Opening VS Code with URLs](https://code.visualstudio.com/docs/configure/command-line#_opening-vs-code-with-urls).
   VS Code's
   [`parseLineAndColumnAware`](https://github.com/microsoft/vscode/blob/c3707c7be89bff2c6e20e6f863721bc26593d07e/src/vs/base/common/extpath.ts#L369-L392)
   consumes colon-delimited numeric segments. Adding `:` to the percent-encode
   set alone is insufficient because VS Code decodes the path before this
   parsing step.

   Reported behavior and impact:

   ```plantuml
   @startuml
   actor User
   participant Lens
   participant "VS Code URL parser" as VSCode
   User -> Lens : select report:33
   Lens -> VSCode : vscode://file/.../report:33
   VSCode -> VSCode : treat :33 as line selector
   VSCode --> User : opens report at line 33\nor reports missing file
   @enduml
   ```

   Proposed fix: Reject the editor destination (fail closed) when the canonical
   path contains a colon-delimited numeric segment outside the Windows drive
   prefix, unless a supported representation can be proven to preserve the
   exact filesystem path through VS Code's parser. Retain the authored
   destination and existing Lens guidance when the path is ambiguous.

   Suggested solution:

   ```plantuml
   @startuml
   actor User
   participant Lens
   participant Resolver
   User -> Lens : select report:33
   Lens -> Resolver : validate canonical path
   Resolver -> Resolver : detect ambiguous :number segment
   Resolver --> Lens : no editor URL
   Lens --> User : retain authored destination\nand guidance behavior
   @enduml
   ```

   Test coverage: Add Unix resolver cases for files named `report:33` and
   `report:33:4`. Verify that Lens does not generate a misleading `vscode:`
   destination and preserves the authored link.

2. **[Low] Email autolinks can become malformed editor destinations
   — [`src/markdown.rs:60`](../../src/markdown.rs#L60)**

   Explanation and impact: The renderer passes every `Tag::Link` through
   source-file resolution without distinguishing `LinkType::Email`. If a
   regular file named `team@example.com` exists relative to the displayed
   document, Lens resolves the email autolink `<team@example.com>` as a VS Code
   destination while retaining its email link type. Pulldown-Cmark then applies
   the email type's `mailto:` prefix, producing
   `mailto:vscode://file/.../team@example.com` while Lens adds the visible
   `(opens in VS Code)` indication. Selecting the link reaches the wrong
   handler with a malformed destination instead of preserving the authored
   email behavior.

   Reported behavior and impact:

   ```plantuml
   @startuml
   actor User
   participant Lens
   participant "Markdown renderer" as Renderer
   participant Browser
   User -> Lens : request document
   Lens -> Renderer : render <team@example.com>
   Renderer -> Renderer : resolve colliding file\nas VS Code URL
   Renderer -> Browser : emit mailto:vscode://file/...
   Browser --> User : wrong handler and destination
   @enduml
   ```

   Proposed fix: Preserve `LinkType::Email` before document or source-file
   resolution. Only link types that can represent authored filesystem paths
   should reach `resolve_link`.

   Suggested solution:

   ```plantuml
   @startuml
   actor User
   participant Lens
   participant "Markdown renderer" as Renderer
   participant Browser
   User -> Lens : request document
   Lens -> Renderer : render <team@example.com>
   Renderer -> Renderer : preserve LinkType.Email
   Renderer -> Browser : emit mailto:team@example.com
   Browser --> User : opens email handler
   @enduml
   ```

   Test coverage: Render `<team@example.com>` while a colliding regular file
   exists. Assert `href="mailto:team@example.com"` and verify that no
   source-link indication appears.

## Validation

- GitHub's `verify` check passed for PR #12 at the reviewed head.
- `git diff --check origin/main...HEAD` passed.
- `cargo fmt --check` passed.
- `cargo test --locked` passed 74 library tests and 5 CLI integration tests.
- `cargo clippy --locked --all-targets --all-features -- -D warnings` passed.
- `npm run test:browser` passed all 25 browser scenarios.
- `cargo test --locked source_link -- --nocapture` passed all 12 focused
  source-link and refresh checks.
- VS Code's official URL documentation, URI decoding, and
  line-and-column-aware path parser were inspected to confirm the first
  finding.
- Pulldown-Cmark 0.9.3's checked-out HTML renderer was inspected to confirm
  that `LinkType::Email` prefixes its destination with `mailto:`.
- All four review diagrams returned non-empty `image/svg+xml` responses with
  HTTP 200 from the configured default PlantUML server.
- The worktree contained no unrelated tracked changes before this review
  record was added.

## Residual Risks

- Source authorization is necessarily a point-in-time check. A validated file
  can be replaced after rendering and before the user selects its editor URL;
  the design records this external-editor race and keeps it outside Lens's HTTP
  authority.
- Automated browser checks run on Linux and inspect generated destinations
  without invoking the external scheme. The iteration record supplies a manual
  Linux VS Code walkthrough, while native macOS and Windows URL-handler
  behavior remains release evidence.
