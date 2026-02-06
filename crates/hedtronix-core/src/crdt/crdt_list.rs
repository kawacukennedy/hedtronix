//! CRDT List implementation for collections (allergies, medications)

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

/// CRDT List element
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ListElement<T> {
    pub id: Uuid,
    pub value: T,
    pub timestamp: DateTime<Utc>,
    pub device_id: Uuid,
    pub deleted: bool,
}

/// CRDT List for managing collections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRDTList<T: Clone> {
    pub elements: HashMap<Uuid, ListElement<T>>,
}

impl<T: Clone> CRDTList<T> {
    /// Create a new empty CRDT list
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
        }
    }

    /// Add an element to the list
    pub fn add(&mut self, value: T, device_id: Uuid) -> Uuid {
        let id = Uuid::new_v4();
        let element = ListElement {
            id,
            value,
            timestamp: Utc::now(),
            device_id,
            deleted: false,
        };
        self.elements.insert(id, element);
        id
    }

    /// Remove an element (soft delete)
    pub fn remove(&mut self, id: &Uuid, device_id: Uuid) -> bool {
        if let Some(element) = self.elements.get_mut(id) {
            if !element.deleted {
                element.deleted = true;
                element.timestamp = Utc::now();
                element.device_id = device_id;
                return true;
            }
        }
        false
    }

    /// Update an element
    pub fn update(&mut self, id: &Uuid, value: T, device_id: Uuid) -> bool {
        if let Some(element) = self.elements.get_mut(id) {
            if !element.deleted {
                element.value = value;
                element.timestamp = Utc::now();
                element.device_id = device_id;
                return true;
            }
        }
        false
    }

    /// Merge with another CRDT list
    pub fn merge(&mut self, other: &CRDTList<T>) {
        for (id, other_element) in &other.elements {
            match self.elements.get_mut(id) {
                Some(self_element) => {
                    // Element exists in both - keep the one with later timestamp
                    if other_element.timestamp > self_element.timestamp {
                        *self_element = other_element.clone();
                    } else if other_element.timestamp == self_element.timestamp {
                        // Tie-break using device_id
                        if other_element.device_id > self_element.device_id {
                            *self_element = other_element.clone();
                        }
                    }
                }
                None => {
                    // Element only exists in other - add it
                    self.elements.insert(*id, other_element.clone());
                }
            }
        }
    }

    /// Get all active (non-deleted) elements
    pub fn get_active(&self) -> Vec<&ListElement<T>> {
        self.elements
            .values()
            .filter(|e| !e.deleted)
            .collect()
    }

    /// Get all elements (including deleted)
    pub fn get_all(&self) -> Vec<&ListElement<T>> {
        self.elements.values().collect()
    }

    /// Get element by ID
    pub fn get(&self, id: &Uuid) -> Option<&ListElement<T>> {
        self.elements.get(id)
    }

    /// Count active elements
    pub fn len(&self) -> usize {
        self.elements.values().filter(|e| !e.deleted).count()
    }

    /// Check if list is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: Clone> Default for CRDTList<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crdt_list_add() {
        let device_id = Uuid::new_v4();
        let mut list = CRDTList::new();
        
        let id = list.add("item1", device_id);
        assert_eq!(list.len(), 1);
        assert_eq!(list.get(&id).unwrap().value, "item1");
    }

    #[test]
    fn test_crdt_list_remove() {
        let device_id = Uuid::new_v4();
        let mut list = CRDTList::new();
        
        let id = list.add("item1", device_id);
        assert_eq!(list.len(), 1);
        
        list.remove(&id, device_id);
        assert_eq!(list.len(), 0);
        assert!(list.get(&id).unwrap().deleted);
    }

    #[test]
    fn test_crdt_list_update() {
        let device_id = Uuid::new_v4();
        let mut list = CRDTList::new();
        
        let id = list.add("item1", device_id);
        list.update(&id, "item1_updated", device_id);
        
        assert_eq!(list.get(&id).unwrap().value, "item1_updated");
    }

    #[test]
    fn test_crdt_list_merge() {
        let device1 = Uuid::new_v4();
        let device2 = Uuid::new_v4();
        
        let mut list1 = CRDTList::new();
        let id1 = list1.add("item1", device1);
        
        let mut list2 = CRDTList::new();
        let id2 = list2.add("item2", device2);
        
        list1.merge(&list2);
        
        assert_eq!(list1.len(), 2);
        assert!(list1.get(&id1).is_some());
        assert!(list1.get(&id2).is_some());
    }

    #[test]
    fn test_crdt_list_merge_conflict() {
        let device1 = Uuid::new_v4();
        let device2 = Uuid::new_v4();
        
        let shared_id = Uuid::new_v4();
        
        let mut list1 = CRDTList::new();
        list1.elements.insert(
            shared_id,
            ListElement {
                id: shared_id,
                value: "value1",
                timestamp: Utc::now(),
                device_id: device1,
                deleted: false,
            },
        );
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let mut list2 = CRDTList::new();
        list2.elements.insert(
            shared_id,
            ListElement {
                id: shared_id,
                value: "value2",
                timestamp: Utc::now(),
                device_id: device2,
                deleted: false,
            },
        );
        
        list1.merge(&list2);
        
        // list2's value should win because it has a later timestamp
        assert_eq!(list1.get(&shared_id).unwrap().value, "value2");
    }
}
