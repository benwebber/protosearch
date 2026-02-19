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
    /// The OpenAPI specification is invalid.
    #[error("Invalid spec: {0}")]
    InvalidSpec(String),
    /// On merging fields, a new field conflicts with an existing field.
    #[error("field conflict: field {0} exists with a different type")]
    FieldConflict(String),
    /// On merging fields, a new tag conflicts with an existing tag.
    #[error("tag conflict: tag {tag} is assigned to both {current} and {other}")]
    TagConflict {
        tag: u32,
        current: String,
        other: String,
    },
    /// On merging files, the new package name conflicts with the existing file.
    #[error("package conflict: cannot merge {other} into {current}")]
    PackageConflict { current: String, other: String },
}
