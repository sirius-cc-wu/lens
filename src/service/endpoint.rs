#[cfg(unix)]
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum EndpointError {
    #[error("another Lens background service owns the current user's endpoint")]
    AlreadyOwned,
    #[error("the Lens background service is stopping")]
    ServiceStopping,
    #[cfg_attr(not(windows), allow(dead_code))]
    #[error("the Lens background service endpoint is temporarily busy")]
    Busy,
    #[cfg(unix)]
    #[error("unsafe Lens runtime directory {path}: {reason}")]
    UnsafeRuntimeDirectory { path: PathBuf, reason: String },
    #[cfg(unix)]
    #[error("unsafe existing Lens endpoint {path}: {reason}")]
    UnsafeEndpoint { path: PathBuf, reason: String },
    #[cfg(unix)]
    #[error("endpoint peer user {peer} does not match owner user {owner}")]
    UnauthorizedPeer { peer: u32, owner: u32 },
    #[error("{context}: {source}")]
    Io {
        context: &'static str,
        #[source]
        source: std::io::Error,
    },
}

impl EndpointError {
    fn io(context: &'static str, source: std::io::Error) -> Self {
        Self::Io { context, source }
    }

    pub(crate) fn is_stopping(&self) -> bool {
        matches!(self, Self::ServiceStopping)
    }

    pub(crate) fn is_busy(&self) -> bool {
        matches!(self, Self::Busy)
    }

    pub(crate) fn is_unavailable(&self) -> bool {
        if self.is_busy() {
            return true;
        }
        let Self::Io { source, .. } = self else {
            return false;
        };
        matches!(
            source.kind(),
            std::io::ErrorKind::NotFound
                | std::io::ErrorKind::ConnectionRefused
                | std::io::ErrorKind::WouldBlock
        ) || matches!(source.raw_os_error(), Some(2 | 3 | 231))
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
    authorize, claim, connect, ClientConnection, Listener, ServerConnection,
};
