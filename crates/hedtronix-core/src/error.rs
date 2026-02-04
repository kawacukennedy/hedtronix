//! Error types for HEDTRONIX

use thiserror::Error;

/// Result type alias for HEDTRONIX operations
pub type Result<T> = std::result::Result<T, Error>;

/// Core error types for the HEDTRONIX system
#[derive(Error, Debug)]
pub enum Error {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {entity} with id {id}")]
    NotFound { entity: String, id: String },

    #[error("Duplicate entry: {0}")]
    Duplicate(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Device not registered")]
    DeviceNotRegistered,

    #[error("Device revoked")]
    DeviceRevoked,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Sync conflict: {0}")]
    SyncConflict(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("CRDT merge error: {0}")]
    CrdtMerge(String),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}
