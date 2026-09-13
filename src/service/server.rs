use std::time::Duration;

use indexmap::{map::Entry, IndexMap};
use thiserror::Error;
use tokio::{
    sync::{mpsc, oneshot},
    task::{JoinHandle, JoinSet},
};

use super::{
    endpoint::{self, EndpointError, ServerConnection},
    protocol::{
        self, OpenError, OpenErrorCode, OpenRequest, ProtocolError, ProtocolVersion, RequestId,
        ServiceRequest, ServiceResponse, StopRequest,
    },
};
use crate::viewer::ViewerSession;

const CONTROLLER_CAPACITY: usize = 32;
const SHUTDOWN_DRAIN_TIMEOUT: Duration = Duration::from_millis(1000);

#[derive(Debug, Error)]
enum ConnectionError {
    #[error(transparent)]
    Endpoint(#[from] EndpointError),
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
    #[error(transparent)]
    Controller(#[from] ControllerError),
}

pub(crate) async fn run_background_service() -> Result<(), EndpointError> {
    let mut listener = match endpoint::claim() {
        Ok(listener) => listener,
        Err(EndpointError::AlreadyOwned) => return Ok(()),
        Err(error) => return Err(error),
    };
    let mut controller = start_controller();
    let mut shutdown_receiver = controller
        .take_shutdown_receiver()
        .expect("controller should provide shutdown receiver");
    let mut connections = JoinSet::new();
    println!("Lens background service is ready");

    loop {
        tokio::select! {
            connection = listener.accept() => {
                let connection = connection?;
                let handle = controller.handle();
                connections.spawn(async move {
                    if let Err(error) = handle_connection(connection, handle).await {
                        eprintln!("Lens background service rejected a command: {error}");
                    }
                });
            }
            Some(_) = connections.join_next(), if !connections.is_empty() => {}
            _ = &mut shutdown_receiver => {
                break;
            }
        }
    }

    let drain_deadline = tokio::time::Instant::now() + SHUTDOWN_DRAIN_TIMEOUT;
    loop {
        tokio::select! {
            connection = listener.accept() => {
                let connection = connection?;
                let handle = controller.handle();
                connections.spawn(async move {
                    if let Err(error) = handle_connection(connection, handle).await {
                        eprintln!("Lens background service rejected a command: {error}");
                    }
                });
            }
            Some(_) = connections.join_next(), if !connections.is_empty() => {
                if connections.is_empty() {
                    break;
                }
            }
            _ = tokio::time::sleep_until(drain_deadline) => {
                break;
            }
        }
    }

    connections.abort_all();
    while connections.join_next().await.is_some() {}

    Ok(())
}

async fn handle_connection(
    mut connection: ServerConnection,
    controller: ServiceControllerHandle,
) -> Result<(), ConnectionError> {
    endpoint::authorize(&connection)?;
    let request: ServiceRequest = protocol::read_frame(&mut connection).await?;
    if request.validate_version().is_err() {
        protocol::write_frame(
            &mut connection,
            &ServiceResponse::Incompatible {
                supported_version: ProtocolVersion::CURRENT,
            },
        )
        .await?;
        return Ok(());
    }

    let response = match request {
        ServiceRequest::Open(request) => controller.open(request).await?,
        ServiceRequest::Stop(request) => controller.stop(request).await?,
    };
    protocol::write_frame(&mut connection, &response).await?;
    Ok(())
}

#[derive(Clone)]
pub(crate) struct ServiceControllerHandle {
    sender: mpsc::Sender<ControllerMessage>,
}

pub(crate) struct ServiceControllerRuntime {
    handle: ServiceControllerHandle,
    task: JoinHandle<()>,
    shutdown_receiver: Option<oneshot::Receiver<()>>,
}

impl ServiceControllerRuntime {
    pub(crate) fn handle(&self) -> ServiceControllerHandle {
        self.handle.clone()
    }

    pub(crate) fn take_shutdown_receiver(&mut self) -> Option<oneshot::Receiver<()>> {
        self.shutdown_receiver.take()
    }
}

impl Drop for ServiceControllerRuntime {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ControllerStats {
    pub(crate) requests: usize,
    pub(crate) ready_sessions: usize,
}

#[derive(Debug, Error)]
pub(crate) enum ControllerError {
    #[error("the Lens background service controller is unavailable")]
    Unavailable,
}

impl ServiceControllerHandle {
    pub(crate) async fn open(
        &self,
        request: OpenRequest,
    ) -> Result<ServiceResponse, ControllerError> {
        let (reply, response) = oneshot::channel();
        self.sender
            .send(ControllerMessage::Open { request, reply })
            .await
            .map_err(|_| ControllerError::Unavailable)?;
        response.await.map_err(|_| ControllerError::Unavailable)
    }

    pub(crate) async fn stop(
        &self,
        request: StopRequest,
    ) -> Result<ServiceResponse, ControllerError> {
        let (reply, response) = oneshot::channel();
        self.sender
            .send(ControllerMessage::Stop { request, reply })
            .await
            .map_err(|_| ControllerError::Unavailable)?;
        response.await.map_err(|_| ControllerError::Unavailable)
    }

    #[cfg(test)]
    pub(crate) async fn stats(&self) -> Result<ControllerStats, ControllerError> {
        let (reply, response) = oneshot::channel();
        self.sender
            .send(ControllerMessage::Stats { reply })
            .await
            .map_err(|_| ControllerError::Unavailable)?;
        response.await.map_err(|_| ControllerError::Unavailable)
    }
}

pub(crate) fn start_controller() -> ServiceControllerRuntime {
    let (sender, receiver) = mpsc::channel(CONTROLLER_CAPACITY);
    let (shutdown_sender, shutdown_receiver) = oneshot::channel();
    let handle = ServiceControllerHandle {
        sender: sender.clone(),
    };
    let controller = ServiceController {
        state: ControllerState::Running,
        requests: RequestLedger::default(),
        completion_sender: sender,
        shutdown_signal: Some(shutdown_sender),
    };
    let task = tokio::spawn(controller.run(receiver));
    ServiceControllerRuntime {
        handle,
        task,
        shutdown_receiver: Some(shutdown_receiver),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ControllerState {
    Running,
    Stopping,
}

enum ControllerMessage {
    Open {
        request: OpenRequest,
        reply: oneshot::Sender<ServiceResponse>,
    },
    Stop {
        request: StopRequest,
        reply: oneshot::Sender<ServiceResponse>,
    },
    Complete {
        request_id: RequestId,
        completion: SessionCompletion,
    },
    #[cfg(test)]
    Stats {
        reply: oneshot::Sender<ControllerStats>,
    },
}

struct ServiceController {
    state: ControllerState,
    requests: RequestLedger,
    completion_sender: mpsc::Sender<ControllerMessage>,
    shutdown_signal: Option<oneshot::Sender<()>>,
}

impl ServiceController {
    async fn run(mut self, mut receiver: mpsc::Receiver<ControllerMessage>) {
        while let Some(message) = receiver.recv().await {
            match message {
                ControllerMessage::Open { request, reply } => self.open(request, reply),
                ControllerMessage::Stop { request, reply } => self.stop(request, reply).await,
                ControllerMessage::Complete {
                    request_id,
                    completion,
                } => self.complete(request_id, completion).await,
                #[cfg(test)]
                ControllerMessage::Stats { reply } => {
                    let _ = reply.send(self.requests.stats());
                }
            }
        }
    }

    fn open(&mut self, request: OpenRequest, reply: oneshot::Sender<ServiceResponse>) {
        if self.state == ControllerState::Stopping {
            let _ = reply.send(ServiceResponse::Rejected {
                request_id: request.request_id,
                error: OpenError {
                    code: OpenErrorCode::Session,
                    message: "Lens background service is stopping".to_owned(),
                },
            });
            return;
        }

        let request_id = request.request_id;
        match self.requests.entries.entry(request_id) {
            Entry::Occupied(mut entry) => match entry.get_mut() {
                RequestState::InFlight { waiters } => waiters.push(reply),
                RequestState::Complete { response, .. } => {
                    let _ = reply.send(response.clone());
                }
            },
            Entry::Vacant(entry) => {
                entry.insert(RequestState::InFlight {
                    waiters: vec![reply],
                });
                let sender = self.completion_sender.clone();
                tokio::spawn(async move {
                    let completion = create_session(request).await;
                    let _ = sender
                        .send(ControllerMessage::Complete {
                            request_id,
                            completion,
                        })
                        .await;
                });
            }
        }
    }

    async fn stop(&mut self, request: StopRequest, reply: oneshot::Sender<ServiceResponse>) {
        let is_first_stop = self.state == ControllerState::Running;
        self.state = ControllerState::Stopping;

        let _ = reply.send(ServiceResponse::Stopped {
            request_id: request.request_id,
        });

        if is_first_stop {
            let sessions = self.requests.drain_sessions();
            for session in sessions {
                session.stop().await;
            }
            if let Some(shutdown) = self.shutdown_signal.take() {
                let _ = shutdown.send(());
            }
        }
    }

    async fn complete(&mut self, request_id: RequestId, completion: SessionCompletion) {
        if self.state == ControllerState::Stopping {
            if let Some(session) = completion.session {
                session.stop().await;
            }
            return;
        }
        self.requests.complete(request_id, completion);
    }
}

#[derive(Default)]
struct RequestLedger {
    entries: IndexMap<RequestId, RequestState>,
}

impl RequestLedger {
    fn complete(&mut self, request_id: RequestId, completion: SessionCompletion) {
        let Some(RequestState::InFlight { waiters }) = self.entries.shift_remove(&request_id)
        else {
            return;
        };
        for waiter in waiters {
            let _ = waiter.send(completion.response.clone());
        }
        self.entries.insert(
            request_id,
            RequestState::Complete {
                response: completion.response,
                session: completion.session,
            },
        );
    }

    fn drain_sessions(&mut self) -> Vec<ViewerSession> {
        let mut sessions = Vec::new();
        for state in self.entries.values_mut() {
            if let RequestState::Complete { session, .. } = state {
                if let Some(session) = session.take() {
                    sessions.push(session);
                }
            }
        }
        sessions
    }

    #[cfg(test)]
    fn stats(&self) -> ControllerStats {
        ControllerStats {
            requests: self.entries.len(),
            ready_sessions: self
                .entries
                .values()
                .filter(|state| {
                    matches!(
                        state,
                        RequestState::Complete {
                            session: Some(_),
                            ..
                        }
                    )
                })
                .count(),
        }
    }
}

enum RequestState {
    InFlight {
        waiters: Vec<oneshot::Sender<ServiceResponse>>,
    },
    Complete {
        response: ServiceResponse,
        session: Option<ViewerSession>,
    },
}

struct SessionCompletion {
    response: ServiceResponse,
    session: Option<ViewerSession>,
}

async fn create_session(request: OpenRequest) -> SessionCompletion {
    let request_id = request.request_id;
    let invocation_directory = match request.invocation_directory.into_path() {
        Ok(path) => path,
        Err(error) => return rejected(request_id, OpenErrorCode::Target, error.to_string()),
    };
    let target = match request.target.map(|path| path.into_path()).transpose() {
        Ok(path) => path,
        Err(error) => return rejected(request_id, OpenErrorCode::Target, error.to_string()),
    };
    let target = match crate::target::load_markdown_target_from(
        &invocation_directory,
        target.as_deref(),
        request.scope,
    ) {
        Ok(target) => target,
        Err(error) => return rejected(request_id, OpenErrorCode::Target, error.to_string()),
    };
    let plantuml_server = crate::plantuml::server_from_value(request.plantuml_server.as_deref());
    match crate::viewer::start_session(target, plantuml_server).await {
        Ok(session) => SessionCompletion {
            response: ServiceResponse::Ready {
                request_id,
                view_url: session.view_url().to_owned(),
            },
            session: Some(session),
        },
        Err(error) => rejected(request_id, OpenErrorCode::Session, error.to_string()),
    }
}

fn rejected(request_id: RequestId, code: OpenErrorCode, message: String) -> SessionCompletion {
    SessionCompletion {
        response: ServiceResponse::Rejected {
            request_id,
            error: OpenError { code, message },
        },
        session: None,
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, net::TcpListener, path::PathBuf};

    use axum::{http::header, routing::get, Router};

    use super::start_controller;
    use crate::{
        plantuml::PUBLIC_SERVER,
        service::protocol::{
            OpenError, OpenErrorCode, OpenRequest, ProtocolVersion, RequestId, ServiceResponse,
            StopRequest, WirePath,
        },
        TargetScope,
    };

    #[tokio::test]
    async fn stop_message_when_running_then_replies_stopped_and_transitions_state() {
        // Arrange
        let root = document_root("stop-running", "# Stop running");
        let request = open_request(10, &root, PUBLIC_SERVER);
        let runtime = start_controller();
        let handle = runtime.handle();
        let ready = handle.open(request).await.expect("open should succeed");
        assert!(matches!(ready, ServiceResponse::Ready { .. }));
        let stats_before = handle.stats().await.expect("stats should succeed");
        assert_eq!(stats_before.ready_sessions, 1);

        // Act
        let stop_request = StopRequest {
            protocol_version: ProtocolVersion::CURRENT,
            request_id: RequestId::from_bytes([20; 16]),
        };
        let response = handle
            .stop(stop_request)
            .await
            .expect("stop should succeed");

        // Assert
        assert_eq!(
            response,
            ServiceResponse::Stopped {
                request_id: RequestId::from_bytes([20; 16]),
            }
        );
        let stats_after = handle.stats().await.expect("stats should succeed");
        assert_eq!(stats_after.ready_sessions, 0);
        drop(runtime);
        fs::remove_dir_all(root).expect("test fixture should be removable");
    }

    #[tokio::test]
    async fn open_message_when_stopping_then_rejects_with_session_error() {
        // Arrange
        let root = document_root("open-stopping", "# Open while stopping");
        let runtime = start_controller();
        let handle = runtime.handle();
        let stop_request = StopRequest {
            protocol_version: ProtocolVersion::CURRENT,
            request_id: RequestId::from_bytes([21; 16]),
        };
        handle
            .stop(stop_request)
            .await
            .expect("stop should succeed");

        // Act
        let open_req = open_request(22, &root, PUBLIC_SERVER);
        let response = handle.open(open_req).await.expect("open should complete");

        // Assert
        assert_eq!(
            response,
            ServiceResponse::Rejected {
                request_id: RequestId::from_bytes([22; 16]),
                error: OpenError {
                    code: OpenErrorCode::Session,
                    message: "Lens background service is stopping".to_owned(),
                },
            }
        );
        drop(runtime);
        fs::remove_dir_all(root).expect("test fixture should be removable");
    }

    #[tokio::test]
    async fn repeated_stop_message_when_stopping_then_replies_stopped_idempotently() {
        // Arrange
        let runtime = start_controller();
        let handle = runtime.handle();
        let first_stop = StopRequest {
            protocol_version: ProtocolVersion::CURRENT,
            request_id: RequestId::from_bytes([30; 16]),
        };
        let first_response = handle
            .stop(first_stop)
            .await
            .expect("first stop should succeed");
        assert_eq!(
            first_response,
            ServiceResponse::Stopped {
                request_id: RequestId::from_bytes([30; 16]),
            }
        );

        // Act
        let second_stop = StopRequest {
            protocol_version: ProtocolVersion::CURRENT,
            request_id: RequestId::from_bytes([31; 16]),
        };
        let second_response = handle
            .stop(second_stop)
            .await
            .expect("second stop should succeed");

        // Assert
        assert_eq!(
            second_response,
            ServiceResponse::Stopped {
                request_id: RequestId::from_bytes([31; 16]),
            }
        );
        drop(runtime);
    }

    #[tokio::test]
    async fn same_request_retried_then_one_viewing_session_is_retained() {
        // Arrange
        let root = document_root("same-request", "# Idempotent session");
        let request = open_request(1, &root, PUBLIC_SERVER);
        let runtime = start_controller();
        let first_handle = runtime.handle();
        let second_handle = runtime.handle();

        // Act
        let (first, second) = tokio::join!(
            first_handle.open(request.clone()),
            second_handle.open(request)
        );

        // Assert
        let first = first.expect("first request should complete");
        let second = second.expect("retried request should complete");
        assert_eq!(first, second);
        assert!(matches!(first, ServiceResponse::Ready { .. }));
        let stats = runtime
            .handle()
            .stats()
            .await
            .expect("controller statistics should be available");
        assert_eq!(stats.requests, 1);
        assert_eq!(stats.ready_sessions, 1);
        drop(runtime);
        fs::remove_dir_all(root).expect("test fixture should be removable");
    }

    #[tokio::test]
    async fn different_requests_then_isolated_sessions_keep_their_roots_and_servers() {
        // Arrange
        let first_root = document_root(
            "first-isolated-request",
            "# First isolated root\n\n```plantuml\n@startuml\n@enduml\n```",
        );
        let second_root = document_root(
            "second-isolated-request",
            "# Second isolated root\n\n```plantuml\n@startuml\n@enduml\n```",
        );
        let first_server = mock_plantuml_server("first renderer").await;
        let second_server = mock_plantuml_server("second renderer").await;
        let runtime = start_controller();
        let first_handle = runtime.handle();
        let second_handle = runtime.handle();

        // Act
        let (first, second) = tokio::join!(
            first_handle.open(open_request(2, &first_root, &first_server)),
            second_handle.open(open_request(3, &second_root, &second_server))
        );
        let first_url = ready_url(first.expect("first request should complete"));
        let second_url = ready_url(second.expect("second request should complete"));
        let (first_origin, first_query) = first_url
            .split_once("/?")
            .expect("first ready url should contain query token");
        let (second_origin, second_query) = second_url
            .split_once("/?")
            .expect("second ready url should contain query token");
        let first_page = reqwest::get(&first_url)
            .await
            .expect("first viewer should respond")
            .text()
            .await
            .expect("first page should be readable");
        let second_page = reqwest::get(&second_url)
            .await
            .expect("second viewer should respond")
            .text()
            .await
            .expect("second page should be readable");
        let first_diagram = reqwest::get(format!("{first_origin}/diagrams/0/0?{first_query}"))
            .await
            .expect("first diagram should respond")
            .text()
            .await
            .expect("first diagram should be readable");
        let second_diagram = reqwest::get(format!("{second_origin}/diagrams/0/0?{second_query}"))
            .await
            .expect("second diagram should respond")
            .text()
            .await
            .expect("second diagram should be readable");
        let unauthenticated_diagram = reqwest::get(format!("{first_origin}/diagrams/0/0"))
            .await
            .expect("unauthenticated request should reach viewer");
        let cross_session_diagram =
            reqwest::get(format!("{first_origin}/diagrams/0/0?{second_query}"))
                .await
                .expect("cross-session request should reach viewer");

        // Assert
        assert_ne!(first_url, second_url);
        assert!(first_page.contains("First isolated root"));
        assert!(!first_page.contains("Second isolated root"));
        assert!(second_page.contains("Second isolated root"));
        assert!(!second_page.contains("First isolated root"));
        assert!(first_diagram.contains("first renderer"));
        assert!(second_diagram.contains("second renderer"));
        assert_eq!(
            unauthenticated_diagram.status(),
            reqwest::StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            cross_session_diagram.status(),
            reqwest::StatusCode::UNAUTHORIZED
        );
        let stats = runtime
            .handle()
            .stats()
            .await
            .expect("controller statistics should be available");
        assert_eq!(stats.ready_sessions, 2);
        drop(runtime);
        fs::remove_dir_all(first_root).expect("first fixture should be removable");
        fs::remove_dir_all(second_root).expect("second fixture should be removable");
    }

    #[tokio::test]
    async fn target_rejected_then_no_viewing_session_becomes_reachable() {
        // Arrange
        let missing =
            std::env::temp_dir().join(format!("lens-controller-{}-missing", std::process::id()));
        if missing.exists() {
            fs::remove_dir_all(&missing).expect("stale test fixture should be removable");
        }
        let runtime = start_controller();

        // Act
        let response = runtime
            .handle()
            .open(open_request(4, &missing, PUBLIC_SERVER))
            .await
            .expect("rejected request should receive an outcome");
        let stats = runtime
            .handle()
            .stats()
            .await
            .expect("controller statistics should be available");

        // Assert
        assert!(matches!(
            response,
            ServiceResponse::Rejected {
                error: crate::service::protocol::OpenError {
                    code: crate::service::protocol::OpenErrorCode::Target,
                    ..
                },
                ..
            }
        ));
        assert_eq!(stats.requests, 1);
        assert_eq!(stats.ready_sessions, 0);
    }

    fn open_request(marker: u8, root: &std::path::Path, plantuml_server: &str) -> OpenRequest {
        OpenRequest {
            protocol_version: ProtocolVersion::CURRENT,
            request_id: RequestId::from_bytes([marker; 16]),
            invocation_directory: WirePath::from_path(root),
            target: None,
            scope: TargetScope::Target,
            plantuml_server: Some(plantuml_server.to_owned()),
        }
    }

    fn document_root(name: &str, source: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("lens-controller-{}-{name}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).expect("stale test fixture should be removable");
        }
        fs::create_dir(&root).expect("test document root should be creatable");
        fs::write(root.join("README.md"), source).expect("test document should be writable");
        root
    }

    fn ready_url(response: ServiceResponse) -> String {
        match response {
            ServiceResponse::Ready { view_url, .. } => view_url,
            other => panic!("expected ready response, got {other:?}"),
        }
    }

    async fn mock_plantuml_server(label: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("mock server should bind");
        let address = listener
            .local_addr()
            .expect("mock server should have an address");
        let server = Router::new().route(
            "/svg/*encoded",
            get(move || async move {
                (
                    [(header::CONTENT_TYPE, "image/svg+xml")],
                    format!("<svg><text>{label}</text></svg>"),
                )
            }),
        );
        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .expect("mock server should serve")
                .serve(server.into_make_service())
                .await
                .expect("mock server should not fail");
        });
        format!("http://{address}")
    }
}
