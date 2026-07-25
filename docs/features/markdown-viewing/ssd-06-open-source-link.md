---
type: "System Sequence Diagram"
title: "SSD-06: Open a Referenced Repository File"
description: "Shows Lens returning a validated editor link and the user-selected handoff from the browser to VS Code."
id: "SSD-06"
use_case: "UC-06"
scenario: "Render and follow a qualifying source-file link."
status: "active"
tags: [analysis, ssd, source-link]
---

# SSD-06: Open a Referenced Repository File

Use case: `UC-06`

Scenario: The developer requests a known Markdown document and selects a
qualifying relative link to a repository source file.

## Actors

- Developer or technical writer
- Operating system browser
- Visual Studio Code

## System Events

1. Operating system browser -> Lens: `request_document(document_id)`
2. Lens -> Operating system browser: `rendered_document` containing an
   indicated, validated `vscode:` source link
3. Developer -> Operating system browser: select the source link
4. Operating system browser -> Visual Studio Code: open the selected
   `vscode:` URL
5. Visual Studio Code -> Developer: display the referenced file

Steps 3 through 5 occur outside the Lens system boundary. Lens returns an
ordinary link but does not observe its selection, receive another request, or
launch an editor process.

## Discovered System Operations

- `request_document(document_id)`: return one document already known to the
  current viewing session, retaining Lens routes for discovered documents and
  generating editor destinations only for qualifying relative source targets.

## Significant Extensions

- A disallowed or unavailable target changes only the returned link
  destination; it introduces no filesystem-path request or editor-launch
  operation.
- A missing `vscode:` handler changes browser and operating-system behavior
  after the response; it introduces no Lens recovery operation.
