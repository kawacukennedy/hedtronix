//! Device model for multi-device support

use serde::{Deserialize, Serialize};

use crate::types::{DeviceType, Id, Timestamp};

/// Device entity for multi-device support
/// CRDT Type: MV_REGISTER
/// Conflict Resolution: Admin-declared truth for revocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: Id,
    pub user_id: Id,
    
    /// Device public key for authentication
    pub public_key: String,
    
    pub device_type: DeviceType,
    
    /// User-friendly device name
    pub device_name: Option<String>,
    
    pub last_sync_at: Option<Timestamp>,
    pub ip_address: Option<String>,
    pub user_agent: String,
    
    /// Whether device has been revoked
    pub revoked: bool,
    pub revoked_at: Option<Timestamp>,
    pub revoked_by: Option<Id>,
    
    pub created_at: Timestamp,
}

impl Device {
    pub fn new(
        user_id: Id,
        public_key: String,
        device_type: DeviceType,
        user_agent: String,
    ) -> Self {
        Self {
            id: Id::new_v4(),
            user_id,
            public_key,
            device_type,
            device_name: None,
            last_sync_at: None,
            ip_address: None,
            user_agent,
            revoked: false,
            revoked_at: None,
            revoked_by: None,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.revoked
    }

    pub fn revoke(&mut self, revoked_by: Id) {
        self.revoked = true;
        self.revoked_at = Some(chrono::Utc::now());
        self.revoked_by = Some(revoked_by);
    }
}

/// DTO for device registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDevice {
    pub public_key: String,
    pub device_type: DeviceType,
    pub device_name: Option<String>,
    pub user_agent: String,
}
