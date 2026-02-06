//! Version Vector for tracking causality in distributed systems

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Version Vector for tracking causality
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionVector {
    pub versions: HashMap<Uuid, u64>,
}

impl VersionVector {
    /// Create a new empty version vector
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
        }
    }

    /// Increment the version for a device
    pub fn increment(&mut self, device_id: Uuid) {
        let counter = self.versions.entry(device_id).or_insert(0);
        *counter += 1;
    }

    /// Get the version for a device
    pub fn get(&self, device_id: &Uuid) -> u64 {
        self.versions.get(device_id).copied().unwrap_or(0)
    }

    /// Merge with another version vector (take maximum for each device)
    pub fn merge(&mut self, other: &VersionVector) {
        for (device_id, &version) in &other.versions {
            let current = self.versions.entry(*device_id).or_insert(0);
            *current = (*current).max(version);
        }
    }

    /// Check if this version vector happens before another
    pub fn happens_before(&self, other: &VersionVector) -> bool {
        let mut strictly_less = false;
        
        // Check all devices in self
        for (device_id, &self_version) in &self.versions {
            let other_version = other.get(device_id);
            if self_version > other_version {
                return false;
            }
            if self_version < other_version {
                strictly_less = true;
            }
        }
        
        // Check devices only in other
        for (device_id, &other_version) in &other.versions {
            if !self.versions.contains_key(device_id) && other_version > 0 {
                strictly_less = true;
            }
        }
        
        strictly_less
    }

    /// Check if two version vectors are concurrent (neither happens before the other)
    pub fn is_concurrent(&self, other: &VersionVector) -> bool {
        !self.happens_before(other) && !other.happens_before(self) && self != other
    }

    /// Check if this version vector is equal to another
    pub fn equals(&self, other: &VersionVector) -> bool {
        self == other
    }
}

impl Default for VersionVector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_vector_increment() {
        let device_id = Uuid::new_v4();
        let mut vv = VersionVector::new();
        
        vv.increment(device_id);
        assert_eq!(vv.get(&device_id), 1);
        
        vv.increment(device_id);
        assert_eq!(vv.get(&device_id), 2);
    }

    #[test]
    fn test_version_vector_merge() {
        let device1 = Uuid::new_v4();
        let device2 = Uuid::new_v4();
        
        let mut vv1 = VersionVector::new();
        vv1.increment(device1);
        vv1.increment(device1);
        
        let mut vv2 = VersionVector::new();
        vv2.increment(device2);
        vv2.increment(device2);
        vv2.increment(device2);
        
        vv1.merge(&vv2);
        
        assert_eq!(vv1.get(&device1), 2);
        assert_eq!(vv1.get(&device2), 3);
    }

    #[test]
    fn test_version_vector_happens_before() {
        let device1 = Uuid::new_v4();
        let device2 = Uuid::new_v4();
        
        let mut vv1 = VersionVector::new();
        vv1.increment(device1);
        
        let mut vv2 = VersionVector::new();
        vv2.increment(device1);
        vv2.increment(device1);
        vv2.increment(device2);
        
        assert!(vv1.happens_before(&vv2));
        assert!(!vv2.happens_before(&vv1));
    }

    #[test]
    fn test_version_vector_concurrent() {
        let device1 = Uuid::new_v4();
        let device2 = Uuid::new_v4();
        
        let mut vv1 = VersionVector::new();
        vv1.increment(device1);
        vv1.increment(device1);
        
        let mut vv2 = VersionVector::new();
        vv2.increment(device2);
        
        assert!(vv1.is_concurrent(&vv2));
        assert!(vv2.is_concurrent(&vv1));
    }
}
