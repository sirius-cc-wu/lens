# Glossary

| Term | Meaning |
|---|---|
| Lens | The standalone CLI and its local browser viewer. |
| Target | The optional file or directory argument passed to `lens`; when omitted, the current working directory is the target. |
| Document root | The canonical directory Lens authorizes for one viewing session. It is the current directory, a directory target, or the canonical parent of a file target. |
| Document set | The Markdown documents Lens discovers inside a document root and may serve during one viewing session. |
| Document revision | A session-local, monotonically increasing number for one known document. Lens advances it only after successfully rendering changed saved contents. |
| Viewing session | A local loopback session that exposes one selected Markdown document, fixed viewer assets, and the diagrams recognized in that document. |
| Browser session | The local HTTP session Lens starts and opens in a browser for a resolved target. |
| Markdown document | A supported text file whose content Lens renders as Markdown. The supported extensions are not yet finalized. |
| PlantUML block | A fenced Markdown code block labeled `plantuml` whose contents describe a PlantUML diagram. |
| Diagram renderer | The session-fixed PlantUML conversion choice: the default public server, the installed local `plantuml` command, or disabled rendering that leaves source visible. |
| Renderer status | The document-page message identifying the selected renderer or confirming that rendering is disabled for the current viewing session. |
| Loopback address | A network address reachable only from the same machine, such as `127.0.0.1`; Lens should use it for its browser session by default. |
