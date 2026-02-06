//! Multi-Value Register CRDT implementation

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashSet;

/// Multi-Value Register for concurrent updates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MVRegister<T: Clone + Eq + std::hash::Hash> {
    pub values: HashSet<VersionedValue<T>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VersionedValue<T> {
    pub value: T,
    pub timestamp: DateTime<Utc>,
    pub device_id: Uuid,
}

impl<T: Clone + Eq + std::hash::Hash> MVRegister<T> {
    /// Create a new MV register
    pub fn new(value: T, device_id: Uuid) -> Self {
        let mut values = HashSet::new();
        values.insert(VersionedValue {
            value,
            timestamp: Utc::now(),
            device_id,
        });
        Self { values }
    }

    /// Create an empty register
    pub fn empty() -> Self {
        Self {
            values: HashSet::new(),
        }
    }

    /// Update the value
    pub fn set(&mut self, value: T, device_id: Uuid) {
        self.values.clear();
        self.values.insert(VersionedValue {
            value,
            timestamp: Utc::now(),
            device_id,
        });
    }

    /// Merge with another register
    pub fn merge(&mut self, other: &MVRegister<T>) {
        // Find the maximum timestamp
        let max_timestamp = self
            .values
            .iter()
            .chain(other.values.iter())
            .map(|v| v.timestamp)
            .max();

        if let Some(max_ts) = max_timestamp {
            // Keep only values with the maximum timestamp
            let mut new_values = HashSet::new();
            
            for v in self.values.iter().chain(other.values.iter()) {
                if v.timestamp == max_ts {
                    new_values.insert(v.clone());
                }
            }
            
            self.values = new_values;
        }
    }

    /// Get all concurrent values
    pub fn get_all(&self) -> Vec<&T> {
        self.values.iter().map(|v| &v.value).collect()
    }

    /// Check if there's a conflict (multiple values)
    pub fn has_conflict(&self) -> bool {
        self.values.len() > 1
    }

    /// Get a single value (returns None if there's a conflict)
    pub fn get_single(&self) -> Option<&T> {
        if self.values.len() == 1 {
            self.values.iter().next().map(|v| &v.value)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mv_register_creation() {
        let device_id = Uuid::new_v4();
        let reg = MVRegister::new("initial", device_id);
        assert_eq!(reg.get_single(), Some(&"initial"));
        assert!(!reg.has_conflict());
    }

    #[test]
    fn test_mv_register_update() {
        let device_id = Uuid::new_v4();
        let mut reg = MVRegister::new("initial", device_id);
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        reg.set("updated", device_id);
        
        assert_eq!(reg.get_single(), Some(&"updated"));
        assert!(!reg.has_conflict());
    }

    #[test]
    fn test_mv_register_concurrent_updates() {
        let device1 = Uuid::new_v4();
        let device2 = Uuid::new_v4();
        
        let timestamp = Utc::now();
        
        let mut reg1 = MVRegister::empty();
        reg1.values.insert(VersionedValue {
            value: "value1",
            timestamp,
            device_id: device1,
        });
        
        let mut reg2 = MVRegister::empty();
        reg2.values.insert(VersionedValue {
            value: "value2",
            timestamp,
            device_id: device2,
        });
        
        reg1.merge(&reg2);
        
        assert!(reg1.has_conflict());
        assert_eq!(reg1.get_all().len(), 2);
    }

    #[test]
    fn test_mv_register_merge_no_conflict() {
        let device1 = Uuid::new_v4();
        let device2 = Uuid::new_v4();
        
        let mut reg1 = MVRegister::new("value1", device1);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let reg2 = MVRegister::new("value2", device2);
        
        reg1.merge(&reg2);
        
        assert!(!reg1.has_conflict());
        assert_eq!(reg1.get_single(), Some(&"value2"));
    }
}
