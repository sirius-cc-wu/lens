use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use thiserror::Error;
use tokio::task::JoinHandle;

use super::{
    controller::{start_controller, ControllerError, ServiceControllerHandle},
    endpoint::{self, EndpointError, ServerConnection},
    protocol::{self, ProtocolError, ProtocolVersion, ServiceRequest, ServiceResponse},
};

#[derive(Debug, Error)]
enum ConnectionError {
    #[error(transparent)]
    Endpoint(#[from] EndpointError),
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
    #[error(transparent)]
    Controller(#[from] ControllerError),
}

#[derive(Debug, Error)]
pub(crate) enum ServerError {
    #[error(transparent)]
    Endpoint(#[from] EndpointError),
    #[error(transparent)]
    Controller(#[from] ControllerError),
}

pub(crate) async fn run_background_service() -> Result<(), ServerError> {
    let mut listener = match endpoint::claim() {
        Ok(listener) => listener,
        Err(EndpointError::AlreadyOwned) => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    let mut controller = start_controller();
    let mut connections = HashMap::new();
    let mut next_connection_id = 0_u64;
    println!("Lens background service is ready");

    loop {
        tokio::select! {
            stop = controller.wait_for_stop_request() => {
                stop?;
                break;
            }
            accepted = listener.accept() => {
                accept_connection(
                    accepted?,
                    &controller,
                    &mut connections,
                    &mut next_connection_id,
                ).await;
            }
        }
    }

    loop {
        tokio::select! {
            cleanup = controller.wait_until_cleanup_complete() => {
                cleanup?;
                break;
            }
            accepted = listener.accept() => {
                accept_connection(
                    accepted?,
                    &controller,
                    &mut connections,
                    &mut next_connection_id,
                ).await;
            }
        }
    }

    let shutdown_barrier = listener.begin_shutdown()?;
    tokio::task::yield_now().await;
    for connection in connections.values() {
        if !connection.request_submitted.load(Ordering::SeqCst) {
            connection.task.abort();
        }
    }
    controller.endpoint_removed().await?;
    controller.wait_until_stopped().await?;

    for (_, connection) in connections {
        let _ = connection.task.await;
    }
    drop(shutdown_barrier);
    Ok(())
}

async fn accept_connection(
    connection: ServerConnection,
    controller: &super::controller::ServiceControllerRuntime,
    connections: &mut HashMap<u64, ConnectionTask>,
    next_connection_id: &mut u64,
) {
    let request_submitted = Arc::new(AtomicBool::new(false));
    let task = spawn_connection(connection, controller.handle(), request_submitted.clone());
    connections.insert(
        *next_connection_id,
        ConnectionTask {
            request_submitted,
            task,
        },
    );
    *next_connection_id = next_connection_id.wrapping_add(1);
    remove_finished_connections(connections).await;
}

fn spawn_connection(
    connection: ServerConnection,
    controller: ServiceControllerHandle,
    request_submitted: Arc<AtomicBool>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        if let Err(error) = handle_connection(connection, controller, request_submitted).await {
            eprintln!("Lens background service rejected a command: {error}");
        }
    })
}

async fn remove_finished_connections(connections: &mut HashMap<u64, ConnectionTask>) {
    let finished = connections
        .iter()
        .filter_map(|(id, connection)| connection.task.is_finished().then_some(*id))
        .collect::<Vec<_>>();
    for id in finished {
        if let Some(connection) = connections.remove(&id) {
            let _ = connection.task.await;
        }
    }
}

struct ConnectionTask {
    request_submitted: Arc<AtomicBool>,
    task: JoinHandle<()>,
}

async fn handle_connection(
    mut connection: ServerConnection,
    controller: ServiceControllerHandle,
    request_submitted: Arc<AtomicBool>,
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

    request_submitted.store(true, Ordering::SeqCst);
    let response = match request {
        ServiceRequest::Open(request) => controller.open(request).await?,
        ServiceRequest::Stop(_) => controller.stop().await?,
    };
    protocol::write_frame(&mut connection, &response).await?;
    Ok(())
}
