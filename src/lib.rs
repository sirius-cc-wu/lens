use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub type LensResult<T> = Result<T, LensError>;

#[derive(Debug)]
pub enum LensError {
    Io(io::Error),
    InvalidTarget(String),
    Forbidden(String),
    NotFound(String),
    BadRequest(String),
    Renderer(String),
}

impl Display for LensError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::InvalidTarget(message)
            | Self::Forbidden(message)
            | Self::NotFound(message)
            | Self::BadRequest(message)
            | Self::Renderer(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for LensError {}

impl From<io::Error> for LensError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetKind {
    File,
    Directory,
}

#[derive(Clone, Debug)]
pub struct Workspace {
    root: PathBuf,
    target: PathBuf,
    kind: TargetKind,
}

impl Workspace {
    pub fn from_arg(input: Option<&Path>, cwd: &Path) -> LensResult<Self> {
        let target = match input {
            Some(path) => {
                let path = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    cwd.join(path)
                };
                fs::canonicalize(path).map_err(|error| {
                    LensError::InvalidTarget(format!("cannot resolve target: {error}"))
                })?
            }
            None => find_repository_root(cwd)?,
        };

        let metadata = fs::metadata(&target).map_err(|error| {
            LensError::InvalidTarget(format!(
                "cannot inspect target {}: {error}",
                target.display()
            ))
        })?;

        if metadata.is_dir() {
            return Ok(Self {
                root: target.clone(),
                target,
                kind: TargetKind::Directory,
            });
        }

        if metadata.is_file() {
            let root = target.parent().ok_or_else(|| {
                LensError::InvalidTarget("target file has no parent directory".to_string())
            })?;
            return Ok(Self {
                root: root.to_path_buf(),
                target,
                kind: TargetKind::File,
            });
        }

        Err(LensError::InvalidTarget(format!(
            "target is neither a file nor a directory: {}",
            target.display()
        )))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn target(&self) -> &Path {
        &self.target
    }

    pub fn target_kind(&self) -> TargetKind {
        self.kind
    }

    pub fn target_display_path(&self) -> String {
        self.relative_api_path(&self.target)
    }

    pub fn resolve_relative(&self, requested: &str) -> LensResult<PathBuf> {
        let requested = requested.trim();
        if requested.is_empty() || requested == "." {
            return Ok(self.target.clone());
        }

        let relative = Path::new(requested);
        if relative.is_absolute()
            || relative
                .components()
                .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(LensError::Forbidden(format!(
                "path is outside the workspace: {requested}"
            )));
        }

        if self.kind == TargetKind::File {
            let file_name = self
                .target
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            if requested != file_name {
                return Err(LensError::Forbidden(format!(
                    "path is outside the selected file: {requested}"
                )));
            }
            return Ok(self.target.clone());
        }

        let candidate = self.root.join(relative);
        let canonical = fs::canonicalize(candidate).map_err(|error| {
            LensError::NotFound(format!(
                "cannot resolve workspace path {requested}: {error}"
            ))
        })?;
        if !is_within(&self.root, &canonical) {
            return Err(LensError::Forbidden(format!(
                "path is outside the workspace: {requested}"
            )));
        }
        Ok(canonical)
    }

    pub fn read_file(&self, requested: &str) -> LensResult<String> {
        let path = self.resolve_relative(requested)?;
        if !path.is_file() {
            return Err(LensError::BadRequest(format!(
                "workspace path is not a file: {requested}"
            )));
        }
        fs::read_to_string(&path).map_err(|error| {
            LensError::BadRequest(format!("cannot read file {requested}: {error}"))
        })
    }

    pub fn list(&self, requested: &str) -> LensResult<Vec<WorkspaceEntry>> {
        let path = self.resolve_relative(requested)?;
        if self.kind == TargetKind::File {
            return Ok(vec![WorkspaceEntry {
                name: self
                    .target
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("file")
                    .to_string(),
                path: self.target_display_path(),
                is_directory: false,
            }]);
        }

        if !path.is_dir() {
            return Err(LensError::BadRequest(format!(
                "workspace path is not a directory: {requested}"
            )));
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let candidate = entry.path();
            let canonical = match fs::canonicalize(&candidate) {
                Ok(path) if is_within(&self.root, &path) => path,
                _ => continue,
            };
            let metadata = match fs::metadata(&canonical) {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };
            let name = entry.file_name().to_string_lossy().into_owned();
            entries.push(WorkspaceEntry {
                name,
                path: self.relative_api_path(&canonical),
                is_directory: metadata.is_dir(),
            });
        }
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(entries)
    }

    fn relative_api_path(&self, path: &Path) -> String {
        if self.kind == TargetKind::File {
            return self
                .target
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
        }
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
}

fn find_repository_root(cwd: &Path) -> LensResult<PathBuf> {
    let mut current = fs::canonicalize(cwd).map_err(|error| {
        LensError::InvalidTarget(format!("cannot resolve current directory: {error}"))
    })?;
    loop {
        if current.join(".git").exists() {
            return Ok(current);
        }
        if !current.pop() {
            break;
        }
    }
    Err(LensError::InvalidTarget(
        "no Git repository found for the current directory".to_string(),
    ))
}

fn is_within(root: &Path, candidate: &Path) -> bool {
    candidate == root || candidate.starts_with(root)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlantUmlBlock {
    pub source: String,
    pub start_line: usize,
    pub end_line: usize,
}

pub fn extract_plantuml_blocks(markdown: &str) -> Vec<PlantUmlBlock> {
    let mut blocks = Vec::new();
    let mut content: Option<(usize, Vec<String>)> = None;

    for (index, line) in markdown.lines().enumerate() {
        let line_number = index + 1;
        if let Some((start_line, lines)) = content.as_mut() {
            if line.trim() == "```" {
                blocks.push(PlantUmlBlock {
                    source: lines.join("\n"),
                    start_line: *start_line,
                    end_line: line_number,
                });
                content = None;
            } else {
                lines.push(line.to_string());
            }
            continue;
        }

        let trimmed = line.trim();
        let Some(info) = trimmed.strip_prefix("```") else {
            continue;
        };
        let info = info.trim();
        if info == "plantuml" || info.starts_with("plantuml |") {
            content = Some((line_number, Vec::new()));
        }
    }

    if let Some((start_line, lines)) = content {
        blocks.push(PlantUmlBlock {
            source: lines.join("\n"),
            start_line,
            end_line: markdown.lines().count(),
        });
    }

    blocks
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedDiagram {
    pub content_type: String,
    pub body: Vec<u8>,
}

pub trait PlantUmlRenderer: Send + Sync {
    fn render(&self, source: &str) -> LensResult<RenderedDiagram>;
}

pub struct CurlRenderer {
    endpoint: String,
}

impl CurlRenderer {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}

impl PlantUmlRenderer for CurlRenderer {
    fn render(&self, source: &str) -> LensResult<RenderedDiagram> {
        let mut child = Command::new("curl")
            .args([
                "--fail",
                "--silent",
                "--show-error",
                "--max-time",
                "20",
                "--request",
                "POST",
                "--header",
                "Content-Type: text/plain; charset=utf-8",
                "--header",
                "Accept: image/svg+xml",
                "--data-binary",
                "@-",
                &self.endpoint,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| LensError::Renderer(format!("cannot start curl: {error}")))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(source.as_bytes()).map_err(|error| {
                LensError::Renderer(format!("cannot send PlantUML source: {error}"))
            })?;
        }

        let output = child
            .wait_with_output()
            .map_err(|error| LensError::Renderer(format!("renderer request failed: {error}")))?;
        if !output.status.success() {
            let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(LensError::Renderer(if message.is_empty() {
                format!("renderer returned {}", output.status)
            } else {
                message
            }));
        }
        if output.stdout.is_empty() {
            return Err(LensError::Renderer(
                "renderer returned an empty diagram".to_string(),
            ));
        }

        Ok(RenderedDiagram {
            content_type: "image/svg+xml".to_string(),
            body: output.stdout,
        })
    }
}

pub fn render_block(
    workspace: &Workspace,
    requested: &str,
    block_index: usize,
    renderer: &dyn PlantUmlRenderer,
) -> LensResult<RenderedDiagram> {
    let source = workspace.read_file(requested)?;
    let block = extract_plantuml_blocks(&source)
        .get(block_index)
        .cloned()
        .ok_or_else(|| LensError::NotFound(format!("PlantUML block not found: {block_index}")))?;
    if block.source.trim().is_empty() {
        return Err(LensError::BadRequest(format!(
            "PlantUML block {block_index} is empty"
        )));
    }
    renderer.render(&block.source)
}

pub struct WorkspaceServer {
    address: SocketAddr,
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl WorkspaceServer {
    pub fn start(
        workspace: Workspace,
        renderer: Arc<dyn PlantUmlRenderer>,
        port: u16,
    ) -> LensResult<Self> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;
        listener.set_nonblocking(true)?;
        let address = listener.local_addr()?;
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let join = thread::spawn(move || {
            while !thread_stop.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let _ = handle_connection(stream, &workspace, renderer.as_ref());
                    }
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            address,
            stop,
            join: Some(join),
        })
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn url(&self) -> String {
        format!("http://{}", self.address)
    }

    pub fn shutdown(&mut self) {
        if self.stop.swap(true, Ordering::Release) {
            return;
        }
        let _ = TcpStream::connect(self.address);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }

    pub fn wait(mut self) {
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
        self.stop.store(true, Ordering::Release);
    }
}

impl Drop for WorkspaceServer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn handle_connection(
    mut stream: TcpStream,
    workspace: &Workspace,
    renderer: &dyn PlantUmlRenderer,
) -> LensResult<()> {
    let mut buffer = [0_u8; 64 * 1024];
    let size = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..size]);
    let request_line = request
        .lines()
        .next()
        .ok_or_else(|| LensError::BadRequest("empty HTTP request".to_string()))?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().unwrap_or_default();
    let uri = request_parts.next().unwrap_or_default();
    if method != "GET" {
        return write_response(
            &mut stream,
            405,
            "text/plain; charset=utf-8",
            b"only GET is supported",
        );
    }

    let (path, query) = uri.split_once('?').unwrap_or((uri, ""));
    let params = parse_query(query)?;
    let response = match path {
        "/" => Ok(HttpResponse::ok(
            "text/html; charset=utf-8",
            INDEX_HTML.as_bytes().to_vec(),
        )),
        "/api/tree" => tree_response(
            workspace,
            params.get("path").map(String::as_str).unwrap_or(""),
        ),
        "/api/file" => file_response(
            workspace,
            params.get("path").map(String::as_str).unwrap_or(""),
        ),
        "/api/render" => render_response(
            workspace,
            params.get("path").map(String::as_str).unwrap_or(""),
            params
                .get("block")
                .ok_or_else(|| LensError::BadRequest("missing block parameter".to_string()))?
                .parse()
                .map_err(|_| LensError::BadRequest("invalid block parameter".to_string()))?,
            renderer,
        ),
        _ => Err(LensError::NotFound(format!("route not found: {path}"))),
    };

    match response {
        Ok(response) => write_response(
            &mut stream,
            response.status,
            &response.content_type,
            &response.body,
        ),
        Err(error) => write_error(&mut stream, error),
    }
}

struct HttpResponse {
    status: u16,
    content_type: String,
    body: Vec<u8>,
}

impl HttpResponse {
    fn ok(content_type: impl Into<String>, body: Vec<u8>) -> Self {
        Self {
            status: 200,
            content_type: content_type.into(),
            body,
        }
    }
}

fn tree_response(workspace: &Workspace, requested: &str) -> LensResult<HttpResponse> {
    let entries = workspace.list(requested)?;
    let body = entries
        .iter()
        .map(|entry| {
            format!(
                "{{\"name\":{},\"path\":{},\"directory\":{}}}",
                json_string(&entry.name),
                json_string(&entry.path),
                entry.is_directory
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    Ok(HttpResponse::ok(
        "application/json; charset=utf-8",
        format!("[{}]", body).into_bytes(),
    ))
}

fn file_response(workspace: &Workspace, requested: &str) -> LensResult<HttpResponse> {
    let content = workspace.read_file(requested)?;
    let blocks = extract_plantuml_blocks(&content);
    let blocks = blocks
        .iter()
        .map(|block| {
            format!(
                "{{\"startLine\":{},\"endLine\":{}}}",
                block.start_line, block.end_line
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let body = format!(
        "{{\"path\":{},\"content\":{},\"plantumlBlocks\":[{}]}}",
        json_string(requested),
        json_string(&content),
        blocks
    );
    Ok(HttpResponse::ok(
        "application/json; charset=utf-8",
        body.into_bytes(),
    ))
}

fn render_response(
    workspace: &Workspace,
    requested: &str,
    block_index: usize,
    renderer: &dyn PlantUmlRenderer,
) -> LensResult<HttpResponse> {
    let diagram = render_block(workspace, requested, block_index, renderer)?;
    Ok(HttpResponse {
        status: 200,
        content_type: diagram.content_type,
        body: diagram.body,
    })
}

fn parse_query(query: &str) -> LensResult<HashMap<String, String>> {
    let mut params = HashMap::new();
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        params.insert(percent_decode(key)?, percent_decode(value)?);
    }
    Ok(params)
}

fn percent_decode(value: &str) -> LensResult<String> {
    let mut bytes = Vec::with_capacity(value.len());
    let raw = value.as_bytes();
    let mut index = 0;
    while index < raw.len() {
        match raw[index] {
            b'+' => bytes.push(b' '),
            b'%' if index + 2 < raw.len() => {
                let high = hex_value(raw[index + 1])?;
                let low = hex_value(raw[index + 2])?;
                bytes.push((high << 4) | low);
                index += 2;
            }
            b'%' => return Err(LensError::BadRequest("invalid URL escape".to_string())),
            byte => bytes.push(byte),
        }
        index += 1;
    }
    String::from_utf8(bytes).map_err(|_| LensError::BadRequest("query is not UTF-8".to_string()))
}

fn hex_value(value: u8) -> LensResult<u8> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(LensError::BadRequest("invalid URL escape".to_string())),
    }
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> LensResult<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        502 => "Bad Gateway",
        _ => "Internal Server Error",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    stream.shutdown(Shutdown::Write)?;
    Ok(())
}

fn write_error(stream: &mut TcpStream, error: LensError) -> LensResult<()> {
    let (status, message) = match error {
        LensError::Forbidden(message) => (403, message),
        LensError::NotFound(message) => (404, message),
        LensError::BadRequest(message) => (400, message),
        LensError::Renderer(message) => (502, message),
        LensError::InvalidTarget(message) => (500, message),
        LensError::Io(error) => (500, error.to_string()),
    };
    write_response(
        stream,
        status,
        "text/plain; charset=utf-8",
        message.as_bytes(),
    )
}

fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped.push('"');
    escaped
}

#[cfg(target_os = "linux")]
pub fn open_browser(url: &str) -> io::Result<()> {
    Command::new("xdg-open").arg(url).spawn().map(|_| ())
}

#[cfg(target_os = "macos")]
pub fn open_browser(url: &str) -> io::Result<()> {
    Command::new("open").arg(url).spawn().map(|_| ())
}

#[cfg(target_os = "windows")]
pub fn open_browser(url: &str) -> io::Result<()> {
    Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()
        .map(|_| ())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn open_browser(_url: &str) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "automatic browser launch is unsupported on this platform",
    ))
}

const INDEX_HTML: &str = r###"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Lens</title>
  <style>
    :root { color-scheme: light dark; font: 15px system-ui, sans-serif; }
    body { margin: 0; display: grid; grid-template-columns: 280px 1fr; min-height: 100vh; }
    aside { padding: 1rem; border-right: 1px solid #8885; overflow: auto; }
    main { padding: 1rem 2rem; overflow: auto; }
    button { display: block; border: 0; background: transparent; color: inherit; padding: .35rem; text-align: left; cursor: pointer; }
    button:hover { background: #8883; }
    pre { white-space: pre-wrap; overflow-wrap: anywhere; }
    img { max-width: 100%; background: white; }
    .directory { font-weight: 700; }
    .error { color: #d66; }
    @media (max-width: 720px) { body { grid-template-columns: 1fr; } aside { border-right: 0; border-bottom: 1px solid #8885; max-height: 35vh; } }
  </style>
</head>
<body>
  <aside><h1>Lens</h1><div id="tree">Loading...</div></aside>
  <main><h2 id="title">Select a file</h2><div id="content"></div></main>
  <script>
    const tree = document.querySelector('#tree');
    const title = document.querySelector('#title');
    const content = document.querySelector('#content');
    async function loadTree(path = '') {
      const response = await fetch('/api/tree?path=' + encodeURIComponent(path));
      const entries = await response.json();
      tree.replaceChildren();
      for (const entry of entries) {
        const button = document.createElement('button');
        button.textContent = (entry.directory ? '[dir] ' : '[file] ') + entry.name;
        button.className = entry.directory ? 'directory' : '';
        button.onclick = () => entry.directory ? loadTree(entry.path) : loadFile(entry.path);
        tree.append(button);
      }
    }
    async function loadFile(path) {
      const response = await fetch('/api/file?path=' + encodeURIComponent(path));
      const file = await response.json();
      title.textContent = file.path;
      content.replaceChildren();
      const source = document.createElement('pre');
      source.textContent = file.content;
      content.append(source);
      for (let index = 0; index < file.plantumlBlocks.length; index++) {
        const image = document.createElement('img');
        image.alt = 'PlantUML diagram';
        image.src = '/api/render?path=' + encodeURIComponent(path) + '&block=' + index;
        image.onerror = () => image.replaceWith(document.createTextNode('Diagram render failed.'));
        content.append(image);
      }
    }
    loadTree().catch(error => { tree.textContent = error; });
  </script>
</body>
</html>"###;

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpStream;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let suffix = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be after epoch")
                .as_nanos();
            let path =
                std::env::temp_dir().join(format!("lens-test-{}-{suffix}", std::process::id()));
            fs::create_dir_all(&path).expect("temporary directory should be created");
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    struct StubRenderer;

    impl PlantUmlRenderer for StubRenderer {
        fn render(&self, source: &str) -> LensResult<RenderedDiagram> {
            Ok(RenderedDiagram {
                content_type: "image/svg+xml".to_string(),
                body: format!("<svg data-source-length=\"{}\"></svg>", source.len()).into_bytes(),
            })
        }
    }

    struct FailingRenderer;

    impl PlantUmlRenderer for FailingRenderer {
        fn render(&self, _source: &str) -> LensResult<RenderedDiagram> {
            Err(LensError::Renderer("renderer unavailable".to_string()))
        }
    }

    fn fixture() -> TempDir {
        let directory = TempDir::new();
        fs::create_dir(directory.0.join(".git")).expect("git marker should be created");
        fs::create_dir(directory.0.join("src")).expect("source directory should be created");
        fs::write(
            directory.0.join("README.md"),
            "# Example\n\n```plantuml\n@startuml\nAlice -> Bob: hello\n@enduml\n```\n",
        )
        .expect("fixture should be written");
        fs::write(directory.0.join("src/main.rs"), "fn main() {}\n")
            .expect("source fixture should be written");
        directory
    }

    #[test]
    fn resolves_repository_directory_and_file_targets() {
        let directory = fixture();
        let root = Workspace::from_arg(None, &directory.0).expect("repository should resolve");
        assert_eq!(root.target_kind(), TargetKind::Directory);
        assert_eq!(root.root(), directory.0.canonicalize().unwrap());

        let explicit_directory = Workspace::from_arg(Some(Path::new(".")), &directory.0)
            .expect("directory argument should resolve");
        assert_eq!(explicit_directory.target_kind(), TargetKind::Directory);

        let file = Workspace::from_arg(Some(Path::new("README.md")), &directory.0)
            .expect("file argument should resolve");
        assert_eq!(file.target_kind(), TargetKind::File);
        assert!(file.read_file("README.md").unwrap().contains("plantuml"));
    }

    #[test]
    fn rejects_traversal_and_external_symlinks() {
        let directory = fixture();
        let workspace = Workspace::from_arg(None, &directory.0).unwrap();
        assert!(matches!(
            workspace.resolve_relative("../outside"),
            Err(LensError::Forbidden(_))
        ));

        let outside = TempDir::new();
        fs::write(outside.0.join("secret.txt"), "secret").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(
            outside.0.join("secret.txt"),
            directory.0.join("outside-link.txt"),
        )
        .unwrap();
        #[cfg(unix)]
        assert!(!workspace
            .list("")
            .unwrap()
            .iter()
            .any(|entry| entry.name == "outside-link.txt"));
    }

    #[test]
    fn extracts_closed_and_unclosed_plantuml_blocks() {
        let blocks = extract_plantuml_blocks(
            "before\n```plantuml |500\nAlice -> Bob\n```\nafter\n```plantuml\nsecond\n",
        );
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].source, "Alice -> Bob");
        assert_eq!(blocks[0].start_line, 2);
        assert_eq!(blocks[1].source, "second");
    }

    #[test]
    fn renders_a_block_through_the_injected_adapter() {
        let directory = fixture();
        let workspace = Workspace::from_arg(None, &directory.0).unwrap();
        let diagram = render_block(&workspace, "README.md", 0, &StubRenderer).unwrap();
        assert_eq!(diagram.content_type, "image/svg+xml");
        assert!(String::from_utf8(diagram.body)
            .unwrap()
            .contains("data-source-length"));
    }

    #[test]
    fn serves_tree_file_and_render_routes() {
        let directory = fixture();
        let workspace = Workspace::from_arg(None, &directory.0).unwrap();
        let mut server = WorkspaceServer::start(workspace, Arc::new(StubRenderer), 0).unwrap();

        for request in [
            "GET /api/tree?path= HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            "GET /api/file?path=README.md HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            "GET /api/render?path=README.md&block=0 HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        ] {
            let mut stream = TcpStream::connect(server.address()).unwrap();
            stream.write_all(request.as_bytes()).unwrap();
            stream.shutdown(Shutdown::Write).unwrap();
            let mut response = String::new();
            stream.read_to_string(&mut response).unwrap();
            assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
        }
        server.shutdown();
    }

    #[test]
    fn reports_renderer_failure_without_crashing_the_server() {
        let directory = fixture();
        let workspace = Workspace::from_arg(None, &directory.0).unwrap();
        let mut server = WorkspaceServer::start(workspace, Arc::new(FailingRenderer), 0).unwrap();
        let mut stream = TcpStream::connect(server.address()).unwrap();
        stream
            .write_all(
                b"GET /api/render?path=README.md&block=0 HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        assert!(response.starts_with("HTTP/1.1 502 Bad Gateway"));
        server.shutdown();
    }
}
