use std::{path::PathBuf, time::Duration};

use thiserror::Error;
use tokio::time::{sleep, timeout, Instant};

use super::{
    endpoint::{self, ClientConnection, EndpointError},
    process::{self, ProcessError},
    protocol::{
        self, OpenErrorCode, OpenRequest, ProtocolError, ProtocolVersion, RequestId,
        ServiceRequest, ServiceResponse, WirePath,
    },
};
use crate::TargetScope;

const STARTUP_TIMEOUT: Duration = Duration::from_secs(3);
const ACKNOWLEDGMENT_TIMEOUT: Duration = Duration::from_secs(10);
const CONNECT_RETRY_INTERVAL: Duration = Duration::from_millis(25);

#[derive(Debug)]
pub(crate) struct OpenInvocation {
    pub(crate) invocation_directory: PathBuf,
    pub(crate) target: Option<PathBuf>,
    pub(crate) scope: TargetScope,
    pub(crate) plantuml_server: Option<String>,
}

impl OpenInvocation {
    pub(crate) fn capture(
        target: Option<PathBuf>,
        scope: TargetScope,
    ) -> Result<Self, ClientError> {
        Ok(Self {
            invocation_directory: std::env::current_dir().map_err(ClientError::CurrentDirectory)?,
            target,
            scope,
            plantuml_server: std::env::var("LENS_PLANTUML_SERVER").ok(),
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ClientOutcome {
    Opened { view_url: String },
    ManualUrl { view_url: String, error: String },
}

#[derive(Debug, Error)]
pub(crate) enum ClientError {
    #[error("Could not identify the current directory: {0}")]
    CurrentDirectory(std::io::Error),
    #[error("Could not create a request identifier: {0}")]
    RequestIdentifier(String),
    #[error(transparent)]
    Endpoint(#[from] EndpointError),
    #[error(transparent)]
    Process(#[from] ProcessError),
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
    #[error("Lens background service did not start within three seconds")]
    StartupTimeout,
    #[error("Lens background service did not acknowledge the request within ten seconds")]
    AcknowledgmentTimeout,
    #[error("Lens background service returned a response for a different request")]
    MismatchedResponse,
    #[error("Lens background service uses an incompatible command protocol")]
    IncompatibleService,
    #[error("{message}")]
    Rejected {
        code: OpenErrorCode,
        message: String,
    },
}

pub(crate) async fn request_target_view(
    invocation: OpenInvocation,
) -> Result<ClientOutcome, ClientError> {
    request_target_view_with(
        invocation,
        || process::spawn_detached_service().map_err(ClientError::from),
        crate::browser::open_browser,
    )
    .await
}

pub(crate) async fn request_target_view_with<S, B>(
    invocation: OpenInvocation,
    mut spawn_service: S,
    open_browser: B,
) -> Result<ClientOutcome, ClientError>
where
    S: FnMut() -> Result<(), ClientError>,
    B: FnOnce(&str) -> std::io::Result<()>,
{
    let request_id = new_request_id()?;
    let request = ServiceRequest::Open(OpenRequest {
        protocol_version: ProtocolVersion::CURRENT,
        request_id,
        invocation_directory: WirePath::from_path(&invocation.invocation_directory),
        target: invocation.target.as_deref().map(WirePath::from_path),
        scope: invocation.scope,
        plantuml_server: invocation.plantuml_server,
    });
    let mut connection = connect_or_start(&mut spawn_service).await?;
    let response = match exchange(&mut connection, &request).await {
        Err(ExchangeError::Protocol(ProtocolError::Io(_))) => {
            let mut connection = connect_or_start(&mut spawn_service).await?;
            exchange(&mut connection, &request).await?
        }
        result => result?,
    };

    match response {
        ServiceResponse::Ready {
            request_id: response_id,
            view_url,
        } => {
            verify_request_id(request_id, response_id)?;
            match open_browser(&view_url) {
                Ok(()) => Ok(ClientOutcome::Opened { view_url }),
                Err(error) => Ok(ClientOutcome::ManualUrl {
                    view_url,
                    error: error.to_string(),
                }),
            }
        }
        ServiceResponse::Rejected {
            request_id: response_id,
            error,
        } => {
            verify_request_id(request_id, response_id)?;
            Err(ClientError::Rejected {
                code: error.code,
                message: error.message,
            })
        }
        ServiceResponse::Incompatible { .. } => Err(ClientError::IncompatibleService),
    }
}

fn new_request_id() -> Result<RequestId, ClientError> {
    let mut bytes = [0; 16];
    getrandom::getrandom(&mut bytes)
        .map_err(|error| ClientError::RequestIdentifier(error.to_string()))?;
    Ok(RequestId::from_bytes(bytes))
}

async fn connect_or_start<S>(spawn_service: &mut S) -> Result<ClientConnection, ClientError>
where
    S: FnMut() -> Result<(), ClientError>,
{
    match endpoint::connect().await {
        Ok(connection) => return Ok(connection),
        Err(error) if error.is_unavailable() => {}
        Err(error) => return Err(error.into()),
    }
    spawn_service()?;

    let deadline = Instant::now() + STARTUP_TIMEOUT;
    loop {
        match endpoint::connect().await {
            Ok(connection) => return Ok(connection),
            Err(error) if error.is_unavailable() && Instant::now() < deadline => {
                sleep(CONNECT_RETRY_INTERVAL).await;
            }
            Err(error) if error.is_unavailable() => return Err(ClientError::StartupTimeout),
            Err(error) => return Err(error.into()),
        }
    }
}

#[derive(Debug, Error)]
enum ExchangeError {
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
    #[error("Lens background service did not acknowledge the request within ten seconds")]
    AcknowledgmentTimeout,
}

impl From<ExchangeError> for ClientError {
    fn from(error: ExchangeError) -> Self {
        match error {
            ExchangeError::Protocol(error) => Self::Protocol(error),
            ExchangeError::AcknowledgmentTimeout => Self::AcknowledgmentTimeout,
        }
    }
}

async fn exchange(
    connection: &mut ClientConnection,
    request: &ServiceRequest,
) -> Result<ServiceResponse, ExchangeError> {
    timeout(ACKNOWLEDGMENT_TIMEOUT, async {
        protocol::write_frame(connection, request).await?;
        protocol::read_frame(connection).await
    })
    .await
    .map_err(|_| ExchangeError::AcknowledgmentTimeout)?
    .map_err(ExchangeError::Protocol)
}

fn verify_request_id(expected: RequestId, received: RequestId) -> Result<(), ClientError> {
    if expected == received {
        Ok(())
    } else {
        Err(ClientError::MismatchedResponse)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::OsString,
        fs, io,
        path::{Path, PathBuf},
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc, Mutex, MutexGuard,
        },
    };

    use tokio::task::JoinHandle;

    use super::{
        request_target_view_with, ClientError, ClientOutcome, OpenErrorCode, OpenInvocation,
    };
    use crate::{service::endpoint::EndpointError, TargetScope};

    static TEST_ENVIRONMENT: Mutex<()> = Mutex::new(());
    static FIXTURE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);
    type ServiceTask = JoinHandle<Result<(), EndpointError>>;

    #[tokio::test]
    async fn missing_service_then_command_starts_service_and_returns_after_view_ready() {
        // Arrange
        let mut fixture = TestRuntime::new("missing-service");
        let document_root = fixture.document_root("started", "# Background session");
        let browser_attempts = Arc::new(AtomicUsize::new(0));
        let observed_attempts = browser_attempts.clone();

        // Act
        let outcome = request_target_view_with(
            invocation(&document_root),
            fixture.service_spawner(),
            move |_| {
                observed_attempts.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
        )
        .await
        .expect("a missing service should be started automatically");
        let view_url = outcome_url(&outcome);
        let page = reqwest::get(&view_url)
            .await
            .expect("acknowledged view should respond")
            .text()
            .await
            .expect("acknowledged view should be readable");

        // Assert
        assert!(matches!(outcome, ClientOutcome::Opened { .. }));
        assert_eq!(browser_attempts.load(Ordering::SeqCst), 1);
        assert!(page.contains("Background session"));
        fixture.shutdown().await;
    }

    #[tokio::test]
    async fn concurrent_first_commands_then_one_service_accepts_both_requests() {
        // Arrange
        let mut fixture = TestRuntime::new("concurrent-first");
        let first_root = fixture.document_root("first", "# First command");
        let second_root = fixture.document_root("second", "# Second command");
        let browser_attempts = Arc::new(AtomicUsize::new(0));
        let first_attempts = browser_attempts.clone();
        let second_attempts = browser_attempts.clone();

        // Act
        let (first, second) = tokio::join!(
            request_target_view_with(
                invocation(&first_root),
                fixture.service_spawner(),
                move |_| {
                    first_attempts.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                },
            ),
            request_target_view_with(
                invocation(&second_root),
                fixture.service_spawner(),
                move |_| {
                    second_attempts.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                },
            )
        );
        let first_url = outcome_url(&first.expect("first command should be accepted"));
        let second_url = outcome_url(&second.expect("second command should be accepted"));
        let (first_page, second_page) =
            tokio::join!(response_text(&first_url), response_text(&second_url));

        // Assert
        assert_ne!(first_url, second_url);
        assert!(first_page.contains("First command"));
        assert!(second_page.contains("Second command"));
        assert_eq!(browser_attempts.load(Ordering::SeqCst), 2);
        fixture.shutdown().await;
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn stale_endpoint_then_next_command_recovers_without_manual_cleanup() {
        // Arrange
        let mut fixture = TestRuntime::new("stale-endpoint");
        let document_root = fixture.document_root("recovered", "# Recovered command");
        let endpoint = fixture.directory.join("service-v1.sock");
        let stale = std::os::unix::net::UnixListener::bind(&endpoint)
            .expect("stale endpoint should be creatable");
        drop(stale);

        // Act
        let outcome = request_target_view_with(
            invocation(&document_root),
            fixture.service_spawner(),
            |_| Ok(()),
        )
        .await;

        // Assert
        assert!(matches!(outcome, Ok(ClientOutcome::Opened { .. })));
        fixture.shutdown().await;
    }

    #[tokio::test]
    async fn browser_launch_failure_then_reports_manual_url_and_keeps_session_available() {
        // Arrange
        let mut fixture = TestRuntime::new("browser-failure");
        let document_root = fixture.document_root("manual", "# Manual URL session");

        // Act
        let outcome = request_target_view_with(
            invocation(&document_root),
            fixture.service_spawner(),
            |_| {
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "controlled browser failure",
                ))
            },
        )
        .await
        .expect("browser failure should preserve the ready outcome");
        let view_url = outcome_url(&outcome);
        let page = response_text(&view_url).await;

        // Assert
        assert!(matches!(
            outcome,
            ClientOutcome::ManualUrl { ref error, .. }
                if error.contains("controlled browser failure")
        ));
        assert!(page.contains("Manual URL session"));
        fixture.shutdown().await;
    }

    #[tokio::test]
    async fn target_rejected_then_command_returns_error_without_browser_attempt() {
        // Arrange
        let mut fixture = TestRuntime::new("target-rejection");
        let browser_attempts = Arc::new(AtomicUsize::new(0));
        let observed_attempts = browser_attempts.clone();
        let invocation = OpenInvocation {
            invocation_directory: fixture.directory.clone(),
            target: Some(PathBuf::from("missing.md")),
            scope: TargetScope::Repository,
            plantuml_server: None,
        };

        // Act
        let outcome = request_target_view_with(invocation, fixture.service_spawner(), move |_| {
            observed_attempts.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .await;

        // Assert
        assert!(matches!(
            outcome,
            Err(ClientError::Rejected {
                code: OpenErrorCode::Target,
                ..
            })
        ));
        assert_eq!(browser_attempts.load(Ordering::SeqCst), 0);
        fixture.shutdown().await;
    }

    struct TestRuntime {
        directory: PathBuf,
        previous_runtime_directory: Option<OsString>,
        tasks: Arc<Mutex<Vec<ServiceTask>>>,
        _environment_guard: MutexGuard<'static, ()>,
    }

    impl TestRuntime {
        fn new(name: &str) -> Self {
            let environment_guard = TEST_ENVIRONMENT
                .lock()
                .expect("test environment lock should be available");
            let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::SeqCst);
            let directory = std::env::temp_dir().join(format!(
                "lens-client-{}-{sequence}-{name}",
                std::process::id()
            ));
            if directory.exists() {
                fs::remove_dir_all(&directory).expect("stale fixture should be removable");
            }
            create_private_directory(&directory);
            let previous_runtime_directory = std::env::var_os("XDG_RUNTIME_DIR");
            std::env::set_var("XDG_RUNTIME_DIR", &directory);
            Self {
                directory,
                previous_runtime_directory,
                tasks: Arc::new(Mutex::new(Vec::new())),
                _environment_guard: environment_guard,
            }
        }

        fn document_root(&self, name: &str, source: &str) -> PathBuf {
            let root = self.directory.join(name);
            fs::create_dir(&root).expect("document root should be creatable");
            fs::write(root.join("README.md"), source).expect("test document should be writable");
            root
        }

        fn service_spawner(&self) -> impl FnMut() -> Result<(), ClientError> {
            let tasks = self.tasks.clone();
            move || {
                let task = tokio::spawn(crate::service::server::run_background_service());
                tasks
                    .lock()
                    .expect("service task list should be available")
                    .push(task);
                Ok(())
            }
        }

        async fn shutdown(&mut self) {
            let tasks = self
                .tasks
                .lock()
                .expect("service task list should be available")
                .drain(..)
                .collect::<Vec<_>>();
            for task in &tasks {
                task.abort();
            }
            for task in tasks {
                let _ = task.await;
            }
        }
    }

    impl Drop for TestRuntime {
        fn drop(&mut self) {
            if let Some(previous) = &self.previous_runtime_directory {
                std::env::set_var("XDG_RUNTIME_DIR", previous);
            } else {
                std::env::remove_var("XDG_RUNTIME_DIR");
            }
            if self.directory.exists() {
                fs::remove_dir_all(&self.directory).expect("test fixture should be removable");
            }
        }
    }

    fn invocation(document_root: &Path) -> OpenInvocation {
        OpenInvocation {
            invocation_directory: document_root.to_path_buf(),
            target: None,
            scope: TargetScope::Target,
            plantuml_server: None,
        }
    }

    fn outcome_url(outcome: &ClientOutcome) -> String {
        match outcome {
            ClientOutcome::Opened { view_url } | ClientOutcome::ManualUrl { view_url, .. } => {
                view_url.clone()
            }
        }
    }

    async fn response_text(url: &str) -> String {
        reqwest::get(url)
            .await
            .expect("acknowledged view should respond")
            .text()
            .await
            .expect("acknowledged view should be readable")
    }

    #[cfg(unix)]
    fn create_private_directory(path: &Path) {
        use std::os::unix::fs::DirBuilderExt;

        fs::DirBuilder::new()
            .mode(0o700)
            .create(path)
            .expect("private runtime directory should be creatable");
    }

    #[cfg(windows)]
    fn create_private_directory(path: &Path) {
        fs::create_dir(path).expect("private runtime directory should be creatable");
    }
}
