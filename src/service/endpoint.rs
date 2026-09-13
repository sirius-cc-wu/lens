#[cfg(unix)]
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum EndpointError {
    #[error("another Lens background service owns the current user's endpoint")]
    AlreadyOwned,
    #[cfg(unix)]
    #[error("unsafe Lens runtime directory {path}: {reason}")]
    UnsafeRuntimeDirectory { path: PathBuf, reason: String },
    #[cfg(unix)]
    #[error("unsafe existing Lens endpoint {path}: {reason}")]
    UnsafeEndpoint { path: PathBuf, reason: String },
    #[cfg(unix)]
    #[error("endpoint peer user {peer} does not match owner user {owner}")]
    UnauthorizedPeer { peer: u32, owner: u32 },
    #[cfg(windows)]
    #[error("endpoint peer identity {peer} does not match owner identity {owner}")]
    UnauthorizedPeer { peer: String, owner: String },
    #[error("{context}: {source}")]
    Io {
        context: &'static str,
        #[source]
        source: std::io::Error,
    },
}

impl EndpointError {
    #[allow(dead_code)]
    pub(crate) fn io(context: &'static str, source: std::io::Error) -> Self {
        Self::Io { context, source }
    }

    pub(crate) fn is_absent(&self) -> bool {
        let Self::Io { source, .. } = self else {
            return false;
        };
        matches!(
            source.kind(),
            std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
        ) || matches!(source.raw_os_error(), Some(2 | 3))
    }

    pub(crate) fn is_busy(&self) -> bool {
        let Self::Io { source, .. } = self else {
            return false;
        };
        matches!(source.kind(), std::io::ErrorKind::WouldBlock)
            || matches!(source.raw_os_error(), Some(231))
    }

    pub(crate) fn is_unavailable(&self) -> bool {
        self.is_absent() || self.is_busy()
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::EndpointError;

    #[test]
    fn not_found_io_error_then_is_absent_and_unavailable() {
        // Arrange
        let error = EndpointError::io("test", io::Error::new(io::ErrorKind::NotFound, "not found"));

        // Act & Assert
        assert!(error.is_absent());
        assert!(!error.is_busy());
        assert!(error.is_unavailable());
    }

    #[test]
    fn connection_refused_io_error_then_is_absent_and_unavailable() {
        // Arrange
        let error = EndpointError::io(
            "test",
            io::Error::new(io::ErrorKind::ConnectionRefused, "refused"),
        );

        // Act & Assert
        assert!(error.is_absent());
        assert!(!error.is_busy());
        assert!(error.is_unavailable());
    }

    #[test]
    fn pipe_busy_raw_os_error_then_is_busy_and_unavailable_but_not_absent() {
        // Arrange
        let error = EndpointError::io("test", io::Error::from_raw_os_error(231));

        // Act & Assert
        assert!(!error.is_absent());
        assert!(error.is_busy());
        assert!(error.is_unavailable());
    }

    #[test]
    fn would_block_io_error_then_is_busy_and_unavailable_but_not_absent() {
        // Arrange
        let error = EndpointError::io(
            "test",
            io::Error::new(io::ErrorKind::WouldBlock, "would block"),
        );

        // Act & Assert
        assert!(!error.is_absent());
        assert!(error.is_busy());
        assert!(error.is_unavailable());
    }

    #[test]
    fn other_io_error_then_is_neither_absent_nor_busy() {
        // Arrange
        let error = EndpointError::io(
            "test",
            io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
        );

        // Act & Assert
        assert!(!error.is_absent());
        assert!(!error.is_busy());
        assert!(!error.is_unavailable());
    }
}

#[cfg(unix)]
#[path = "endpoint/unix.rs"]
mod platform;

#[cfg(windows)]
#[path = "endpoint/windows.rs"]
mod platform;

#[allow(unused_imports)]
pub(crate) use platform::{
    authorize, claim, clean_stale_endpoint, connect, ClientConnection, Listener, ServerConnection,
};
