//! Audit Log model - immutable event log

use serde::{Deserialize, Serialize};

use crate::types::{AuditEventType, Id, Timestamp};

/// Audit Log entry - immutable record of all system events
/// CRDT Type: APPEND_ONLY_LOG
/// Conflict Resolution: Immutable, ordered by timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Id,
    pub event_type: AuditEventType,
    
    /// User who performed the action (None for system events)
    pub user_id: Option<Id>,
    
    /// Device that generated the event
    pub device_id: Option<Id>,
    
    /// Type of entity affected (e.g., "Patient", "Appointment")
    pub entity_type: String,
    
    /// ID of the affected entity
    pub entity_id: String,
    
    /// JSON representation of changes
    pub changes: serde_json::Value,
    
    /// IP address of the request
    pub ip_address: Option<String>,
    
    /// User agent string
    pub user_agent: Option<String>,
    
    /// Timestamp of the event
    pub timestamp: Timestamp,
    
    /// Cryptographic signature for non-repudiation
    pub signature: String,
    
    /// Hash of previous log entry for chain integrity
    pub previous_hash: Option<String>,
    
    /// Hash of this log entry
    pub hash: String,
}

impl AuditLog {
    pub fn new(
        event_type: AuditEventType,
        user_id: Option<Id>,
        device_id: Option<Id>,
        entity_type: String,
        entity_id: String,
        changes: serde_json::Value,
    ) -> Self {
        let now = chrono::Utc::now();
        let id = Id::new_v4();
        
        // Create a simple hash (in production, use proper cryptographic hashing)
        let hash_input = format!("{}{:?}{}{}", id, event_type, entity_type, now);
        let hash = format!("{:x}", md5_hash(&hash_input));
        
        Self {
            id,
            event_type,
            user_id,
            device_id,
            entity_type,
            entity_id,
            changes,
            ip_address: None,
            user_agent: None,
            timestamp: now,
            signature: String::new(), // Will be set by signing service
            previous_hash: None,
            hash,
        }
    }

    /// Create a read event audit log
    pub fn read_event(
        user_id: Id,
        device_id: Id,
        entity_type: &str,
        entity_id: &str,
    ) -> Self {
        Self::new(
            AuditEventType::Read,
            Some(user_id),
            Some(device_id),
            entity_type.to_string(),
            entity_id.to_string(),
            serde_json::json!({}),
        )
    }

    /// Create a create event audit log
    pub fn create_event(
        user_id: Id,
        device_id: Id,
        entity_type: &str,
        entity_id: &str,
        data: serde_json::Value,
    ) -> Self {
        Self::new(
            AuditEventType::Create,
            Some(user_id),
            Some(device_id),
            entity_type.to_string(),
            entity_id.to_string(),
            data,
        )
    }

    /// Create an update event audit log
    pub fn update_event(
        user_id: Id,
        device_id: Id,
        entity_type: &str,
        entity_id: &str,
        before: serde_json::Value,
        after: serde_json::Value,
    ) -> Self {
        Self::new(
            AuditEventType::Update,
            Some(user_id),
            Some(device_id),
            entity_type.to_string(),
            entity_id.to_string(),
            serde_json::json!({
                "before": before,
                "after": after
            }),
        )
    }

    /// Create a delete event audit log
    pub fn delete_event(
        user_id: Id,
        device_id: Id,
        entity_type: &str,
        entity_id: &str,
        data: serde_json::Value,
    ) -> Self {
        Self::new(
            AuditEventType::Delete,
            Some(user_id),
            Some(device_id),
            entity_type.to_string(),
            entity_id.to_string(),
            data,
        )
    }

    /// Create a login event
    pub fn login_event(user_id: Id, device_id: Id) -> Self {
        Self::new(
            AuditEventType::Login,
            Some(user_id),
            Some(device_id),
            "Session".to_string(),
            device_id.to_string(),
            serde_json::json!({}),
        )
    }

    /// Create a logout event
    pub fn logout_event(user_id: Id, device_id: Id) -> Self {
        Self::new(
            AuditEventType::Logout,
            Some(user_id),
            Some(device_id),
            "Session".to_string(),
            device_id.to_string(),
            serde_json::json!({}),
        )
    }

    /// Create a sync event
    pub fn sync_event(
        user_id: Id,
        device_id: Id,
        changes_pushed: usize,
        changes_pulled: usize,
    ) -> Self {
        Self::new(
            AuditEventType::Sync,
            Some(user_id),
            Some(device_id),
            "Sync".to_string(),
            device_id.to_string(),
            serde_json::json!({
                "changes_pushed": changes_pushed,
                "changes_pulled": changes_pulled
            }),
        )
    }

    pub fn with_ip_address(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }

    pub fn with_user_agent(mut self, ua: String) -> Self {
        self.user_agent = Some(ua);
        self
    }

    pub fn with_previous_hash(mut self, hash: String) -> Self {
        self.previous_hash = Some(hash);
        self
    }
}

/// Simple MD5 hash function (for demo purposes - use proper crypto in production)
fn md5_hash(input: &str) -> u128 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish() as u128
}

/// Audit log query filters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditLogFilters {
    pub user_id: Option<Id>,
    pub device_id: Option<Id>,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub event_types: Option<Vec<AuditEventType>>,
    pub start_time: Option<Timestamp>,
    pub end_time: Option<Timestamp>,
    pub page: u32,
    pub limit: u32,
}
