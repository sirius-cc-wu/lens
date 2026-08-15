use std::{
    env, fs,
    os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
};

use tokio::net::{UnixListener, UnixStream};

use super::EndpointError;

const SOCKET_NAME: &str = "service-v1.sock";
const STOPPING_SUFFIX: &str = ".stopping";

pub(crate) type ClientConnection = UnixStream;
pub(crate) type ServerConnection = UnixStream;

pub(crate) struct Listener {
    inner: UnixListener,
    path: PathBuf,
    device: u64,
    inode: u64,
}

impl Listener {
    pub(crate) async fn accept(&mut self) -> Result<ServerConnection, EndpointError> {
        self.inner
            .accept()
            .await
            .map(|(connection, _)| connection)
            .map_err(|source| EndpointError::io("Could not accept a Lens command", source))
    }

    pub(crate) fn begin_shutdown(mut self) -> Result<ShutdownBarrier, EndpointError> {
        let stopping_path = stopping_path(&self.path);
        remove_verified_stale_socket(&stopping_path)?;
        verify_owned_socket(&self.path, self.device, self.inode)?;
        fs::rename(&self.path, &stopping_path).map_err(|source| {
            EndpointError::io("Could not hide the stopping Lens command endpoint", source)
        })?;
        self.path = stopping_path;
        Ok(ShutdownBarrier { _listener: self })
    }
}

pub(crate) struct ShutdownBarrier {
    _listener: Listener,
}

impl Drop for Listener {
    fn drop(&mut self) {
        let Ok(metadata) = fs::symlink_metadata(&self.path) else {
            return;
        };
        if metadata.file_type().is_socket()
            && metadata.uid() == effective_user_id()
            && metadata.dev() == self.device
            && metadata.ino() == self.inode
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}

pub(crate) async fn connect() -> Result<ClientConnection, EndpointError> {
    let path = endpoint_path()?;
    match connect_verified_at(&path).await {
        Ok(connection) => Ok(connection),
        Err(error) if error.is_unavailable() => {
            inspect_stopping_endpoint(&stopping_path(&path)).await?;
            Err(error)
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn claim() -> Result<Listener, EndpointError> {
    claim_at(&endpoint_path()?)
}

pub(crate) fn authorize(connection: &ServerConnection) -> Result<(), EndpointError> {
    let peer = connection
        .peer_cred()
        .map_err(|source| EndpointError::io("Could not identify the Lens command peer", source))?
        .uid();
    authorize_user(peer, effective_user_id())
}

fn endpoint_path() -> Result<PathBuf, EndpointError> {
    Ok(runtime_directory()?.join(SOCKET_NAME))
}

fn runtime_directory() -> Result<PathBuf, EndpointError> {
    let directory = env::var_os("XDG_RUNTIME_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join(format!("lens-runtime-{}", effective_user_id())));
    prepare_runtime_directory(&directory)?;
    Ok(directory)
}

fn prepare_runtime_directory(path: &Path) -> Result<(), EndpointError> {
    match fs::symlink_metadata(path) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::DirBuilder::new()
                .mode(0o700)
                .create(path)
                .map_err(|source| {
                    EndpointError::io(
                        "Could not create the private Lens runtime directory",
                        source,
                    )
                })?;
        }
        Err(source) => {
            return Err(EndpointError::io(
                "Could not inspect the Lens runtime directory",
                source,
            ));
        }
    }

    let metadata = fs::symlink_metadata(path).map_err(|source| {
        EndpointError::io("Could not inspect the Lens runtime directory", source)
    })?;
    let owner = effective_user_id();
    if !metadata.file_type().is_dir() {
        return Err(EndpointError::UnsafeRuntimeDirectory {
            path: path.to_path_buf(),
            reason: "the path is not a directory".to_owned(),
        });
    }
    if metadata.uid() != owner {
        return Err(EndpointError::UnsafeRuntimeDirectory {
            path: path.to_path_buf(),
            reason: format!(
                "owner user {} does not match current user {owner}",
                metadata.uid()
            ),
        });
    }
    if metadata.mode() & 0o077 != 0 {
        return Err(EndpointError::UnsafeRuntimeDirectory {
            path: path.to_path_buf(),
            reason: format!(
                "permissions {:o} allow group or other access",
                metadata.mode() & 0o777
            ),
        });
    }
    Ok(())
}

async fn connect_at(path: &Path) -> Result<ClientConnection, EndpointError> {
    UnixStream::connect(path).await.map_err(|source| {
        EndpointError::io("Could not connect to the Lens background service", source)
    })
}

async fn connect_verified_at(path: &Path) -> Result<ClientConnection, EndpointError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| {
        EndpointError::io("Could not inspect the Lens command endpoint", source)
    })?;
    validate_owned_socket(path, &metadata)?;
    let connection = connect_at(path).await?;
    if let Err(error) = verify_owned_socket(path, metadata.dev(), metadata.ino()) {
        drop(connection);
        return Err(error);
    }
    Ok(connection)
}

fn claim_at(path: &Path) -> Result<Listener, EndpointError> {
    claim_at_with(path, || {})
}

fn claim_at_with<F>(path: &Path, after_stopping_check: F) -> Result<Listener, EndpointError>
where
    F: FnOnce(),
{
    let stopping_path = stopping_path(path);
    remove_verified_stale_socket(&stopping_path)?;
    after_stopping_check();
    remove_verified_stale_socket(path)?;
    let inner = UnixListener::bind(path).map_err(|source| {
        if source.kind() == std::io::ErrorKind::AddrInUse {
            EndpointError::AlreadyOwned
        } else {
            EndpointError::io("Could not claim the Lens command endpoint", source)
        }
    })?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|source| {
        EndpointError::io("Could not restrict the Lens command endpoint", source)
    })?;
    let metadata = fs::symlink_metadata(path).map_err(|source| {
        EndpointError::io(
            "Could not inspect the claimed Lens command endpoint",
            source,
        )
    })?;
    let listener = Listener {
        inner,
        path: path.to_path_buf(),
        device: metadata.dev(),
        inode: metadata.ino(),
    };
    if let Err(error) = remove_verified_stale_socket(&stopping_path) {
        drop(listener);
        return Err(error);
    }
    Ok(listener)
}

async fn inspect_stopping_endpoint(path: &Path) -> Result<(), EndpointError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(EndpointError::io(
                "Could not inspect a stopping Lens command endpoint",
                source,
            ));
        }
    };
    validate_owned_socket(path, &metadata)?;
    match connect_verified_at(path).await {
        Ok(connection) => {
            drop(connection);
            Err(EndpointError::ServiceStopping)
        }
        Err(error) if error.is_unavailable() => Ok(()),
        Err(error) => Err(error),
    }
}

fn stopping_path(path: &Path) -> PathBuf {
    let mut file_name = path
        .file_name()
        .expect("endpoint path should have a file name")
        .to_os_string();
    file_name.push(STOPPING_SUFFIX);
    path.with_file_name(file_name)
}

fn verify_owned_socket(path: &Path, device: u64, inode: u64) -> Result<(), EndpointError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| {
        EndpointError::io("Could not verify the owned Lens command endpoint", source)
    })?;
    validate_owned_socket(path, &metadata)?;
    if metadata.dev() != device || metadata.ino() != inode {
        return Err(EndpointError::UnsafeEndpoint {
            path: path.to_path_buf(),
            reason: "the endpoint changed during identity verification".to_owned(),
        });
    }
    Ok(())
}

fn validate_owned_socket(path: &Path, metadata: &fs::Metadata) -> Result<(), EndpointError> {
    if !metadata.file_type().is_socket() {
        return Err(EndpointError::UnsafeEndpoint {
            path: path.to_path_buf(),
            reason: "the existing path is not a Unix socket".to_owned(),
        });
    }
    let owner = effective_user_id();
    if metadata.uid() != owner {
        return Err(EndpointError::UnsafeEndpoint {
            path: path.to_path_buf(),
            reason: format!(
                "socket owner user {} does not match current user {owner}",
                metadata.uid()
            ),
        });
    }
    Ok(())
}

fn remove_verified_stale_socket(path: &Path) -> Result<(), EndpointError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(EndpointError::io(
                "Could not inspect an existing Lens command endpoint",
                source,
            ));
        }
    };

    validate_owned_socket(path, &metadata)?;
    if std::os::unix::net::UnixStream::connect(path).is_ok() {
        return Err(EndpointError::AlreadyOwned);
    }
    let owner = effective_user_id();
    let verified = fs::symlink_metadata(path).map_err(|source| {
        EndpointError::io("Could not recheck a stale Lens command endpoint", source)
    })?;
    if !verified.file_type().is_socket()
        || verified.uid() != owner
        || verified.dev() != metadata.dev()
        || verified.ino() != metadata.ino()
    {
        return Err(EndpointError::UnsafeEndpoint {
            path: path.to_path_buf(),
            reason: "the endpoint changed during stale-socket verification".to_owned(),
        });
    }
    fs::remove_file(path).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            return EndpointError::AlreadyOwned;
        }
        EndpointError::io("Could not remove a stale Lens command endpoint", source)
    })
}

fn authorize_user(peer: u32, owner: u32) -> Result<(), EndpointError> {
    if peer == owner {
        Ok(())
    } else {
        Err(EndpointError::UnauthorizedPeer { peer, owner })
    }
}

fn effective_user_id() -> u32 {
    // SAFETY: geteuid has no preconditions and does not dereference pointers.
    unsafe { libc::geteuid() }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        os::unix::{
            fs::{symlink, DirBuilderExt, MetadataExt, PermissionsExt},
            net::UnixListener as StdUnixListener,
        },
        sync::{mpsc, Arc},
        thread,
    };

    use tokio::sync::Barrier;

    use super::{
        authorize, authorize_user, claim_at, claim_at_with, connect_at, connect_verified_at,
        effective_user_id, prepare_runtime_directory, EndpointError,
    };

    #[tokio::test]
    async fn unclaimed_private_endpoint_then_listener_becomes_owner() {
        // Arrange
        let (root, path) = endpoint_fixture("claim");

        // Act
        let listener = claim_at(&path);

        // Assert
        assert!(listener.is_ok());
        drop(listener);
        fs::remove_dir_all(root).expect("test fixture should be removable");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn concurrent_claims_then_exactly_one_listener_owns_endpoint() {
        // Arrange
        let (root, path) = endpoint_fixture("concurrent");
        let barrier = Arc::new(Barrier::new(2));
        let first_barrier = barrier.clone();
        let first_path = path.clone();
        let first = tokio::spawn(async move {
            first_barrier.wait().await;
            claim_at(&first_path)
        });
        let second = tokio::spawn(async move {
            barrier.wait().await;
            claim_at(&path)
        });

        // Act
        let results = [
            first.await.expect("first claim task should complete"),
            second.await.expect("second claim task should complete"),
        ];

        // Assert
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter(|result| matches!(result, Err(EndpointError::AlreadyOwned)))
                .count(),
            1
        );
        drop(results);
        fs::remove_dir_all(root).expect("test fixture should be removable");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn shutdown_rename_between_claim_checks_then_replacement_claim_is_denied() {
        // Arrange
        let (root, path) = endpoint_fixture("claim-shutdown-race");
        let listener = claim_at(&path).expect("original endpoint should be claimable");
        let (paused_sender, paused_receiver) = mpsc::channel();
        let (resume_sender, resume_receiver) = mpsc::channel();
        let claim_path = path.clone();
        let runtime = tokio::runtime::Handle::current();
        let competing_claim = thread::spawn(move || {
            let _runtime = runtime.enter();
            claim_at_with(&claim_path, move || {
                paused_sender
                    .send(())
                    .expect("claim pause should be observable");
                resume_receiver
                    .recv()
                    .expect("claim should be resumed after shutdown rename");
            })
        });
        paused_receiver
            .recv()
            .expect("claim should pause after its first stopping-barrier check");

        // Act
        let shutdown_barrier = listener
            .begin_shutdown()
            .expect("original owner should begin shutdown");
        resume_sender
            .send(())
            .expect("competing claim should resume");
        let result = competing_claim
            .join()
            .expect("competing claim thread should join");

        // Assert
        assert!(matches!(result, Err(EndpointError::AlreadyOwned)));
        assert!(
            !path.exists(),
            "replacement socket should be removed exactly"
        );
        drop(shutdown_barrier);
        let replacement = claim_at(&path).expect("claim should succeed after barrier release");
        drop(replacement);
        fs::remove_dir_all(root).expect("test fixture should be removable");
    }

    #[tokio::test]
    async fn live_symlink_endpoint_then_connection_fails_before_foreign_listener_is_reached() {
        // Arrange
        let (root, path) = endpoint_fixture("live-symlink");
        let foreign_path = root.join("foreign.sock");
        let foreign =
            StdUnixListener::bind(&foreign_path).expect("foreign endpoint should be bindable");
        foreign
            .set_nonblocking(true)
            .expect("foreign endpoint should be nonblocking");
        symlink(&foreign_path, &path).expect("endpoint symlink should be creatable");

        // Act
        let result = connect_verified_at(&path).await;
        let foreign_connection = foreign.accept();

        // Assert
        assert!(matches!(result, Err(EndpointError::UnsafeEndpoint { .. })));
        assert!(matches!(
            foreign_connection,
            Err(ref error) if error.kind() == std::io::ErrorKind::WouldBlock
        ));
        assert!(
            fs::symlink_metadata(&path)
                .expect("endpoint symlink should remain")
                .file_type()
                .is_symlink(),
            "unsafe endpoint symlink must be preserved"
        );
        fs::remove_dir_all(root).expect("test fixture should be removable");
    }

    #[tokio::test]
    async fn shutdown_barrier_then_endpoint_is_hidden_and_replacement_claim_is_blocked() {
        // Arrange
        let (root, path) = endpoint_fixture("shutdown-barrier");
        let listener = claim_at(&path).expect("endpoint should be claimable");

        // Act
        let barrier = listener
            .begin_shutdown()
            .expect("shutdown barrier should be retained");
        let original_connection = connect_at(&path).await;
        let competing_claim = claim_at(&path);

        // Assert
        assert!(matches!(
            original_connection,
            Err(EndpointError::Io { ref source, .. }) if matches!(
                source.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
            )
        ));
        assert!(matches!(competing_claim, Err(EndpointError::AlreadyOwned)));
        drop(barrier);
        let replacement = claim_at(&path).expect("claim should succeed after response barrier");
        drop(replacement);
        fs::remove_dir_all(root).expect("test fixture should be removable");
    }

    #[tokio::test]
    async fn stale_owned_socket_then_next_claim_recovers_without_manual_cleanup() {
        // Arrange
        let (root, path) = endpoint_fixture("stale");
        let stale = StdUnixListener::bind(&path).expect("stale socket should bind");
        drop(stale);

        // Act
        let listener = claim_at(&path);

        // Assert
        assert!(listener.is_ok());
        drop(listener);
        fs::remove_dir_all(root).expect("test fixture should be removable");
    }

    #[test]
    fn non_socket_endpoint_then_claim_preserves_and_rejects_path() {
        // Arrange
        let (root, path) = endpoint_fixture("regular-file");
        fs::write(&path, "not a socket").expect("endpoint fixture should be writable");

        // Act
        let result = claim_at(&path);

        // Assert
        assert!(matches!(result, Err(EndpointError::UnsafeEndpoint { .. })));
        assert_eq!(
            fs::read_to_string(&path).expect("unsafe endpoint should remain"),
            "not a socket"
        );
        fs::remove_dir_all(root).expect("test fixture should be removable");
    }

    #[tokio::test]
    async fn same_user_connection_then_socket_is_private_and_peer_is_authorized() {
        // Arrange
        let (root, path) = endpoint_fixture("authorization");
        let mut listener = claim_at(&path).expect("endpoint should be claimable");
        let mode = fs::symlink_metadata(&path)
            .expect("endpoint metadata should be readable")
            .mode()
            & 0o777;

        // Act
        let (client, server) = tokio::join!(connect_at(&path), listener.accept());
        let client = client.expect("client should connect");
        let server = server.expect("listener should accept");
        let authorization = authorize(&server);

        // Assert
        assert_eq!(mode, 0o600);
        assert!(authorization.is_ok());
        drop((client, server, listener));
        fs::remove_dir_all(root).expect("test fixture should be removable");
    }

    #[test]
    fn different_user_peer_then_authorization_is_rejected() {
        // Arrange
        let owner = effective_user_id();
        let peer = owner.wrapping_add(1);

        // Act
        let result = authorize_user(peer, owner);

        // Assert
        assert!(matches!(
            result,
            Err(EndpointError::UnauthorizedPeer {
                peer: rejected,
                owner: expected
            }) if rejected == peer && expected == owner
        ));
    }

    #[test]
    fn group_accessible_runtime_directory_then_endpoint_setup_is_rejected() {
        // Arrange
        let (root, _) = endpoint_fixture("runtime-permissions");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o750))
            .expect("fixture permissions should change");

        // Act
        let result = prepare_runtime_directory(&root);

        // Assert
        assert!(matches!(
            result,
            Err(EndpointError::UnsafeRuntimeDirectory { .. })
        ));
        fs::remove_dir_all(root).expect("test fixture should be removable");
    }

    fn endpoint_fixture(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let root =
            std::env::temp_dir().join(format!("lens-endpoint-{}-{name}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).expect("stale test fixture should be removable");
        }
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .expect("private runtime directory should be creatable");
        let path = root.join("service.sock");
        (root, path)
    }
}
