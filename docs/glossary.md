---
type: "Glossary"
title: "Lens Glossary"
description: "Defines the project-specific vocabulary used in Lens requirements, design, implementation, and verification artifacts."
status: "active"
tags: [reference, glossary]
---

# Glossary

| Term | Meaning |
|---|---|
| Lens | The standalone CLI and its local browser viewer. |
| Lens client | The short-lived part of an ordinary `lens` invocation that submits one open request, performs the browser handoff, reports immediate failures, and returns control to the terminal. |
| Background Lens service | The per-user Lens process that accepts client requests and keeps one or more isolated viewing sessions available after the invoking clients exit. |
| Target | The optional supported file or directory argument passed to `lens`; when omitted, the current working directory is the target. |
| Document root | The canonical directory Lens authorizes for one viewing session. By default it is the nearest recognized repository containing the target, with the selected directory or file parent as the fallback outside a repository. |
| Target scope | The explicit narrow scope selected with `--scope target`; it authorizes only the selected directory, current directory, or selected file's parent instead of broadening to a repository. |
| Document set | The Markdown and standalone `.puml` documents Lens discovers inside a document root and may serve during one viewing session. |
| Document revision | A session-local, monotonically increasing number for one known document. Lens advances it only after successfully rendering changed saved contents. |
| Viewing session | One isolated local loopback browser context with a canonical document root, fixed discovered document set, target scope, source-link rules, and PlantUML server selection. One background Lens service may host several viewing sessions without merging their authority. |
| Browser session | The local HTTP session Lens starts and opens in a browser for a resolved target. |
| Markdown document | A supported text file whose content Lens renders as Markdown. The supported extensions are not yet finalized. |
| PlantUML block | A fenced Markdown code block labeled `plantuml` whose contents describe a PlantUML diagram. |
| Standalone PlantUML file | A visible `.puml` file in the document set that Lens represents as one diagram with the same server-rendering, source-fallback, and retry behavior as an embedded block. |
| PlantUML server | A network service that converts PlantUML source into an image. Lens fixes one server base URL when a viewing session starts, using `LENS_PLANTUML_SERVER` when its normalized value is non-empty and the public default otherwise. |
| Rendering path status | The document-page message identifying server-based PlantUML rendering without exposing the configured server URL. |
| Loopback address | A network address reachable only from the same machine, such as `127.0.0.1`; Lens should use it for its browser session by default. |
