//! Last-Write-Wins Register CRDT implementation

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Last-Write-Wins Register for scalar values
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LWWRegister<T> {
    pub value: T,
    pub timestamp: DateTime<Utc>,
    pub device_id: Uuid,
}

impl<T: Clone> LWWRegister<T> {
    /// Create a new LWW register
    pub fn new(value: T, device_id: Uuid) -> Self {
        Self {
            value,
            timestamp: Utc::now(),
            device_id,
        }
    }

    /// Create with specific timestamp (for deserialization)
    pub fn with_timestamp(value: T, timestamp: DateTime<Utc>, device_id: Uuid) -> Self {
        Self {
            value,
            timestamp,
            device_id,
        }
    }

    /// Update the value
    pub fn set(&mut self, value: T, device_id: Uuid) {
        self.value = value;
        self.timestamp = Utc::now();
        self.device_id = device_id;
    }

    /// Merge with another register (last-write-wins)
    pub fn merge(&mut self, other: &LWWRegister<T>) {
        if other.timestamp > self.timestamp {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
            self.device_id = other.device_id;
        } else if other.timestamp == self.timestamp {
            // Tie-break using device_id for deterministic resolution
            if other.device_id > self.device_id {
                self.value = other.value.clone();
                self.device_id = other.device_id;
            }
        }
    }

    /// Get the current value
    pub fn get(&self) -> &T {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lww_register_creation() {
        let device_id = Uuid::new_v4();
        let reg = LWWRegister::new("initial", device_id);
        assert_eq!(reg.get(), &"initial");
    }

    #[test]
    fn test_lww_register_update() {
        let device_id = Uuid::new_v4();
        let mut reg = LWWRegister::new("initial", device_id);
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        reg.set("updated", device_id);
        
        assert_eq!(reg.get(), &"updated");
    }

    #[test]
    fn test_lww_register_merge() {
        let device1 = Uuid::new_v4();
        let device2 = Uuid::new_v4();
        
        let mut reg1 = LWWRegister::new("value1", device1);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let reg2 = LWWRegister::new("value2", device2);
        
        reg1.merge(&reg2);
        assert_eq!(reg1.get(), &"value2");
    }

    #[test]
    fn test_lww_register_merge_same_timestamp() {
        let device1 = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let device2 = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
        
        let timestamp = Utc::now();
        let mut reg1 = LWWRegister::with_timestamp("value1", timestamp, device1);
        let reg2 = LWWRegister::with_timestamp("value2", timestamp, device2);
        
        reg1.merge(&reg2);
        // Device2 has higher UUID, so it wins
        assert_eq!(reg1.get(), &"value2");
    }
}
