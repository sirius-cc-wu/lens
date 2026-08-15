use std::{future::Future, time::Duration};

use indexmap::{map::Entry, IndexMap};
use thiserror::Error;
use tokio::{
    sync::{mpsc, oneshot, watch},
    task::{JoinHandle, JoinSet},
};

use super::protocol::{OpenError, OpenErrorCode, OpenRequest, RequestId, ServiceResponse};
use crate::viewer::ViewerSession;

const CONTROLLER_CAPACITY: usize = 32;
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub(crate) struct ServiceControllerHandle {
    sender: mpsc::Sender<ControllerMessage>,
}

pub(crate) struct ServiceControllerRuntime {
    handle: ServiceControllerHandle,
    stop_requested: oneshot::Receiver<()>,
    cleanup_complete: watch::Receiver<bool>,
    stopped: watch::Receiver<bool>,
    task: JoinHandle<()>,
}

impl ServiceControllerRuntime {
    pub(crate) fn handle(&self) -> ServiceControllerHandle {
        self.handle.clone()
    }

    pub(crate) async fn wait_for_stop_request(&mut self) -> Result<(), ControllerError> {
        (&mut self.stop_requested)
            .await
            .map_err(|_| ControllerError::Unavailable)
    }

    pub(crate) async fn wait_until_cleanup_complete(&mut self) -> Result<(), ControllerError> {
        while !*self.cleanup_complete.borrow() {
            self.cleanup_complete
                .changed()
                .await
                .map_err(|_| ControllerError::Unavailable)?;
        }
        Ok(())
    }

    pub(crate) async fn endpoint_removed(&self) -> Result<(), ControllerError> {
        self.handle.endpoint_removed().await
    }

    pub(crate) async fn wait_until_stopped(&mut self) -> Result<(), ControllerError> {
        while !*self.stopped.borrow() {
            self.stopped
                .changed()
                .await
                .map_err(|_| ControllerError::Unavailable)?;
        }
        Ok(())
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

    pub(crate) async fn stop(&self) -> Result<ServiceResponse, ControllerError> {
        let (reply, response) = oneshot::channel();
        self.sender
            .send(ControllerMessage::Stop { reply })
            .await
            .map_err(|_| ControllerError::Unavailable)?;
        response.await.map_err(|_| ControllerError::Unavailable)
    }

    async fn endpoint_removed(&self) -> Result<(), ControllerError> {
        let (reply, response) = oneshot::channel();
        self.sender
            .send(ControllerMessage::EndpointRemoved { reply })
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
    start_controller_with(create_session)
}

fn start_controller_with<F, Fut>(create_session: F) -> ServiceControllerRuntime
where
    F: Fn(OpenRequest) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = SessionCompletion> + Send + 'static,
{
    let (sender, receiver) = mpsc::channel(CONTROLLER_CAPACITY);
    let (stop_sender, stop_requested) = oneshot::channel();
    let (cleanup_complete_sender, cleanup_complete) = watch::channel(false);
    let (stopped_sender, stopped) = watch::channel(false);
    let handle = ServiceControllerHandle {
        sender: sender.clone(),
    };
    let controller = ServiceController {
        requests: RequestLedger::default(),
        sender,
        create_session,
        creations: JoinSet::new(),
        late_sessions: Vec::new(),
        phase: ControllerPhase::Running,
        stop_sender: Some(stop_sender),
        cleanup_complete_sender,
        stopped_sender,
        deadline_task: None,
        cleanup_force_sender: None,
        cleanup_tasks: JoinSet::new(),
    };
    let task = tokio::spawn(controller.run(receiver));
    ServiceControllerRuntime {
        handle,
        stop_requested,
        cleanup_complete,
        stopped,
        task,
    }
}

enum ControllerMessage {
    Open {
        request: OpenRequest,
        reply: oneshot::Sender<ServiceResponse>,
    },
    Stop {
        reply: oneshot::Sender<ServiceResponse>,
    },
    EndpointRemoved {
        reply: oneshot::Sender<()>,
    },
    ForceShutdown,
    #[cfg(test)]
    Stats {
        reply: oneshot::Sender<ControllerStats>,
    },
}

enum ControllerPhase {
    Running,
    Stopping {
        stop_waiters: Vec<oneshot::Sender<ServiceResponse>>,
        endpoint_removed: bool,
        cleanup_started: bool,
        cleanup_complete: bool,
        force_requested: bool,
    },
    Stopped,
}

struct ServiceController<F> {
    requests: RequestLedger,
    sender: mpsc::Sender<ControllerMessage>,
    create_session: F,
    creations: JoinSet<(RequestId, SessionCompletion)>,
    late_sessions: Vec<ViewerSession>,
    phase: ControllerPhase,
    stop_sender: Option<oneshot::Sender<()>>,
    cleanup_complete_sender: watch::Sender<bool>,
    stopped_sender: watch::Sender<bool>,
    deadline_task: Option<AbortOnDrop>,
    cleanup_force_sender: Option<watch::Sender<bool>>,
    cleanup_tasks: JoinSet<()>,
}

impl<F, Fut> ServiceController<F>
where
    F: Fn(OpenRequest) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = SessionCompletion> + Send + 'static,
{
    async fn run(mut self, mut receiver: mpsc::Receiver<ControllerMessage>) {
        loop {
            tokio::select! {
                message = receiver.recv() => {
                    let Some(message) = message else {
                        break;
                    };
                    self.handle_message(message);
                }
                completion = self.creations.join_next(), if !self.creations.is_empty() => {
                    if let Some(completion) = completion {
                        self.handle_creation_completion(completion);
                    }
                }
                completion = self.cleanup_tasks.join_next(), if !self.cleanup_tasks.is_empty() => {
                    if let Some(Err(error)) = completion {
                        eprintln!("Lens shutdown task failed: {error}");
                    }
                }
            }
            self.maybe_start_cleanup();
            self.maybe_complete_cleanup();
            self.maybe_finish_shutdown();
        }
    }

    fn handle_message(&mut self, message: ControllerMessage) {
        match message {
            ControllerMessage::Open { request, reply } => self.open(request, reply),
            ControllerMessage::Stop { reply } => self.stop(reply),
            ControllerMessage::EndpointRemoved { reply } => {
                if let ControllerPhase::Stopping {
                    endpoint_removed, ..
                } = &mut self.phase
                {
                    *endpoint_removed = true;
                }
                let _ = reply.send(());
            }
            ControllerMessage::ForceShutdown => self.force_shutdown(),
            #[cfg(test)]
            ControllerMessage::Stats { reply } => {
                let _ = reply.send(self.requests.stats());
            }
        }
    }

    fn open(&mut self, request: OpenRequest, reply: oneshot::Sender<ServiceResponse>) {
        if !matches!(self.phase, ControllerPhase::Running) {
            let _ = reply.send(service_stopping(request.request_id));
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
                let create_session = self.create_session.clone();
                self.creations.spawn(async move {
                    let completion = create_session(request).await;
                    (request_id, completion)
                });
            }
        }
    }

    fn stop(&mut self, reply: oneshot::Sender<ServiceResponse>) {
        match &mut self.phase {
            ControllerPhase::Running => {
                self.requests.reject_inflight();
                self.creations.abort_all();
                self.phase = ControllerPhase::Stopping {
                    stop_waiters: vec![reply],
                    endpoint_removed: false,
                    cleanup_started: false,
                    cleanup_complete: false,
                    force_requested: false,
                };
                let deadline_sender = self.sender.clone();
                self.deadline_task = Some(AbortOnDrop(tokio::spawn(async move {
                    tokio::time::sleep(SHUTDOWN_TIMEOUT).await;
                    let _ = deadline_sender.send(ControllerMessage::ForceShutdown).await;
                })));
                if let Some(stop_sender) = self.stop_sender.take() {
                    let _ = stop_sender.send(());
                }
            }
            ControllerPhase::Stopping { stop_waiters, .. } => stop_waiters.push(reply),
            ControllerPhase::Stopped => {
                let _ = reply.send(ServiceResponse::Stopped);
            }
        }
    }

    fn handle_creation_completion(
        &mut self,
        completion: Result<(RequestId, SessionCompletion), tokio::task::JoinError>,
    ) {
        let Ok((request_id, completion)) = completion else {
            return;
        };
        if matches!(self.phase, ControllerPhase::Running) {
            self.requests.complete(request_id, completion);
        } else if let Some(session) = completion.session {
            self.late_sessions.push(session);
        }
    }

    fn maybe_start_cleanup(&mut self) {
        let ready = matches!(
            self.phase,
            ControllerPhase::Stopping {
                cleanup_started: false,
                ..
            }
        ) && self.creations.is_empty();
        if !ready {
            return;
        }

        let force_requested = match &mut self.phase {
            ControllerPhase::Stopping {
                cleanup_started,
                force_requested,
                ..
            } => {
                *cleanup_started = true;
                *force_requested
            }
            _ => return,
        };
        let (force_sender, force_receiver) = watch::channel(force_requested);
        self.cleanup_force_sender = Some(force_sender);
        let mut sessions = self.requests.take_sessions();
        sessions.append(&mut self.late_sessions);
        for session in sessions {
            let force_receiver = force_receiver.clone();
            self.cleanup_tasks.spawn(async move {
                if let Err(error) = session.shutdown_with_force(force_receiver).await {
                    eprintln!("Lens viewing session did not shut down gracefully: {error}");
                }
            });
        }
    }

    fn maybe_complete_cleanup(&mut self) {
        let complete = matches!(
            self.phase,
            ControllerPhase::Stopping {
                cleanup_started: true,
                cleanup_complete: false,
                ..
            }
        ) && self.creations.is_empty()
            && self.cleanup_tasks.is_empty();
        if !complete {
            return;
        }
        if let ControllerPhase::Stopping {
            cleanup_complete, ..
        } = &mut self.phase
        {
            *cleanup_complete = true;
        }
        self.cleanup_force_sender.take();
        let _ = self.cleanup_complete_sender.send(true);
    }

    fn force_shutdown(&mut self) {
        let ControllerPhase::Stopping {
            force_requested, ..
        } = &mut self.phase
        else {
            return;
        };
        *force_requested = true;
        self.creations.abort_all();
        if let Some(force_sender) = &self.cleanup_force_sender {
            let _ = force_sender.send(true);
        }
    }

    fn maybe_finish_shutdown(&mut self) {
        let finished = matches!(
            self.phase,
            ControllerPhase::Stopping {
                endpoint_removed: true,
                cleanup_complete: true,
                ..
            }
        );
        if !finished {
            return;
        }

        let ControllerPhase::Stopping { stop_waiters, .. } =
            std::mem::replace(&mut self.phase, ControllerPhase::Stopped)
        else {
            unreachable!("finished shutdown should be stopping");
        };
        self.deadline_task.take();
        for waiter in stop_waiters {
            let _ = waiter.send(ServiceResponse::Stopped);
        }
        let _ = self.stopped_sender.send(true);
    }
}

struct AbortOnDrop(JoinHandle<()>);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
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

    fn reject_inflight(&mut self) {
        for (request_id, state) in &mut self.entries {
            if !matches!(state, RequestState::InFlight { .. }) {
                continue;
            }
            let response = service_stopping(*request_id);
            let RequestState::InFlight { waiters } = std::mem::replace(
                state,
                RequestState::Complete {
                    response: response.clone(),
                    session: None,
                },
            ) else {
                unreachable!("in-flight request should remain in flight until replaced");
            };
            for waiter in waiters {
                let _ = waiter.send(response.clone());
            }
        }
    }

    fn take_sessions(&mut self) -> Vec<ViewerSession> {
        std::mem::take(&mut self.entries)
            .into_values()
            .filter_map(|state| match state {
                RequestState::Complete { session, .. } => session,
                RequestState::InFlight { .. } => None,
            })
            .collect()
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

fn service_stopping(request_id: RequestId) -> ServiceResponse {
    ServiceResponse::Rejected {
        request_id,
        error: OpenError {
            code: OpenErrorCode::ServiceStopping,
            message: "Lens background service is stopping".to_owned(),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        future::{pending, Future},
        net::{TcpListener, TcpStream},
        path::PathBuf,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc, Mutex,
        },
    };

    use axum::{http::header, routing::get, Router};
    use tokio::{sync::Notify, time::Duration};

    use super::{start_controller, start_controller_with, SessionCompletion, SHUTDOWN_TIMEOUT};
    use crate::{
        plantuml::PUBLIC_SERVER,
        service::protocol::{
            OpenErrorCode, OpenRequest, ProtocolVersion, RequestId, ServiceResponse, WirePath,
        },
        TargetScope,
    };

    #[tokio::test]
    async fn same_request_retried_then_one_viewing_session_is_retained() {
        // Arrange
        let root = document_root("same-request", "# Idempotent session");
        let request = open_request(1, &root);
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
            first_handle.open(open_request_with_server(2, &first_root, &first_server)),
            second_handle.open(open_request_with_server(3, &second_root, &second_server))
        );
        let first_url = ready_url(first.expect("first request should complete"));
        let second_url = ready_url(second.expect("second request should complete"));
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
        let first_diagram = reqwest::get(format!("{first_url}/diagrams/0/0"))
            .await
            .expect("first diagram should respond")
            .text()
            .await
            .expect("first diagram should be readable");
        let second_diagram = reqwest::get(format!("{second_url}/diagrams/0/0"))
            .await
            .expect("second diagram should respond")
            .text()
            .await
            .expect("second diagram should be readable");

        // Assert
        assert_ne!(first_url, second_url);
        assert!(first_page.contains("First isolated root"));
        assert!(!first_page.contains("Second isolated root"));
        assert!(second_page.contains("Second isolated root"));
        assert!(!second_page.contains("First isolated root"));
        assert!(first_diagram.contains("first renderer"));
        assert!(second_diagram.contains("second renderer"));
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
            .open(open_request(4, &missing))
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
                    code: OpenErrorCode::Target,
                    ..
                },
                ..
            }
        ));
        assert_eq!(stats.requests, 1);
        assert_eq!(stats.ready_sessions, 0);
    }

    #[tokio::test]
    async fn completed_open_before_stop_then_open_result_precedes_successful_stop() {
        // Arrange
        let mut runtime = start_controller_with(immediate_ready);
        let handle = runtime.handle();
        let open_response = handle
            .open(open_request(2, std::path::Path::new(".")))
            .await
            .expect("open should complete before stop");

        // Act
        let stop_task = tokio::spawn({
            let handle = handle.clone();
            async move { handle.stop().await }
        });
        runtime
            .wait_for_stop_request()
            .await
            .expect("stop should be accepted");
        assert!(
            !stop_task.is_finished(),
            "stop must not be acknowledged while the endpoint is owned"
        );
        runtime
            .endpoint_removed()
            .await
            .expect("endpoint removal should be recorded");
        let stop_response = stop_task
            .await
            .expect("stop task should join")
            .expect("stop should complete");

        // Assert
        assert!(matches!(open_response, ServiceResponse::Ready { .. }));
        assert_eq!(stop_response, ServiceResponse::Stopped);
    }

    #[tokio::test]
    async fn unfinished_open_at_stop_then_open_is_rejected_as_service_stopping() {
        // Arrange
        let started = Arc::new(Notify::new());
        let cancelled_creations = Arc::new(AtomicUsize::new(0));
        let factory_started = started.clone();
        let factory_cancelled = cancelled_creations.clone();
        let mut runtime = start_controller_with(move |_request| {
            let factory_started = factory_started.clone();
            let cancellation_guard = CancellationGuard(factory_cancelled.clone());
            async move {
                let _cancellation_guard = cancellation_guard;
                factory_started.notify_one();
                pending::<SessionCompletion>().await
            }
        });
        let handle = runtime.handle();
        let open_task = tokio::spawn({
            let handle = handle.clone();
            async move {
                handle
                    .open(open_request(3, std::path::Path::new(".")))
                    .await
            }
        });
        started.notified().await;

        // Act
        let stop_task = tokio::spawn({
            let handle = handle.clone();
            async move { handle.stop().await }
        });
        runtime
            .wait_for_stop_request()
            .await
            .expect("stop should be accepted");
        let open_response = open_task
            .await
            .expect("open task should join")
            .expect("unfinished open should receive a response");
        runtime
            .wait_until_cleanup_complete()
            .await
            .expect("creation cancellation should be joined");
        assert_eq!(cancelled_creations.load(Ordering::SeqCst), 1);
        runtime
            .endpoint_removed()
            .await
            .expect("endpoint removal should be recorded");
        let stop_response = stop_task
            .await
            .expect("stop task should join")
            .expect("stop should complete");

        // Assert
        assert_service_stopping(open_response);
        assert_eq!(stop_response, ServiceResponse::Stopped);
    }

    #[tokio::test]
    async fn open_after_stop_acceptance_then_open_is_rejected_as_service_stopping() {
        // Arrange
        let mut runtime = start_controller_with(immediate_ready);
        let handle = runtime.handle();
        let stop_task = tokio::spawn({
            let handle = handle.clone();
            async move { handle.stop().await }
        });
        runtime
            .wait_for_stop_request()
            .await
            .expect("stop should be accepted");

        // Act
        let open_response = handle
            .open(open_request(4, std::path::Path::new(".")))
            .await
            .expect("open should receive a stopping response");
        runtime
            .endpoint_removed()
            .await
            .expect("endpoint removal should be recorded");
        stop_task
            .await
            .expect("stop task should join")
            .expect("stop should complete");

        // Assert
        assert_service_stopping(open_response);
    }

    #[tokio::test]
    async fn concurrent_and_repeated_stops_then_each_receives_stopped() {
        // Arrange
        let mut runtime = start_controller_with(immediate_ready);
        let first_handle = runtime.handle();
        let second_handle = runtime.handle();

        // Act
        let first = tokio::spawn(async move { first_handle.stop().await });
        let second = tokio::spawn(async move { second_handle.stop().await });
        runtime
            .wait_for_stop_request()
            .await
            .expect("one stop should begin shutdown");
        runtime
            .endpoint_removed()
            .await
            .expect("endpoint removal should be recorded");
        let first = first
            .await
            .expect("first stop task should join")
            .expect("first stop should complete");
        let second = second
            .await
            .expect("second stop task should join")
            .expect("second stop should complete");
        let repeated = runtime
            .handle()
            .stop()
            .await
            .expect("repeated stop should complete");

        // Assert
        assert_eq!(first, ServiceResponse::Stopped);
        assert_eq!(second, ServiceResponse::Stopped);
        assert_eq!(repeated, ServiceResponse::Stopped);
    }

    #[tokio::test(start_paused = true)]
    async fn cleanup_reaches_five_seconds_then_tasks_and_listener_are_gone_before_stopped() {
        // Arrange
        let cancelled_tasks = Arc::new(AtomicUsize::new(0));
        let (session, address) =
            crate::viewer::ViewerSession::slow_test_session(cancelled_tasks.clone());
        tokio::task::yield_now().await;
        let completion = Arc::new(Mutex::new(Some(SessionCompletion {
            response: ServiceResponse::Ready {
                request_id: RequestId::from_bytes([9; 16]),
                view_url: format!("http://{address}"),
            },
            session: Some(session),
        })));
        let factory_completion = completion.clone();
        let mut runtime = start_controller_with(move |_request| {
            let factory_completion = factory_completion.clone();
            async move {
                factory_completion
                    .lock()
                    .expect("test completion should be available")
                    .take()
                    .expect("test factory should run once")
            }
        });
        let handle = runtime.handle();
        handle
            .open(open_request(9, std::path::Path::new(".")))
            .await
            .expect("slow session should become ready");
        let stop_task = tokio::spawn(async move { handle.stop().await });
        runtime
            .wait_for_stop_request()
            .await
            .expect("stop should be accepted");

        // Act
        tokio::time::advance(SHUTDOWN_TIMEOUT - Duration::from_millis(1)).await;
        assert_eq!(cancelled_tasks.load(Ordering::SeqCst), 0);
        assert!(!stop_task.is_finished());
        tokio::time::advance(Duration::from_millis(1)).await;
        runtime
            .wait_until_cleanup_complete()
            .await
            .expect("forced cleanup should finish after joining tasks");

        // Assert
        assert_eq!(cancelled_tasks.load(Ordering::SeqCst), 2);
        assert!(TcpStream::connect(address).is_err());
        assert!(
            !stop_task.is_finished(),
            "Stopped must wait until endpoint removal after forced cleanup"
        );
        runtime
            .endpoint_removed()
            .await
            .expect("endpoint removal should be recorded");
        assert_eq!(
            stop_task
                .await
                .expect("stop task should join")
                .expect("stop should complete"),
            ServiceResponse::Stopped
        );
    }

    fn immediate_ready(request: OpenRequest) -> impl Future<Output = SessionCompletion> {
        let request_id = request.request_id;
        async move {
            SessionCompletion {
                response: ServiceResponse::Ready {
                    request_id,
                    view_url: "http://127.0.0.1:1".to_owned(),
                },
                session: None,
            }
        }
    }

    fn assert_service_stopping(response: ServiceResponse) {
        assert!(matches!(
            response,
            ServiceResponse::Rejected {
                error: crate::service::protocol::OpenError {
                    code: OpenErrorCode::ServiceStopping,
                    ..
                },
                ..
            }
        ));
    }

    fn open_request(marker: u8, root: &std::path::Path) -> OpenRequest {
        open_request_with_server(marker, root, PUBLIC_SERVER)
    }

    fn open_request_with_server(
        marker: u8,
        root: &std::path::Path,
        plantuml_server: &str,
    ) -> OpenRequest {
        OpenRequest {
            protocol_version: ProtocolVersion::CURRENT,
            request_id: RequestId::from_bytes([marker; 16]),
            invocation_directory: WirePath::from_path(root),
            target: None,
            scope: TargetScope::Target,
            plantuml_server: Some(plantuml_server.to_owned()),
        }
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

    struct CancellationGuard(Arc<AtomicUsize>);

    impl Drop for CancellationGuard {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
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
}
