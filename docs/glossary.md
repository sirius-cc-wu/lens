# Glossary

| Term | Meaning |
|---|---|
| Lens | The standalone CLI and its local browser viewer. |
| Target | The optional file or directory argument passed to `lens`; when omitted, the current working directory is the target. |
| Browser session | The local HTTP session Lens starts and opens in a browser for a resolved target. |
| Markdown document | A supported text file whose content Lens renders as Markdown. The supported extensions are not yet finalized. |
| PlantUML block | A fenced Markdown code block labeled `plantuml` whose contents describe a PlantUML diagram. |
| Diagram renderer | The local or remote service or library that converts PlantUML source into a browser-displayable diagram. |
| Loopback address | A network address reachable only from the same machine, such as `127.0.0.1`; Lens should use it for its browser session by default. |
