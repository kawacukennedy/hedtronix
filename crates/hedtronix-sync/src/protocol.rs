//! Sync protocol definitions

use hedtronix_core::Id;
use hedtronix_core::crdt::Change;
use serde::{Deserialize, Serialize};

/// Sync push request - send local changes to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushRequest {
    pub device_id: String,
    pub changes: Vec<Change>,
    pub client_time: chrono::DateTime<chrono::Utc>,
}

/// Sync push response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResponse {
    pub acknowledged: Vec<Id>,
    pub rejected: Vec<RejectedChange>,
    pub server_time: chrono::DateTime<chrono::Utc>,
}

/// Rejected change with reason
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectedChange {
    pub change_id: Id,
    pub reason: String,
}

/// Sync pull request - get changes from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub device_id: String,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub entity_types: Option<Vec<String>>,
    pub limit: Option<u32>,
}

/// Sync pull response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullResponse {
    pub changes: Vec<Change>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
    pub server_time: chrono::DateTime<chrono::Utc>,
}

/// Full sync request (initial sync or recovery)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullSyncRequest {
    pub device_id: String,
    pub entity_types: Option<Vec<String>>,
}

/// Sync health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHealth {
    pub status: SyncHealthStatus,
    pub pending_changes: i64,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub device_id: String,
    pub message: Option<String>,
}

/// Sync health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SyncHealthStatus {
    Healthy,
    Warning,
    Error,
    Offline,
}

impl SyncHealth {
    pub fn healthy(device_id: String, last_sync: Option<chrono::DateTime<chrono::Utc>>) -> Self {
        Self {
            status: SyncHealthStatus::Healthy,
            pending_changes: 0,
            last_sync,
            device_id,
            message: None,
        }
    }

    pub fn warning(device_id: String, pending: i64, message: &str) -> Self {
        Self {
            status: SyncHealthStatus::Warning,
            pending_changes: pending,
            last_sync: None,
            device_id,
            message: Some(message.to_string()),
        }
    }

    pub fn error(device_id: String, message: &str) -> Self {
        Self {
            status: SyncHealthStatus::Error,
            pending_changes: 0,
            last_sync: None,
            device_id,
            message: Some(message.to_string()),
        }
    }

    pub fn offline(device_id: String, pending: i64) -> Self {
        Self {
            status: SyncHealthStatus::Offline,
            pending_changes: pending,
            last_sync: None,
            device_id,
            message: Some("Device is offline".to_string()),
        }
    }
}
