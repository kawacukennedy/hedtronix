//! Sync handlers

use axum::{extract::State, Json};
use hedtronix_core::Id;
use hedtronix_core::crdt::Change;
use hedtronix_sync::{
    protocol::{PushRequest, PushResponse, PullRequest, PullResponse, SyncHealth, SyncHealthStatus},
    SyncState,
};

use crate::error::ApiError;
use crate::state::AppState;

/// Push local changes to server
pub async fn push_changes(
    State(state): State<AppState>,
    Json(req): Json<PushRequest>,
) -> Result<Json<PushResponse>, ApiError> {
    let sync_engine = state.sync_engine();
    
    // Apply remote changes
    let result = sync_engine.apply_remote_changes(req.changes)
        .map_err(|e| ApiError::internal(&format!("Sync failed: {}", e)))?;
        
    // In a real implementation we would map conflicts to rejected changes with reasons
    // For now we assume conflicts are rejected
    let rejected = result.conflicts.into_iter()
        .map(|id| hedtronix_sync::RejectedChange {
            change_id: id,
            reason: "Conflict detected".to_string(),
        })
        .collect();
        
    // Calculate acknowledged (all changes not in conflicts are considered successfully processed/resolved)
    let acknowledged = req.changes.iter()
        .map(|c| c.id)
        .filter(|id| !result.conflicts.contains(id))
        .collect();
    
    Ok(Json(PushResponse {
        acknowledged,
        rejected,
        server_time: chrono::Utc::now(),
    }))
}

/// Pull changes from server
pub async fn pull_changes(
    State(state): State<AppState>,
    Json(req): Json<PullRequest>,
) -> Result<Json<PullResponse>, ApiError> {
    let sync_engine = state.sync_engine();
    
    // Get pending changes for the client
    let limit = req.limit.unwrap_or(100);
    // In a real implementation we would filter by 'since' timestamp and 'entity_types'
    // For now we just get pending changes from the queue
    let changes = sync_engine.get_pending_changes(limit)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
        
    // Mark these as synced (in a real app we'd wait for ack)
    // For MVP we assume successful delivery
    let change_ids: Vec<hedtronix_core::Id> = changes.iter().map(|c| c.id).collect();
    if !change_ids.is_empty() {
         let _ = sync_engine.mark_synced(&change_ids);
    }
    
    Ok(Json(PullResponse {
        changes,
        has_more: false, // Would check if count > limit
        next_cursor: None,
        server_time: chrono::Utc::now(),
    }))
}

/// Get sync status
pub async fn get_status(
    State(state): State<AppState>,
) -> Result<Json<SyncStatusResponse>, ApiError> {
    let sync_engine = state.sync_engine();
    let status = sync_engine.get_status();
    
    Ok(Json(SyncStatusResponse {
        state: format!("{:?}", status.state),
        pending_changes: status.pending_changes,
        last_sync: status.last_sync.map(|t| t.to_rfc3339()),
        device_id: status.device_id,
    }))
}

#[derive(Debug, serde::Serialize)]
pub struct SyncStatusResponse {
    pub state: String,
    pub pending_changes: i64,
    pub last_sync: Option<String>,
    pub device_id: String,
}

/// Get sync health
pub async fn get_health(
    State(state): State<AppState>,
) -> Result<Json<SyncHealth>, ApiError> {
    let sync_engine = state.sync_engine();
    let pending = sync_engine.pending_count()
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    let last_sync = sync_engine.get_last_sync()
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    let health = if pending > 100 {
        SyncHealth::warning(state.device_id.clone(), pending, "High number of pending changes")
    } else {
        SyncHealth::healthy(state.device_id.clone(), last_sync)
    };
    
    Ok(Json(health))
}
