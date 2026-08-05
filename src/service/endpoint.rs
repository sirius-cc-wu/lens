use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum EndpointError {
    #[error("another Lens background service owns the current user's endpoint")]
    AlreadyOwned,
    #[error("unsafe Lens runtime directory {path}: {reason}")]
    UnsafeRuntimeDirectory { path: PathBuf, reason: String },
    #[error("unsafe existing Lens endpoint {path}: {reason}")]
    UnsafeEndpoint { path: PathBuf, reason: String },
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
