//! Room model for resource scheduling

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::types::{Id, Timestamp};

/// Room entity for appointment scheduling
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Room {
    pub id: Id,
    
    #[validate(length(min = 1, max = 50))]
    pub name: String,
    
    #[validate(length(max = 20))]
    pub room_number: String,
    
    pub department_id: Option<Id>,
    
    pub room_type: RoomType,
    
    /// Room capacity
    pub capacity: i32,
    
    /// Equipment available in the room
    pub equipment: Vec<String>,
    
    pub active: bool,
    
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Room types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RoomType {
    ExamRoom,
    OperatingRoom,
    ConsultationRoom,
    LabRoom,
    ImagingRoom,
    WaitingRoom,
    RecoveryRoom,
    Other,
}

impl Room {
    pub fn new(name: String, room_number: String, room_type: RoomType) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Id::new_v4(),
            name,
            room_number,
            department_id: None,
            room_type,
            capacity: 1,
            equipment: Vec::new(),
            active: true,
            created_at: now,
            updated_at: now,
        }
    }
}
