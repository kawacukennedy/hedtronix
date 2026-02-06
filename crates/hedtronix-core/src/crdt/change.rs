//! Change tracking for sync operations

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::types::VersionVector;

/// Operation type for changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChangeOperation {
    Create,
    Update,
    Delete,
}

/// A tracked change for sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub operation: ChangeOperation,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub device_id: String,
    pub version: VersionVector,
}

impl Change {
    /// Create a new change
    pub fn new(
        entity_type: impl Into<String>,
        entity_id: Uuid,
        operation: ChangeOperation,
        data: serde_json::Value,
        device_id: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type: entity_type.into(),
            entity_id,
            operation,
            data,
            timestamp: Utc::now(),
            device_id: device_id.into(),
            version: VersionVector::new(),
        }
    }

    /// Create for an entity creation
    pub fn create(entity_type: impl Into<String>, entity_id: Uuid, data: serde_json::Value, device_id: impl Into<String>) -> Self {
        Self::new(entity_type, entity_id, ChangeOperation::Create, data, device_id)
    }

    /// Create for an entity update
    pub fn update(entity_type: impl Into<String>, entity_id: Uuid, data: serde_json::Value, device_id: impl Into<String>) -> Self {
        Self::new(entity_type, entity_id, ChangeOperation::Update, data, device_id)
    }

    /// Create for an entity deletion
    pub fn delete(entity_type: impl Into<String>, entity_id: Uuid, device_id: impl Into<String>) -> Self {
        Self::new(entity_type, entity_id, ChangeOperation::Delete, serde_json::Value::Null, device_id)
    }
}
