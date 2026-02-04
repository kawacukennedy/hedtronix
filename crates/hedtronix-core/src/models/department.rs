//! Department model

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::types::{Id, Timestamp};

/// Department entity
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Department {
    pub id: Id,
    
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    pub description: Option<String>,
    
    /// Parent department for hierarchy
    pub parent_id: Option<Id>,
    
    /// Department manager
    pub manager_id: Option<Id>,
    
    pub active: bool,
    
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Department {
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Id::new_v4(),
            name,
            description: None,
            parent_id: None,
            manager_id: None,
            active: true,
            created_at: now,
            updated_at: now,
        }
    }
}
