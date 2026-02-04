//! API error handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

/// API error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub code: Option<String>,
}

/// API error type
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub error: String,
    pub message: String,
    pub code: Option<String>,
}

impl ApiError {
    pub fn new(status: StatusCode, error: &str, message: &str) -> Self {
        Self {
            status,
            error: error.to_string(),
            message: message.to_string(),
            code: None,
        }
    }

    pub fn with_code(mut self, code: &str) -> Self {
        self.code = Some(code.to_string());
        self
    }

    pub fn unauthorized(message: &str) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "Unauthorized", message)
    }

    pub fn forbidden(message: &str) -> Self {
        Self::new(StatusCode::FORBIDDEN, "Forbidden", message)
    }

    pub fn not_found(entity: &str) -> Self {
        Self::new(StatusCode::NOT_FOUND, "Not Found", &format!("{} not found", entity))
    }

    pub fn bad_request(message: &str) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "Bad Request", message)
    }

    pub fn conflict(message: &str) -> Self {
        Self::new(StatusCode::CONFLICT, "Conflict", message)
    }

    pub fn internal(message: &str) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error", message)
    }

    pub fn validation(message: &str) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, "Validation Error", message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorResponse {
            error: self.error,
            message: self.message,
            code: self.code,
        };
        (self.status, Json(body)).into_response()
    }
}

impl From<hedtronix_db::DbError> for ApiError {
    fn from(e: hedtronix_db::DbError) -> Self {
        match e {
            hedtronix_db::DbError::NotFound(msg) => ApiError::not_found(&msg),
            _ => ApiError::internal(&e.to_string()),
        }
    }
}

impl From<hedtronix_auth::SessionError> for ApiError {
    fn from(e: hedtronix_auth::SessionError) -> Self {
        match e {
            hedtronix_auth::SessionError::InvalidCredentials => {
                ApiError::unauthorized("Invalid email or password")
            }
            hedtronix_auth::SessionError::UserNotFound => {
                ApiError::not_found("User")
            }
            hedtronix_auth::SessionError::UserDisabled => {
                ApiError::forbidden("User account is disabled")
            }
            hedtronix_auth::SessionError::DeviceNotRegistered => {
                ApiError::unauthorized("Device not registered")
            }
            hedtronix_auth::SessionError::DeviceRevoked => {
                ApiError::forbidden("Device has been revoked")
            }
            hedtronix_auth::SessionError::Token(msg) => {
                ApiError::unauthorized(&msg)
            }
            hedtronix_auth::SessionError::Database(msg) => {
                ApiError::internal(&msg)
            }
        }
    }
}

impl From<hedtronix_sync::SyncError> for ApiError {
    fn from(e: hedtronix_sync::SyncError) -> Self {
        match e {
            hedtronix_sync::SyncError::Conflict(msg) => ApiError::conflict(&msg),
            hedtronix_sync::SyncError::Network(msg) => ApiError::internal(&format!("Network error: {}", msg)),
            hedtronix_sync::SyncError::Database(msg) => ApiError::internal(&msg),
            hedtronix_sync::SyncError::Serialization(msg) => ApiError::bad_request(&msg),
            hedtronix_sync::SyncError::SyncInProgress => ApiError::conflict("Sync already in progress"),
        }
    }
}
