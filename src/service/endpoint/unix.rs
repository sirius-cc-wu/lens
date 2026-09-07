use std::{
    env, fs,
    os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
};

use tokio::net::{UnixListener, UnixStream};

use super::EndpointError;

const SOCKET_NAME: &str = "service-v1.sock";

pub(crate) type ClientConnection = UnixStream;
pub(crate) type ServerConnection = UnixStream;

pub(crate) struct Listener {
    inner: UnixListener,
    path: PathBuf,
    _lock: fs::File,
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
    connect_at(&path).await
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
    let xdg = env::var_os("XDG_RUNTIME_DIR").filter(|value| !value.is_empty());
    let xdg_path = xdg.as_ref().map(Path::new);
    let temp_dir = env::temp_dir();
    let directory = runtime_directory_for(xdg_path, &temp_dir, effective_user_id());
    prepare_runtime_directory(&directory)?;
    Ok(directory)
}

fn runtime_directory_for(xdg: Option<&Path>, fallback_parent: &Path, uid: u32) -> PathBuf {
    match xdg {
        Some(base) => base.join("lens"),
        None => fallback_parent.join(format!("lens-runtime-{uid}")),
    }
}

fn prepare_runtime_directory(path: &Path) -> Result<(), EndpointError> {
    match fs::DirBuilder::new().mode(0o700).create(path) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(source) => {
            return Err(EndpointError::io(
                "Could not create the private Lens runtime directory",
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

fn lock_path_for(path: &Path) -> PathBuf {
    path.with_extension("lock")
}

fn acquire_lock(path: &Path) -> Result<fs::File, EndpointError> {
    use std::os::unix::fs::OpenOptionsExt;
    use std::os::unix::io::AsRawFd;

    let lock_path = lock_path_for(path);
    let lock_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(lock_path)
        .map_err(|source| EndpointError::io("Could not open the Lens command lock file", source))?;

    // SAFETY: flock is called on a valid open file descriptor with standard non-blocking exclusive lock flags.
    let rc = unsafe { libc::flock(lock_file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if rc != 0 {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::EWOULDBLOCK) || err.raw_os_error() == Some(libc::EAGAIN)
        {
            return Err(EndpointError::AlreadyOwned);
        }
        return Err(EndpointError::io(
            "Could not lock the Lens command lock file",
            err,
        ));
    }

    Ok(lock_file)
}

fn claim_at(path: &Path) -> Result<Listener, EndpointError> {
    let lock_file = acquire_lock(path)?;
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
    Ok(Listener {
        inner,
        path: path.to_path_buf(),
        _lock: lock_file,
        device: metadata.dev(),
        inode: metadata.ino(),
    })
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

    if std::os::unix::net::UnixStream::connect(path).is_ok() {
        return Err(EndpointError::AlreadyOwned);
    }
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
            fs::{DirBuilderExt, MetadataExt, PermissionsExt},
            net::UnixListener as StdUnixListener,
        },
        sync::Arc,
    };

    use tokio::sync::Barrier;

    use super::{
        acquire_lock, authorize, authorize_user, claim_at, connect_at, effective_user_id,
        prepare_runtime_directory, EndpointError,
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

    #[tokio::test]
    async fn xdg_runtime_directory_then_endpoint_is_isolated_in_lens_subdirectory() {
        // Arrange
        let root = std::env::temp_dir().join(format!("lens-xdg-fixture-{}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).expect("stale test fixture should be removable");
        }
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .expect("fixture root should be creatable");
        let unrelated_socket = root.join("service-v1.sock");
        fs::write(&unrelated_socket, "unrelated application socket")
            .expect("unrelated file should be writable");

        let lens_dir = super::runtime_directory_for(
            Some(&root),
            std::path::Path::new("/tmp"),
            effective_user_id(),
        );
        assert_eq!(lens_dir, root.join("lens"));
        prepare_runtime_directory(&lens_dir).expect("lens runtime directory should prepare");
        let socket_path = lens_dir.join(super::SOCKET_NAME);

        // Act
        let listener = claim_at(&socket_path);

        // Assert
        assert!(listener.is_ok());
        assert_eq!(
            fs::read_to_string(&unrelated_socket).expect("unrelated socket should remain"),
            "unrelated application socket"
        );
        assert!(socket_path.exists());
        drop(listener);
        fs::remove_dir_all(root).expect("test fixture should be removable");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_runtime_directory_creations_then_all_contenders_succeed_safely() {
        // Arrange
        let root = std::env::temp_dir().join(format!(
            "lens-runtime-race-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should advance")
                .as_nanos()
        ));
        if root.exists() {
            let _ = fs::remove_dir_all(&root);
        }
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .expect("root directory should be created");
        let target_dir = root.join("lens");
        let contenders = 16;
        let barrier = Arc::new(Barrier::new(contenders));
        let mut handles = Vec::with_capacity(contenders);

        // Act
        for _ in 0..contenders {
            let b = barrier.clone();
            let d = target_dir.clone();
            handles.push(tokio::spawn(async move {
                b.wait().await;
                prepare_runtime_directory(&d)
            }));
        }

        let mut results = Vec::with_capacity(contenders);
        for handle in handles {
            results.push(handle.await.expect("task should complete"));
        }

        // Assert
        for result in results {
            assert!(
                result.is_ok(),
                "every contender should safely prepare the directory: {result:?}"
            );
        }
        let metadata = fs::symlink_metadata(&target_dir).expect("created directory should exist");
        assert!(metadata.file_type().is_dir());
        assert_eq!(metadata.uid(), effective_user_id());
        assert_eq!(metadata.mode() & 0o077, 0);

        let _ = fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn contender_holding_lock_before_listen_then_second_claim_reports_already_owned() {
        // Arrange
        let (root, path) = endpoint_fixture("lock-exclusion");
        let initial_lock = acquire_lock(&path).expect("first contender should acquire lock");

        // Act
        let claim_attempt = claim_at(&path);

        // Assert
        assert!(matches!(claim_attempt, Err(EndpointError::AlreadyOwned)));
        drop(initial_lock);

        let subsequent_claim = claim_at(&path);
        assert!(subsequent_claim.is_ok());
        drop(subsequent_claim);
        fs::remove_dir_all(root).expect("test fixture should be removable");
    }

    #[tokio::test]
    async fn stale_socket_with_released_lock_then_next_claim_cleans_and_succeeds() {
        // Arrange
        let (root, path) = endpoint_fixture("stale-lock-released");
        let lock = acquire_lock(&path).expect("lock should be acquired");
        let stale = StdUnixListener::bind(&path).expect("stale socket should bind");
        drop(lock);
        drop(stale);

        // Act
        let listener = claim_at(&path);

        // Assert
        assert!(listener.is_ok());
        drop(listener);
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
