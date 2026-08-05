use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::TargetScope;

pub(crate) const MAX_FRAME_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct ProtocolVersion(u16);

impl ProtocolVersion {
    pub(crate) const CURRENT: Self = Self(1);
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct RequestId([u8; 16]);

impl RequestId {
    pub(crate) fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "platform", content = "units", rename_all = "snake_case")]
pub(crate) enum WirePath {
    Unix(Vec<u8>),
    Windows(Vec<u16>),
}

impl WirePath {
    pub(crate) fn from_path(path: &Path) -> Self {
        wire_path_from_native(path)
    }

    pub(crate) fn into_path(self) -> Result<PathBuf, ProtocolError> {
        wire_path_into_native(self)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct OpenRequest {
    pub(crate) protocol_version: ProtocolVersion,
    pub(crate) request_id: RequestId,
    pub(crate) invocation_directory: WirePath,
    pub(crate) target: Option<WirePath>,
    pub(crate) scope: TargetScope,
    pub(crate) plantuml_server: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", content = "body", rename_all = "snake_case")]
pub(crate) enum ServiceRequest {
    Open(OpenRequest),
}

impl ServiceRequest {
    pub(crate) fn validate_version(&self) -> Result<(), ProtocolError> {
        let received = match self {
            Self::Open(request) => request.protocol_version,
        };
        if received == ProtocolVersion::CURRENT {
            Ok(())
        } else {
            Err(ProtocolError::IncompatibleVersion {
                received,
                supported: ProtocolVersion::CURRENT,
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OpenErrorCode {
    Target,
    Session,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct OpenError {
    pub(crate) code: OpenErrorCode,
    pub(crate) message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", content = "body", rename_all = "snake_case")]
pub(crate) enum ServiceResponse {
    Ready {
        request_id: RequestId,
        view_url: String,
    },
    Rejected {
        request_id: RequestId,
        error: OpenError,
    },
    Incompatible {
        supported_version: ProtocolVersion,
    },
}

#[derive(Debug, Error)]
pub(crate) enum ProtocolError {
    #[error("I/O failed while transferring a protocol frame: {0}")]
    Io(#[from] std::io::Error),
    #[error("protocol frame declares {declared} bytes; maximum is {maximum}")]
    OversizedFrame { declared: usize, maximum: usize },
    #[error("protocol frame contains malformed JSON: {0}")]
    MalformedJson(#[from] serde_json::Error),
    #[error("protocol version {received:?} is incompatible; supported version is {supported:?}")]
    IncompatibleVersion {
        received: ProtocolVersion,
        supported: ProtocolVersion,
    },
    #[error("received {received} path encoding on a platform that requires {expected}")]
    WrongPathEncoding {
        received: &'static str,
        expected: &'static str,
    },
    #[error("native path contains a null unit")]
    InvalidNativePath,
}

pub(crate) async fn write_frame<W, T>(writer: &mut W, message: &T) -> Result<(), ProtocolError>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    let payload = serde_json::to_vec(message)?;
    ensure_frame_size(payload.len())?;
    writer
        .write_all(&(payload.len() as u32).to_be_bytes())
        .await?;
    writer.write_all(&payload).await?;
    writer.flush().await?;
    Ok(())
}

pub(crate) async fn read_frame<R, T>(reader: &mut R) -> Result<T, ProtocolError>
where
    R: AsyncRead + Unpin,
    T: DeserializeOwned,
{
    let mut prefix = [0; 4];
    reader.read_exact(&mut prefix).await?;
    let declared = u32::from_be_bytes(prefix) as usize;
    ensure_frame_size(declared)?;

    let mut payload = vec![0; declared];
    reader.read_exact(&mut payload).await?;
    Ok(serde_json::from_slice(&payload)?)
}

fn ensure_frame_size(size: usize) -> Result<(), ProtocolError> {
    if size > MAX_FRAME_BYTES {
        Err(ProtocolError::OversizedFrame {
            declared: size,
            maximum: MAX_FRAME_BYTES,
        })
    } else {
        Ok(())
    }
}

#[cfg(unix)]
fn wire_path_from_native(path: &Path) -> WirePath {
    use std::os::unix::ffi::OsStrExt;

    WirePath::Unix(path.as_os_str().as_bytes().to_vec())
}

#[cfg(windows)]
fn wire_path_from_native(path: &Path) -> WirePath {
    use std::os::windows::ffi::OsStrExt;

    WirePath::Windows(path.as_os_str().encode_wide().collect())
}

#[cfg(unix)]
fn wire_path_into_native(path: WirePath) -> Result<PathBuf, ProtocolError> {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    match path {
        WirePath::Unix(bytes) if bytes.contains(&0) => Err(ProtocolError::InvalidNativePath),
        WirePath::Unix(bytes) => Ok(PathBuf::from(OsString::from_vec(bytes))),
        WirePath::Windows(_) => Err(ProtocolError::WrongPathEncoding {
            received: "Windows",
            expected: "Unix",
        }),
    }
}

#[cfg(windows)]
fn wire_path_into_native(path: WirePath) -> Result<PathBuf, ProtocolError> {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt};

    match path {
        WirePath::Windows(units) if units.contains(&0) => Err(ProtocolError::InvalidNativePath),
        WirePath::Windows(units) => Ok(PathBuf::from(OsString::from_wide(&units))),
        WirePath::Unix(_) => Err(ProtocolError::WrongPathEncoding {
            received: "Unix",
            expected: "Windows",
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde::{de::DeserializeOwned, Serialize};
    use tokio::io::{duplex, AsyncWriteExt};

    use super::{
        read_frame, write_frame, OpenError, OpenErrorCode, OpenRequest, ProtocolError,
        ProtocolVersion, RequestId, ServiceRequest, ServiceResponse, WirePath, MAX_FRAME_BYTES,
    };
    use crate::TargetScope;

    #[tokio::test]
    async fn framed_request_then_preserves_version_scope_and_native_paths() {
        // Arrange
        let invocation_directory = native_test_path();
        let target = invocation_directory.join("guide.md");
        let request = ServiceRequest::Open(OpenRequest {
            protocol_version: ProtocolVersion::CURRENT,
            request_id: request_id(1),
            invocation_directory: WirePath::from_path(&invocation_directory),
            target: Some(WirePath::from_path(&target)),
            scope: TargetScope::Target,
            plantuml_server: Some("http://127.0.0.1:8080/plantuml".to_owned()),
        });

        // Act
        let decoded: ServiceRequest = framed_round_trip(&request).await;

        // Assert
        assert_eq!(decoded, request);
        let ServiceRequest::Open(decoded) = decoded;
        assert_eq!(
            decoded
                .invocation_directory
                .into_path()
                .expect("native invocation path should decode"),
            invocation_directory
        );
        assert_eq!(
            decoded
                .target
                .expect("target should remain present")
                .into_path()
                .expect("native target path should decode"),
            target
        );
    }

    #[tokio::test]
    async fn oversized_frame_then_rejects_declared_size_before_payload_read() {
        // Arrange
        let (mut writer, mut reader) = duplex(16);
        writer
            .write_all(&((MAX_FRAME_BYTES + 1) as u32).to_be_bytes())
            .await
            .expect("frame prefix should be writable");

        // Act
        let result = read_frame::<_, ServiceRequest>(&mut reader).await;

        // Assert
        assert!(matches!(
            result,
            Err(ProtocolError::OversizedFrame {
                declared,
                maximum: MAX_FRAME_BYTES
            }) if declared == MAX_FRAME_BYTES + 1
        ));
    }

    #[tokio::test]
    async fn malformed_json_frame_then_returns_typed_protocol_error() {
        // Arrange
        let malformed = b"{not-json";
        let (mut writer, mut reader) = duplex(32);
        writer
            .write_all(&(malformed.len() as u32).to_be_bytes())
            .await
            .expect("frame prefix should be writable");
        writer
            .write_all(malformed)
            .await
            .expect("frame payload should be writable");

        // Act
        let result = read_frame::<_, ServiceRequest>(&mut reader).await;

        // Assert
        assert!(matches!(result, Err(ProtocolError::MalformedJson(_))));
    }

    #[tokio::test]
    async fn service_responses_then_preserve_each_typed_outcome() {
        // Arrange
        let responses = [
            ServiceResponse::Ready {
                request_id: request_id(2),
                view_url: "http://127.0.0.1:41000".to_owned(),
            },
            ServiceResponse::Rejected {
                request_id: request_id(3),
                error: OpenError {
                    code: OpenErrorCode::Target,
                    message: "Target guide.md does not exist".to_owned(),
                },
            },
            ServiceResponse::Incompatible {
                supported_version: ProtocolVersion::CURRENT,
            },
        ];

        // Act
        let mut decoded = Vec::new();
        for response in &responses {
            decoded.push(framed_round_trip(response).await);
        }

        // Assert
        assert_eq!(decoded, responses);
    }

    #[test]
    fn incompatible_request_version_then_reports_supported_version() {
        // Arrange
        let request = ServiceRequest::Open(OpenRequest {
            protocol_version: ProtocolVersion(99),
            request_id: request_id(4),
            invocation_directory: WirePath::from_path(&native_test_path()),
            target: None,
            scope: TargetScope::Repository,
            plantuml_server: None,
        });

        // Act
        let result = request.validate_version();

        // Assert
        assert!(matches!(
            result,
            Err(ProtocolError::IncompatibleVersion {
                received: ProtocolVersion(99),
                supported: ProtocolVersion::CURRENT
            })
        ));
    }

    #[test]
    fn wrong_platform_path_then_rejects_before_target_resolution() {
        // Arrange
        #[cfg(unix)]
        let path = WirePath::Windows(vec![b'C' as u16, b':' as u16]);
        #[cfg(windows)]
        let path = WirePath::Unix(b"/tmp/guide.md".to_vec());

        // Act
        let result = path.into_path();

        // Assert
        assert!(matches!(
            result,
            Err(ProtocolError::WrongPathEncoding { .. })
        ));
    }

    #[test]
    fn native_path_with_null_unit_then_rejects_before_target_resolution() {
        // Arrange
        #[cfg(unix)]
        let path = WirePath::Unix(b"/tmp/lens\0guide.md".to_vec());
        #[cfg(windows)]
        let path = WirePath::Windows(vec![b'C' as u16, b':' as u16, 0, b'g' as u16]);

        // Act
        let result = path.into_path();

        // Assert
        assert!(matches!(result, Err(ProtocolError::InvalidNativePath)));
    }

    async fn framed_round_trip<T>(message: &T) -> T
    where
        T: DeserializeOwned + Serialize,
    {
        let (mut writer, mut reader) = duplex(MAX_FRAME_BYTES * 2);
        let (write_result, read_result) = tokio::join!(
            write_frame(&mut writer, message),
            read_frame::<_, T>(&mut reader)
        );
        write_result.expect("test frame should encode");
        read_result.expect("test frame should decode")
    }

    fn request_id(marker: u8) -> RequestId {
        RequestId::from_bytes([marker; 16])
    }

    #[cfg(unix)]
    fn native_test_path() -> PathBuf {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};

        PathBuf::from(OsString::from_vec(b"/tmp/lens-\xff".to_vec()))
    }

    #[cfg(windows)]
    fn native_test_path() -> PathBuf {
        use std::{ffi::OsString, os::windows::ffi::OsStringExt};

        PathBuf::from(OsString::from_wide(&[
            b'C' as u16,
            b':' as u16,
            b'\\' as u16,
            0xD800,
        ]))
    }
}
