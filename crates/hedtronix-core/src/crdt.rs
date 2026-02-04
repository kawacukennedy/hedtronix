//! CRDT (Conflict-free Replicated Data Types) implementation
//!
//! Provides offline-first data synchronization with automatic conflict resolution

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{Id, Timestamp, VersionVector};

/// Last-Write-Wins Register for scalar values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LwwRegister<T> {
    pub value: T,
    pub timestamp: Timestamp,
    pub device_id: String,
}

impl<T: Clone> LwwRegister<T> {
    pub fn new(value: T, device_id: String) -> Self {
        Self {
            value,
            timestamp: chrono::Utc::now(),
            device_id,
        }
    }

    pub fn update(&mut self, value: T, device_id: String) {
        self.value = value;
        self.timestamp = chrono::Utc::now();
        self.device_id = device_id;
    }

    /// Merge with another register, keeping the most recent value
    pub fn merge(&mut self, other: &LwwRegister<T>) {
        if other.timestamp > self.timestamp {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
            self.device_id = other.device_id.clone();
        } else if other.timestamp == self.timestamp && other.device_id > self.device_id {
            // Tie-breaker by device ID
            self.value = other.value.clone();
            self.device_id = other.device_id.clone();
        }
    }
}

/// Multi-Value Register for fields that may have concurrent updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MvRegister<T> {
    pub values: Vec<(T, Timestamp, String)>, // (value, timestamp, device_id)
}

impl<T: Clone + PartialEq> MvRegister<T> {
    pub fn new(value: T, device_id: String) -> Self {
        Self {
            values: vec![(value, chrono::Utc::now(), device_id)],
        }
    }

    pub fn set(&mut self, value: T, device_id: String) {
        self.values = vec![(value, chrono::Utc::now(), device_id)];
    }

    /// Get the current value (most recent if multiple)
    pub fn get(&self) -> Option<&T> {
        self.values.iter()
            .max_by_key(|(_, ts, _)| ts)
            .map(|(v, _, _)| v)
    }

    /// Check if there are conflicts
    pub fn has_conflict(&self) -> bool {
        self.values.len() > 1
    }

    /// Merge with another register
    pub fn merge(&mut self, other: &MvRegister<T>) {
        for (value, ts, device_id) in &other.values {
            if !self.values.iter().any(|(v, _, _)| v == value) {
                self.values.push((value.clone(), *ts, device_id.clone()));
            }
        }
        // Sort by timestamp and keep unique values
        self.values.sort_by(|(_, ts1, _), (_, ts2, _)| ts2.cmp(ts1));
    }

    /// Resolve conflict by choosing a specific value
    pub fn resolve(&mut self, chosen_value: T, device_id: String) {
        self.values = vec![(chosen_value, chrono::Utc::now(), device_id)];
    }
}

/// CRDT List element with unique ID for ordering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtListElement<T> {
    pub id: Id,
    pub value: T,
    pub deleted: bool,
    pub timestamp: Timestamp,
    pub device_id: String,
}

/// CRDT List for ordered collections (allergies, medications, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtList<T> {
    pub elements: Vec<CrdtListElement<T>>,
}

impl<T: Clone> CrdtList<T> {
    pub fn new() -> Self {
        Self { elements: Vec::new() }
    }

    pub fn add(&mut self, value: T, device_id: String) -> Id {
        let id = Id::new_v4();
        self.elements.push(CrdtListElement {
            id,
            value,
            deleted: false,
            timestamp: chrono::Utc::now(),
            device_id,
        });
        id
    }

    pub fn remove(&mut self, id: Id, device_id: String) {
        if let Some(elem) = self.elements.iter_mut().find(|e| e.id == id) {
            elem.deleted = true;
            elem.timestamp = chrono::Utc::now();
            elem.device_id = device_id;
        }
    }

    pub fn get(&self, id: Id) -> Option<&T> {
        self.elements.iter()
            .find(|e| e.id == id && !e.deleted)
            .map(|e| &e.value)
    }

    /// Get all active (non-deleted) values
    pub fn values(&self) -> Vec<&T> {
        self.elements.iter()
            .filter(|e| !e.deleted)
            .map(|e| &e.value)
            .collect()
    }

    /// Get all active elements
    pub fn active_elements(&self) -> Vec<&CrdtListElement<T>> {
        self.elements.iter().filter(|e| !e.deleted).collect()
    }

    /// Merge with another list
    pub fn merge(&mut self, other: &CrdtList<T>) {
        for other_elem in &other.elements {
            if let Some(elem) = self.elements.iter_mut().find(|e| e.id == other_elem.id) {
                // Element exists, merge based on timestamp
                if other_elem.timestamp > elem.timestamp {
                    elem.value = other_elem.value.clone();
                    elem.deleted = other_elem.deleted;
                    elem.timestamp = other_elem.timestamp;
                    elem.device_id = other_elem.device_id.clone();
                }
            } else {
                // New element, add it
                self.elements.push(other_elem.clone());
            }
        }
    }

    pub fn len(&self) -> usize {
        self.elements.iter().filter(|e| !e.deleted).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: Clone> Default for CrdtList<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// LWW Map for key-value pairs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LwwMap<K, V> {
    pub entries: HashMap<K, LwwRegister<Option<V>>>,
}

impl<K: Clone + std::hash::Hash + Eq, V: Clone> LwwMap<K, V> {
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    pub fn set(&mut self, key: K, value: V, device_id: String) {
        let register = LwwRegister::new(Some(value), device_id);
        self.entries.insert(key, register);
    }

    pub fn remove(&mut self, key: &K, device_id: String) {
        if let Some(register) = self.entries.get_mut(key) {
            register.update(None, device_id);
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key).and_then(|r| r.value.as_ref())
    }

    pub fn merge(&mut self, other: &LwwMap<K, V>) {
        for (key, other_register) in &other.entries {
            if let Some(register) = self.entries.get_mut(key) {
                register.merge(other_register);
            } else {
                self.entries.insert(key.clone(), other_register.clone());
            }
        }
    }
}

impl<K: Clone + std::hash::Hash + Eq, V: Clone> Default for LwwMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

/// Change record for sync operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub id: Id,
    pub entity_type: String,
    pub entity_id: Id,
    pub operation: ChangeOperation,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
    pub device_id: String,
    pub version: VersionVector,
}

/// Types of changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChangeOperation {
    Create,
    Update,
    Delete,
}

impl Change {
    pub fn create(
        entity_type: String,
        entity_id: Id,
        data: serde_json::Value,
        device_id: String,
    ) -> Self {
        Self {
            id: Id::new_v4(),
            entity_type,
            entity_id,
            operation: ChangeOperation::Create,
            data,
            timestamp: chrono::Utc::now(),
            device_id: device_id.clone(),
            version: {
                let mut v = VersionVector::new();
                v.increment(&device_id);
                v
            },
        }
    }

    pub fn update(
        entity_type: String,
        entity_id: Id,
        data: serde_json::Value,
        device_id: String,
    ) -> Self {
        Self {
            id: Id::new_v4(),
            entity_type,
            entity_id,
            operation: ChangeOperation::Update,
            data,
            timestamp: chrono::Utc::now(),
            device_id: device_id.clone(),
            version: {
                let mut v = VersionVector::new();
                v.increment(&device_id);
                v
            },
        }
    }

    pub fn delete(
        entity_type: String,
        entity_id: Id,
        device_id: String,
    ) -> Self {
        Self {
            id: Id::new_v4(),
            entity_type,
            entity_id,
            operation: ChangeOperation::Delete,
            data: serde_json::json!({}),
            timestamp: chrono::Utc::now(),
            device_id: device_id.clone(),
            version: {
                let mut v = VersionVector::new();
                v.increment(&device_id);
                v
            },
        }
    }
}

/// Conflict information for manual resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub id: Id,
    pub entity_type: String,
    pub entity_id: Id,
    pub local_change: Change,
    pub remote_change: Change,
    pub resolved: bool,
    pub resolution: Option<ConflictResolution>,
    pub created_at: Timestamp,
}

/// How a conflict was resolved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub strategy: ResolutionStrategy,
    pub resolved_by: Option<Id>,
    pub resolved_at: Timestamp,
    pub final_data: serde_json::Value,
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResolutionStrategy {
    KeepLocal,
    KeepRemote,
    Merge,
    Manual,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lww_register_merge() {
        let mut reg1 = LwwRegister::new("value1".to_string(), "device1".to_string());
        std::thread::sleep(std::time::Duration::from_millis(10));
        let reg2 = LwwRegister::new("value2".to_string(), "device2".to_string());
        
        reg1.merge(&reg2);
        assert_eq!(reg1.value, "value2");
    }

    #[test]
    fn test_crdt_list_merge() {
        let mut list1: CrdtList<String> = CrdtList::new();
        let id1 = list1.add("item1".to_string(), "device1".to_string());
        
        let mut list2: CrdtList<String> = CrdtList::new();
        list2.add("item2".to_string(), "device2".to_string());
        
        list1.merge(&list2);
        assert_eq!(list1.len(), 2);
    }
}
