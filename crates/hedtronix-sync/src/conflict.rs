//! Conflict resolution for sync

use hedtronix_core::Id;
use hedtronix_core::crdt::{Change, ChangeOperation};
use serde::{Deserialize, Serialize};

/// Result of conflict resolution
#[derive(Debug, Clone)]
pub enum ResolutionResult {
    KeepLocal,
    KeepRemote,
    Merge(Change),
    Conflict, // Needs manual resolution
}

/// Conflict resolver using CRDT strategies
pub struct ConflictResolver;

impl ConflictResolver {
    pub fn new() -> Self {
        Self
    }

    /// Resolve a conflict between local and remote changes
    pub fn resolve(&self, local: &Change, remote: &Change) -> ResolutionResult {
        // Same entity, different operations
        match (&local.operation, &remote.operation) {
            // Delete always wins (delete bias)
            (ChangeOperation::Delete, _) => ResolutionResult::KeepLocal,
            (_, ChangeOperation::Delete) => ResolutionResult::KeepRemote,
            
            // Create conflicts - shouldn't happen with UUIDs, but use timestamp
            (ChangeOperation::Create, ChangeOperation::Create) => {
                if local.timestamp >= remote.timestamp {
                    ResolutionResult::KeepLocal
                } else {
                    ResolutionResult::KeepRemote
                }
            }
            
            // Update conflicts - try to merge
            (ChangeOperation::Update, ChangeOperation::Update) => {
                self.merge_updates(local, remote)
            }
            
            // Create vs Update - the create should come first
            (ChangeOperation::Create, ChangeOperation::Update) => ResolutionResult::KeepRemote,
            (ChangeOperation::Update, ChangeOperation::Create) => ResolutionResult::KeepLocal,
        }
    }

    /// Try to merge two update operations
    fn merge_updates(&self, local: &Change, remote: &Change) -> ResolutionResult {
        // Check if changes are to different fields
        let local_obj = local.data.as_object();
        let remote_obj = remote.data.as_object();

        match (local_obj, remote_obj) {
            (Some(l), Some(r)) => {
                // Check for overlapping fields
                let local_keys: std::collections::HashSet<_> = l.keys().collect();
                let remote_keys: std::collections::HashSet<_> = r.keys().collect();
                let overlap: Vec<_> = local_keys.intersection(&remote_keys).collect();

                if overlap.is_empty() {
                    // No overlapping fields, merge them
                    let mut merged = l.clone();
                    for (k, v) in r {
                        merged.insert(k.clone(), v.clone());
                    }
                    let merged_change = Change {
                        id: Id::new_v4(),
                        entity_type: local.entity_type.clone(),
                        entity_id: local.entity_id,
                        operation: ChangeOperation::Update,
                        data: serde_json::Value::Object(merged),
                        timestamp: std::cmp::max(local.timestamp, remote.timestamp),
                        device_id: format!("{}_merged", local.device_id),
                        version: local.version.clone(),
                    };
                    ResolutionResult::Merge(merged_change)
                } else {
                    // Overlapping fields - use Last Write Wins
                    if local.timestamp >= remote.timestamp {
                        ResolutionResult::KeepLocal
                    } else {
                        ResolutionResult::KeepRemote
                    }
                }
            }
            _ => {
                // Can't merge non-object data, use LWW
                if local.timestamp >= remote.timestamp {
                    ResolutionResult::KeepLocal
                } else {
                    ResolutionResult::KeepRemote
                }
            }
        }
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Conflict record for manual resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRecord {
    pub id: Id,
    pub entity_type: String,
    pub entity_id: Id,
    pub local_data: serde_json::Value,
    pub remote_data: serde_json::Value,
    pub local_timestamp: chrono::DateTime<chrono::Utc>,
    pub remote_timestamp: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub resolved: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_wins() {
        let resolver = ConflictResolver::new();
        
        let local = Change::delete("Patient".into(), Id::new_v4(), "device1".into());
        let remote = Change::update("Patient".into(), Id::new_v4(), serde_json::json!({"name": "test"}), "device2".into());
        
        match resolver.resolve(&local, &remote) {
            ResolutionResult::KeepLocal => (),
            _ => panic!("Delete should win"),
        }
    }

    #[test]
    fn test_merge_non_overlapping() {
        let resolver = ConflictResolver::new();
        let entity_id = Id::new_v4();
        
        let local = Change::update("Patient".into(), entity_id, serde_json::json!({"name": "John"}), "device1".into());
        std::thread::sleep(std::time::Duration::from_millis(10));
        let remote = Change::update("Patient".into(), entity_id, serde_json::json!({"phone": "555-1234"}), "device2".into());
        
        match resolver.resolve(&local, &remote) {
            ResolutionResult::Merge(merged) => {
                let obj = merged.data.as_object().unwrap();
                assert!(obj.contains_key("name"));
                assert!(obj.contains_key("phone"));
            }
            _ => panic!("Should merge non-overlapping fields"),
        }
    }
}
