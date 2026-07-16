use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub type LensResult<T> = Result<T, LensError>;

const MAX_HTTP_REQUEST_BYTES: usize = 16 * 1024;
const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_FILE_CHUNK_BYTES: usize = 256 * 1024;
const MAX_FILE_CHUNK_LINES: usize = 1000;
const MAX_RENDER_RESPONSE_BYTES: usize = 8 * 1024 * 1024;
const MAX_CONNECTIONS: usize = 32;
const HTTP_READ_TIMEOUT: Duration = Duration::from_secs(5);
const RENDER_TIMEOUT: Duration = Duration::from_secs(20);
const DEFAULT_IGNORE_PATTERNS: &[&str] = &[
    ".git",
    "target/",
    "node_modules/",
    ".venv/",
    "__pycache__/",
    "dist/",
    "build/",
];

#[derive(Debug)]
pub enum LensError {
    Io(io::Error),
    InvalidTarget(String),
    Forbidden(String),
    NotFound(String),
    BadRequest(String),
    PayloadTooLarge(String),
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
            | Self::PayloadTooLarge(message)
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

#[derive(Clone, Debug, Default)]
struct IgnoreRules {
    patterns: Vec<IgnorePattern>,
}

#[derive(Clone, Debug)]
struct IgnorePattern {
    pattern: String,
    negated: bool,
    directory_only: bool,
}

impl IgnoreRules {
    fn load(root: &Path) -> LensResult<Self> {
        let mut rules = Self {
            patterns: DEFAULT_IGNORE_PATTERNS
                .iter()
                .filter_map(|pattern| IgnorePattern::parse(pattern))
                .collect(),
        };
        let ignore_path = root.join(".lensignore");
        match fs::read_to_string(&ignore_path) {
            Ok(contents) => {
                rules
                    .patterns
                    .extend(contents.lines().filter_map(IgnorePattern::parse));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(LensError::InvalidTarget(format!(
                    "cannot read {}: {error}",
                    ignore_path.display()
                )))
            }
        }
        Ok(rules)
    }

    fn is_ignored(&self, relative_path: &str, is_directory: bool) -> bool {
        let mut ignored = false;
        for rule in &self.patterns {
            if rule.directory_only && !is_directory {
                continue;
            }
            if rule.matches(relative_path) {
                ignored = !rule.negated;
            }
        }
        ignored
    }
}

impl IgnorePattern {
    fn parse(line: &str) -> Option<Self> {
        let mut pattern = line.trim();
        if pattern.is_empty() || pattern.starts_with('#') {
            return None;
        }
        let negated = pattern.starts_with('!');
        if negated {
            pattern = &pattern[1..];
        }
        let directory_only = pattern.ends_with('/');
        pattern = pattern.trim_start_matches('/').trim_end_matches('/');
        if pattern.is_empty() {
            return None;
        }
        Some(Self {
            pattern: pattern.to_string(),
            negated,
            directory_only,
        })
    }

    fn matches(&self, relative_path: &str) -> bool {
        if self.pattern.contains('/') {
            glob_matches(&self.pattern, relative_path)
        } else {
            relative_path
                .split('/')
                .any(|component| glob_matches(&self.pattern, component))
        }
    }
}

fn glob_matches(pattern: &str, value: &str) -> bool {
    fn matches(pattern: &[u8], value: &[u8]) -> bool {
        if pattern.is_empty() {
            return value.is_empty();
        }
        if pattern[0] == b'*' {
            if pattern.len() > 1 && pattern[1] == b'*' {
                return matches(&pattern[2..], value)
                    || (!value.is_empty() && matches(pattern, &value[1..]));
            }
            return matches(&pattern[1..], value)
                || (!value.is_empty() && value[0] != b'/' && matches(pattern, &value[1..]));
        }
        if pattern[0] == b'?' {
            return !value.is_empty() && value[0] != b'/' && matches(&pattern[1..], &value[1..]);
        }
        !value.is_empty() && pattern[0] == value[0] && matches(&pattern[1..], &value[1..])
    }

    matches(pattern.as_bytes(), value.as_bytes())
}

#[derive(Clone, Debug)]
pub struct Workspace {
    root: PathBuf,
    target: PathBuf,
    kind: TargetKind,
    ignore_rules: IgnoreRules,
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
            let ignore_rules = IgnoreRules::load(&target)?;
            return Ok(Self {
                root: target.clone(),
                target,
                kind: TargetKind::Directory,
                ignore_rules,
            });
        }

        if metadata.is_file() {
            let root = target.parent().ok_or_else(|| {
                LensError::InvalidTarget("target file has no parent directory".to_string())
            })?;
            let ignore_rules = IgnoreRules::load(root)?;
            return Ok(Self {
                root: root.to_path_buf(),
                target,
                kind: TargetKind::File,
                ignore_rules,
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
        let size = fs::metadata(&path)?.len();
        if size > MAX_FILE_BYTES {
            return Err(LensError::PayloadTooLarge(format!(
                "file exceeds the {} MiB limit: {requested}",
                MAX_FILE_BYTES / (1024 * 1024)
            )));
        }
        fs::read_to_string(&path).map_err(|error| {
            LensError::BadRequest(format!("cannot read file {requested}: {error}"))
        })
    }

    pub fn read_file_chunk(
        &self,
        requested: &str,
        start_line: usize,
        line_count: usize,
    ) -> LensResult<FileChunk> {
        if line_count == 0 || line_count > MAX_FILE_CHUNK_LINES {
            return Err(LensError::BadRequest(format!(
                "line count must be between 1 and {MAX_FILE_CHUNK_LINES}"
            )));
        }
        let path = self.resolve_relative(requested)?;
        if !path.is_file() {
            return Err(LensError::BadRequest(format!(
                "workspace path is not a file: {requested}"
            )));
        }
        let total_bytes = fs::metadata(&path)?.len();
        let mut reader = BufReader::new(File::open(&path)?);
        let mut line_number = 0;
        while line_number < start_line {
            if read_limited_line(&mut reader)?.is_none() {
                return Ok(FileChunk {
                    content: String::new(),
                    start_line: start_line + 1,
                    end_line: start_line,
                    total_bytes,
                    has_more: false,
                });
            }
            line_number += 1;
        }

        let first_line = line_number + 1;
        let mut content = String::new();
        let mut lines_read = 0;
        while lines_read < line_count {
            let Some(line) = read_limited_line(&mut reader)? else {
                break;
            };
            if content.len() + line.len() > MAX_FILE_CHUNK_BYTES {
                break;
            }
            content.push_str(&line);
            lines_read += 1;
            line_number += 1;
        }
        let has_more = read_limited_line(&mut reader)?.is_some();
        Ok(FileChunk {
            content,
            start_line: first_line,
            end_line: line_number,
            total_bytes,
            has_more,
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
            let relative_path = self.relative_api_path(&canonical);
            if self
                .ignore_rules
                .is_ignored(&relative_path, metadata.is_dir())
            {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            entries.push(WorkspaceEntry {
                name,
                path: relative_path,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileChunk {
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub total_bytes: u64,
    pub has_more: bool,
}

fn read_limited_line(reader: &mut BufReader<File>) -> LensResult<Option<String>> {
    let mut bytes = Vec::new();
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            if bytes.is_empty() {
                return Ok(None);
            }
            return String::from_utf8(bytes)
                .map(Some)
                .map_err(|_| LensError::BadRequest("file is not valid UTF-8".to_string()));
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let take = newline.map_or(available.len(), |position| position + 1);
        if bytes.len() + take > MAX_FILE_CHUNK_BYTES {
            return Err(LensError::PayloadTooLarge(
                "a file line exceeds the 256 KiB chunk limit".to_string(),
            ));
        }
        bytes.extend_from_slice(&available[..take]);
        reader.consume(take);
        if newline.is_some() {
            return String::from_utf8(bytes)
                .map(Some)
                .map_err(|_| LensError::BadRequest("file is not valid UTF-8".to_string()));
        }
    }
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
    pub closed: bool,
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
                    closed: true,
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
            closed: false,
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

pub struct UnconfiguredRenderer;

impl PlantUmlRenderer for UnconfiguredRenderer {
    fn render(&self, _source: &str) -> LensResult<RenderedDiagram> {
        Err(LensError::Renderer(
            "no PlantUML renderer is configured; use --renderer-url or LENS_RENDERER_URL"
                .to_string(),
        ))
    }
}

pub struct HttpRenderer {
    endpoint: String,
}

impl HttpRenderer {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}

impl PlantUmlRenderer for HttpRenderer {
    fn render(&self, source: &str) -> LensResult<RenderedDiagram> {
        let response = ureq::post(&self.endpoint)
            .set("Content-Type", "text/plain; charset=utf-8")
            .set("Accept", "image/svg+xml")
            .timeout(RENDER_TIMEOUT)
            .send_string(source)
            .map_err(|error| LensError::Renderer(format!("renderer request failed: {error}")))?;
        let content_type = response.header("Content-Type").unwrap_or_default();
        if !content_type
            .to_ascii_lowercase()
            .starts_with("image/svg+xml")
        {
            return Err(LensError::Renderer(format!(
                "renderer returned unexpected content type: {}",
                if content_type.is_empty() {
                    "missing Content-Type"
                } else {
                    content_type
                }
            )));
        }

        let mut body = Vec::new();
        response
            .into_reader()
            .take((MAX_RENDER_RESPONSE_BYTES + 1) as u64)
            .read_to_end(&mut body)
            .map_err(|error| {
                LensError::Renderer(format!("cannot read renderer response: {error}"))
            })?;
        if body.len() > MAX_RENDER_RESPONSE_BYTES {
            return Err(LensError::Renderer(
                "renderer response exceeds the 8 MiB limit".to_string(),
            ));
        }
        if !body.windows(4).any(|window| window == b"<svg") {
            return Err(LensError::Renderer(
                "renderer response is not SVG content".to_string(),
            ));
        }

        Ok(RenderedDiagram {
            content_type: "image/svg+xml".to_string(),
            body,
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
    connections: Arc<Mutex<Vec<JoinHandle<()>>>>,
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
        let workspace = Arc::new(workspace);
        let active_connections = Arc::new(AtomicUsize::new(0));
        let connections = Arc::new(Mutex::new(Vec::new()));
        let thread_stop = Arc::clone(&stop);
        let thread_workspace = Arc::clone(&workspace);
        let thread_renderer = Arc::clone(&renderer);
        let thread_active_connections = Arc::clone(&active_connections);
        let thread_connections = Arc::clone(&connections);
        let join = thread::spawn(move || {
            while !thread_stop.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        if thread_stop.load(Ordering::Acquire) {
                            let _ = stream.shutdown(Shutdown::Both);
                            break;
                        }
                        if thread_active_connections.load(Ordering::Acquire) >= MAX_CONNECTIONS {
                            let _ = write_response(
                                &mut stream,
                                503,
                                "text/plain; charset=utf-8",
                                b"too many concurrent connections",
                            );
                            continue;
                        }

                        thread_active_connections.fetch_add(1, Ordering::AcqRel);
                        let connection_workspace = Arc::clone(&thread_workspace);
                        let connection_renderer = Arc::clone(&thread_renderer);
                        let connection_active = Arc::clone(&thread_active_connections);
                        let connection = thread::spawn(move || {
                            let _guard = ConnectionGuard(connection_active);
                            let _ = serve_connection(
                                stream,
                                connection_workspace.as_ref(),
                                connection_renderer.as_ref(),
                            );
                        });
                        if let Ok(mut connections) = thread_connections.lock() {
                            connections.push(connection);
                        } else {
                            let _ = connection.join();
                        }
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
            connections,
        })
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn url(&self) -> String {
        format!("http://{}", self.address)
    }

    pub fn shutdown(&mut self) {
        self.stop.store(true, Ordering::Release);
        let _ = TcpStream::connect(self.address);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
        self.join_connections();
    }

    pub fn wait(mut self) {
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
        self.join_connections();
        self.stop.store(true, Ordering::Release);
    }

    fn join_connections(&self) {
        let connections = self
            .connections
            .lock()
            .map(|mut connections| connections.drain(..).collect::<Vec<_>>())
            .unwrap_or_default();
        for connection in connections {
            let _ = connection.join();
        }
    }
}

impl Drop for WorkspaceServer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

struct ConnectionGuard(Arc<AtomicUsize>);

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

fn serve_connection(
    mut stream: TcpStream,
    workspace: &Workspace,
    renderer: &dyn PlantUmlRenderer,
) -> LensResult<()> {
    stream.set_read_timeout(Some(HTTP_READ_TIMEOUT))?;
    stream.set_write_timeout(Some(HTTP_READ_TIMEOUT))?;
    match handle_connection(&mut stream, workspace, renderer) {
        Ok(()) => Ok(()),
        Err(error) => write_error(&mut stream, error),
    }
}

fn handle_connection(
    stream: &mut TcpStream,
    workspace: &Workspace,
    renderer: &dyn PlantUmlRenderer,
) -> LensResult<()> {
    let request = read_http_request(stream)?;
    let request_line = request
        .lines()
        .next()
        .ok_or_else(|| LensError::BadRequest("empty HTTP request".to_string()))?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().unwrap_or_default();
    let uri = request_parts.next().unwrap_or_default();
    let version = request_parts.next().unwrap_or_default();
    if request_parts.next().is_some() || !matches!(version, "HTTP/1.0" | "HTTP/1.1") {
        return Err(LensError::BadRequest(
            "invalid HTTP request line".to_string(),
        ));
    }
    if method != "GET" {
        return write_response(
            stream,
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
            optional_query_usize(&params, "startLine")?,
            optional_query_usize(&params, "lineCount")?,
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
            stream,
            response.status,
            &response.content_type,
            &response.body,
        ),
        Err(error) => write_error(stream, error),
    }
}

fn read_http_request(stream: &mut TcpStream) -> LensResult<String> {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        let size = stream.read(&mut buffer)?;
        if size == 0 {
            break;
        }
        if request.len() + size > MAX_HTTP_REQUEST_BYTES {
            return Err(LensError::PayloadTooLarge(
                "HTTP request exceeds the 16 KiB limit".to_string(),
            ));
        }
        request.extend_from_slice(&buffer[..size]);
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }
    if request.is_empty() {
        return Err(LensError::BadRequest("empty HTTP request".to_string()));
    }
    if !request.windows(4).any(|window| window == b"\r\n\r\n") {
        return Err(LensError::BadRequest(
            "incomplete HTTP request headers".to_string(),
        ));
    }
    String::from_utf8(request)
        .map_err(|_| LensError::BadRequest("HTTP request is not UTF-8".to_string()))
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

fn file_response(
    workspace: &Workspace,
    requested: &str,
    start_line: Option<usize>,
    line_count: Option<usize>,
) -> LensResult<HttpResponse> {
    if start_line.is_some() || line_count.is_some() {
        let chunk = workspace.read_file_chunk(
            requested,
            start_line.unwrap_or(0),
            line_count.unwrap_or(MAX_FILE_CHUNK_LINES),
        )?;
        return Ok(file_chunk_response(requested, chunk));
    }

    let content = match workspace.read_file(requested) {
        Ok(content) => content,
        Err(LensError::PayloadTooLarge(_)) => {
            let chunk = workspace.read_file_chunk(requested, 0, MAX_FILE_CHUNK_LINES)?;
            return Ok(file_chunk_response(requested, chunk));
        }
        Err(error) => return Err(error),
    };
    let blocks = extract_plantuml_blocks(&content);
    let blocks = blocks
        .iter()
        .map(|block| {
            format!(
                "{{\"startLine\":{},\"endLine\":{},\"closed\":{}}}",
                block.start_line, block.end_line, block.closed
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let body = format!(
        "{{\"path\":{},\"content\":{},\"partial\":false,\"startLine\":1,\"endLine\":{},\"totalBytes\":{},\"hasMore\":false,\"plantumlBlocks\":[{}]}}",
        json_string(requested),
        json_string(&content),
        content.lines().count(),
        content.len(),
        blocks
    );
    Ok(HttpResponse::ok(
        "application/json; charset=utf-8",
        body.into_bytes(),
    ))
}

fn file_chunk_response(requested: &str, chunk: FileChunk) -> HttpResponse {
    let body = format!(
        "{{\"path\":{},\"content\":{},\"partial\":true,\"startLine\":{},\"endLine\":{},\"totalBytes\":{},\"hasMore\":{},\"plantumlBlocks\":[]}}",
        json_string(requested),
        json_string(&chunk.content),
        chunk.start_line,
        chunk.end_line,
        chunk.total_bytes,
        chunk.has_more
    );
    HttpResponse::ok("application/json; charset=utf-8", body.into_bytes())
}

fn render_response(
    workspace: &Workspace,
    requested: &str,
    block_index: usize,
    renderer: &dyn PlantUmlRenderer,
) -> LensResult<HttpResponse> {
    let diagram = render_block(workspace, requested, block_index, renderer)?;
    if !diagram
        .content_type
        .to_ascii_lowercase()
        .starts_with("image/svg+xml")
        || !diagram.body.windows(4).any(|window| window == b"<svg")
    {
        return Err(LensError::Renderer(
            "renderer response is not valid SVG content".to_string(),
        ));
    }
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

fn optional_query_usize(params: &HashMap<String, String>, name: &str) -> LensResult<Option<usize>> {
    params
        .get(name)
        .map(|value| {
            value
                .parse()
                .map_err(|_| LensError::BadRequest(format!("invalid {name} parameter")))
        })
        .transpose()
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
        413 => "Payload Too Large",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
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
        LensError::PayloadTooLarge(message) => (413, message),
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

const INDEX_HTML: &str = include_str!("../assets/index.html");

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

    fn http_request(address: SocketAddr, request: &str) -> String {
        let mut stream = TcpStream::connect(address).expect("server should accept connections");
        stream
            .write_all(request.as_bytes())
            .expect("request should be written");
        stream
            .shutdown(Shutdown::Write)
            .expect("request should be closed");
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .expect("response should be readable");
        response
    }

    fn renderer_stub(content_type: &str, body: &[u8]) -> (String, JoinHandle<Vec<u8>>) {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let content_type = content_type.to_string();
        let body = body.to_vec();
        let join = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let size = stream.read(&mut buffer).unwrap();
                if size == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..size]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(header.as_bytes()).unwrap();
            stream.write_all(&body).unwrap();
            stream.shutdown(Shutdown::Write).unwrap();
            request
        });
        (format!("http://{address}/render"), join)
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
    fn skips_common_generated_and_vendor_directories_without_indexing_them() {
        let directory = fixture();
        for name in ["target", "node_modules", "dist", "build", ".venv"] {
            fs::create_dir(directory.0.join(name)).unwrap();
            fs::write(directory.0.join(name).join("generated.txt"), "generated").unwrap();
        }
        let workspace = Workspace::from_arg(None, &directory.0).unwrap();
        let entries = workspace.list("").unwrap();
        assert!(entries.iter().any(|entry| entry.name == "src"));
        assert!(!entries.iter().any(|entry| entry.name == "target"));
        assert!(!entries.iter().any(|entry| entry.name == "node_modules"));
        assert!(!entries.iter().any(|entry| entry.name == "dist"));
    }

    #[test]
    fn honors_lensignore_while_allowing_explicit_reads() {
        let directory = fixture();
        fs::create_dir(directory.0.join("private")).unwrap();
        fs::write(directory.0.join("private/notes.md"), "private").unwrap();
        fs::write(directory.0.join("credentials.secret"), "secret").unwrap();
        fs::write(directory.0.join(".lensignore"), "private/\n*.secret\n").unwrap();

        let workspace = Workspace::from_arg(None, &directory.0).unwrap();
        let entries = workspace.list("").unwrap();
        assert!(!entries.iter().any(|entry| entry.name == "private"));
        assert!(!entries
            .iter()
            .any(|entry| entry.name == "credentials.secret"));
        assert_eq!(workspace.read_file("private/notes.md").unwrap(), "private");
        assert_eq!(workspace.read_file("credentials.secret").unwrap(), "secret");
    }

    #[test]
    fn extracts_closed_and_unclosed_plantuml_blocks() {
        let blocks = extract_plantuml_blocks(
            "before\n```plantuml |500\nAlice -> Bob\n```\nafter\n```plantuml\nsecond\n",
        );
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].source, "Alice -> Bob");
        assert_eq!(blocks[0].start_line, 2);
        assert!(blocks[0].closed);
        assert_eq!(blocks[1].source, "second");
        assert!(!blocks[1].closed);
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

        let root = http_request(
            server.address(),
            "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        );
        assert!(root.starts_with("HTTP/1.1 200 OK"));
        assert!(root.contains("<title>Lens</title>"));

        let file = http_request(
            server.address(),
            "GET /api/file?path=README.md HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        );
        assert!(file.contains("\"closed\":true"));

        for request in [
            "GET /api/tree?path= HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            "GET /api/render?path=README.md&block=0 HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        ] {
            let response = http_request(server.address(), request);
            assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
        }
        server.shutdown();
    }

    #[test]
    fn reports_renderer_failure_without_crashing_the_server() {
        let directory = fixture();
        let workspace = Workspace::from_arg(None, &directory.0).unwrap();
        let mut server = WorkspaceServer::start(workspace, Arc::new(FailingRenderer), 0).unwrap();
        let response = http_request(
            server.address(),
            "GET /api/render?path=README.md&block=0 HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        );
        assert!(response.starts_with("HTTP/1.1 502 Bad Gateway"));
        server.shutdown();
    }

    #[test]
    fn uses_the_native_http_renderer_and_validates_svg() {
        let (endpoint, join) =
            renderer_stub("image/svg+xml", b"<?xml version=\"1.0\"?><svg></svg>");
        let renderer = HttpRenderer::new(endpoint);
        let diagram = renderer.render("@startuml\nAlice -> Bob\n@enduml").unwrap();
        assert_eq!(diagram.content_type, "image/svg+xml");
        assert!(diagram.body.ends_with(b"<svg></svg>"));
        let request = join.join().unwrap();
        assert!(String::from_utf8_lossy(&request).contains("Content-Type: text/plain"));
    }

    #[test]
    fn rejects_non_svg_renderer_responses() {
        let (endpoint, join) = renderer_stub("text/plain", b"not a diagram");
        let renderer = HttpRenderer::new(endpoint);
        assert!(matches!(
            renderer.render("source"),
            Err(LensError::Renderer(_))
        ));
        join.join().unwrap();
    }

    #[test]
    fn keeps_remote_rendering_opt_in() {
        assert!(matches!(
            UnconfiguredRenderer.render("source"),
            Err(LensError::Renderer(message)) if message.contains("is configured")
        ));
    }

    #[test]
    fn rejects_malformed_and_oversized_requests() {
        let directory = fixture();
        let workspace = Workspace::from_arg(None, &directory.0).unwrap();
        let mut server = WorkspaceServer::start(workspace, Arc::new(StubRenderer), 0).unwrap();

        let malformed = http_request(
            server.address(),
            "GET / HTTP/2.0\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        );
        assert!(malformed.starts_with("HTTP/1.1 400 Bad Request"));

        let oversized = format!(
            "GET / HTTP/1.1\r\nX-Large: {}\r\n\r\n",
            "x".repeat(MAX_HTTP_REQUEST_BYTES)
        );
        let oversized = http_request(server.address(), &oversized);
        assert!(oversized.starts_with("HTTP/1.1 413 Payload Too Large"));
        server.shutdown();
    }

    #[test]
    fn rejects_oversized_files_and_allows_concurrent_requests() {
        let directory = fixture();
        fs::write(
            directory.0.join("large.txt"),
            vec![b'x'; (MAX_FILE_BYTES + 1) as usize],
        )
        .unwrap();
        fs::write(
            directory.0.join("large.md"),
            "line of source\n".repeat(400_000),
        )
        .unwrap();
        let workspace = Workspace::from_arg(None, &directory.0).unwrap();
        assert!(matches!(
            workspace.read_file("large.txt"),
            Err(LensError::PayloadTooLarge(_))
        ));

        let mut server = WorkspaceServer::start(workspace, Arc::new(StubRenderer), 0).unwrap();
        let address = server.address();
        let large_response = http_request(
            address,
            "GET /api/file?path=large.md HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        );
        assert!(large_response.contains("\"partial\":true"));
        assert!(large_response.len() < 300_000);
        let responses = thread::scope(|scope| {
            let handles = (0..8)
                .map(|_| scope.spawn(|| http_request(address, "GET / HTTP/1.1\r\n\r\n")))
                .collect::<Vec<_>>();
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert!(responses
            .iter()
            .all(|response| response.starts_with("HTTP/1.1 200 OK")));
        server.shutdown();
        assert!(TcpStream::connect(address).is_err());
    }
}
