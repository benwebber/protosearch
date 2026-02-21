//! Error type for this crate.
use std::io;

/// A result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("protobuf error: {0}")]
    Protobuf(#[from] protobuf::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
