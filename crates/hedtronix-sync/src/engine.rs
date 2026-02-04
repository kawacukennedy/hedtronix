//! Sync engine for offline-first operation

use hedtronix_core::{Id, Timestamp};
use hedtronix_core::crdt::{Change, ChangeOperation};
use hedtronix_db::{Database, SyncRepository};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::conflict::{ConflictResolver, ResolutionResult};

/// Sync error types
#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Merge conflict: {0}")]
    Conflict(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Sync in progress")]
    SyncInProgress,
}

/// Result type for sync operations
pub type Result<T> = std::result::Result<T, SyncError>;

/// Sync engine state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncState {
    Idle,
    Syncing,
    Error,
    Offline,
}

/// Sync engine for managing offline-first data synchronization
pub struct SyncEngine {
    db: Database,
    device_id: String,
    state: SyncState,
    last_sync: Option<Timestamp>,
}

impl SyncEngine {
    pub fn new(db: Database, device_id: String) -> Self {
        Self {
            db,
            device_id,
            state: SyncState::Idle,
            last_sync: None,
        }
    }

    /// Get current sync state
    pub fn state(&self) -> SyncState {
        self.state
    }

    /// Queue a local change for sync
    pub fn queue_change(&self, change: Change) -> Result<()> {
        let sync_repo = SyncRepository::new(self.db.clone());
        sync_repo.queue_change(&change)
            .map_err(|e| SyncError::Database(e.to_string()))
    }

    /// Create and queue a create change
    pub fn track_create(
        &self,
        entity_type: &str,
        entity_id: Id,
        data: serde_json::Value,
    ) -> Result<()> {
        let change = Change::create(
            entity_type.to_string(),
            entity_id,
            data,
            self.device_id.clone(),
        );
        self.queue_change(change)
    }

    /// Create and queue an update change
    pub fn track_update(
        &self,
        entity_type: &str,
        entity_id: Id,
        data: serde_json::Value,
    ) -> Result<()> {
        let change = Change::update(
            entity_type.to_string(),
            entity_id,
            data,
            self.device_id.clone(),
        );
        self.queue_change(change)
    }

    /// Create and queue a delete change
    pub fn track_delete(&self, entity_type: &str, entity_id: Id) -> Result<()> {
        let change = Change::delete(
            entity_type.to_string(),
            entity_id,
            self.device_id.clone(),
        );
        self.queue_change(change)
    }

    /// Get pending changes to sync
    pub fn get_pending_changes(&self, limit: u32) -> Result<Vec<Change>> {
        let sync_repo = SyncRepository::new(self.db.clone());
        sync_repo.get_pending_changes(limit)
            .map_err(|e| SyncError::Database(e.to_string()))
    }

    /// Get pending change count
    pub fn pending_count(&self) -> Result<i64> {
        let sync_repo = SyncRepository::new(self.db.clone());
        sync_repo.pending_count()
            .map_err(|e| SyncError::Database(e.to_string()))
    }

    /// Apply remote changes locally
    pub fn apply_remote_changes(&self, changes: Vec<Change>) -> Result<ApplyResult> {
        let mut applied = 0;
        let mut conflicts = Vec::new();
        let resolver = ConflictResolver::new();

        for change in changes {
            match self.apply_single_change(&change, &resolver) {
                Ok(()) => applied += 1,
                Err(SyncError::Conflict(msg)) => {
                    conflicts.push(change.entity_id);
                }
                Err(e) => return Err(e),
            }
        }

        Ok(ApplyResult { applied, conflicts })
    }

    fn apply_single_change(
        &self,
        change: &Change,
        resolver: &ConflictResolver,
    ) -> Result<()> {
        // Check for local changes to the same entity
        let sync_repo = SyncRepository::new(self.db.clone());
        let local_changes = sync_repo.get_pending_changes(100)
            .map_err(|e| SyncError::Database(e.to_string()))?;

        let conflicting = local_changes.iter()
            .find(|c| c.entity_id == change.entity_id && c.entity_type == change.entity_type);

        if let Some(local) = conflicting {
            // Resolve conflict using CRDT strategy
            let result = resolver.resolve(local, change);
            match result {
                ResolutionResult::KeepLocal => {
                    // Local wins, ignore remote
                    Ok(())
                }
                ResolutionResult::KeepRemote => {
                    // Remote wins, apply it
                    self.apply_change_to_db(change)
                }
                ResolutionResult::Merge(merged) => {
                    // Apply merged data
                    self.apply_change_to_db(&merged)
                }
                ResolutionResult::Conflict => {
                    // Manual resolution needed
                    Err(SyncError::Conflict(format!(
                        "Conflict on {} {}",
                        change.entity_type, change.entity_id
                    )))
                }
            }
        } else {
            // No conflict, apply directly
            self.apply_change_to_db(change)
        }
    }

    fn apply_change_to_db(&self, change: &Change) -> Result<()> {
        // This would dispatch to the appropriate repository based on entity_type
        // For now, just log that it would be applied
        tracing::info!(
            "Applying {} {:?} for {}",
            change.entity_type,
            change.operation,
            change.entity_id
        );
        Ok(())
    }

    /// Mark changes as synced
    pub fn mark_synced(&self, change_ids: &[Id]) -> Result<()> {
        let sync_repo = SyncRepository::new(self.db.clone());
        sync_repo.mark_synced(change_ids)
            .map_err(|e| SyncError::Database(e.to_string()))
    }

    /// Update last sync time
    pub fn set_last_sync(&mut self, time: Timestamp) -> Result<()> {
        let sync_repo = SyncRepository::new(self.db.clone());
        sync_repo.set_last_sync_time(time)
            .map_err(|e| SyncError::Database(e.to_string()))?;
        self.last_sync = Some(time);
        Ok(())
    }

    /// Get last sync time
    pub fn get_last_sync(&self) -> Result<Option<Timestamp>> {
        if let Some(time) = self.last_sync {
            return Ok(Some(time));
        }
        let sync_repo = SyncRepository::new(self.db.clone());
        sync_repo.get_last_sync_time()
            .map_err(|e| SyncError::Database(e.to_string()))
    }

    /// Get sync status for UI display
    pub fn get_status(&self) -> SyncStatus {
        let pending = self.pending_count().unwrap_or(0);
        SyncStatus {
            state: self.state,
            pending_changes: pending,
            last_sync: self.last_sync,
            device_id: self.device_id.clone(),
        }
    }
}

/// Result of applying remote changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    pub applied: usize,
    pub conflicts: Vec<Id>,
}

/// Sync status for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub state: SyncState,
    pub pending_changes: i64,
    pub last_sync: Option<Timestamp>,
    pub device_id: String,
}

/// Sync request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    pub device_id: String,
    pub last_sync: Option<Timestamp>,
    pub changes: Vec<Change>,
}

/// Sync response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub changes: Vec<Change>,
    pub server_time: Timestamp,
    pub acknowledged: Vec<Id>,
}
