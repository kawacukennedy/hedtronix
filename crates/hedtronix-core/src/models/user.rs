//! User model

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::types::{Id, Timestamp, UserRole, VersionVector};

/// User entity representing system users (physicians, nurses, admin, etc.)
/// CRDT Type: LWW_REGISTER
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct User {
    pub id: Id,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    
    pub role: UserRole,
    
    pub department_id: Option<Id>,
    
    /// License number (for clinical staff)
    pub license_number: Option<String>,
    
    /// NPI number (National Provider Identifier)
    pub npi_number: Option<String>,
    
    pub active: bool,
    
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub last_login_at: Option<Timestamp>,
    
    /// Password hash (Argon2)
    #[serde(skip_serializing)]
    pub password_hash: String,
    
    /// CRDT version tracking
    pub version: VersionVector,
    
    /// Last modification device ID
    pub last_modified_by: Option<String>,
}

impl User {
    pub fn new(email: String, name: String, role: UserRole, password_hash: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Id::new_v4(),
            email,
            name,
            role,
            department_id: None,
            license_number: None,
            npi_number: None,
            active: true,
            created_at: now,
            updated_at: now,
            last_login_at: None,
            password_hash,
            version: VersionVector::new(),
            last_modified_by: None,
        }
    }

    /// Check if user has permission for an action
    pub fn has_permission(&self, resource: &str, action: &str) -> bool {
        match self.role {
            UserRole::Admin => true, // Admin has all permissions
            UserRole::Physician => {
                matches!(
                    (resource, action),
                    ("patients", "read" | "write" | "create")
                        | ("appointments", "read" | "write" | "create")
                        | ("clinical_notes", "read" | "write" | "create" | "sign")
                        | ("prescriptions", "read" | "write" | "create" | "sign")
                        | ("billing", "read")
                        | ("reports", "read")
                )
            }
            UserRole::Nurse => {
                matches!(
                    (resource, action),
                    ("patients", "read" | "write")
                        | ("appointments", "read" | "write")
                        | ("clinical_notes", "read" | "write")
                        | ("vitals", "read" | "write" | "create")
                        | ("medication_administration", "read" | "write" | "create")
                        | ("billing", "read")
                )
            }
            UserRole::Receptionist => {
                matches!(
                    (resource, action),
                    ("patients", "read" | "write" | "create")
                        | ("appointments", "read" | "write" | "create" | "cancel")
                        | ("billing", "read" | "create_charges")
                        | ("clinical_notes", "read")
                )
            }
            UserRole::Billing => {
                matches!(
                    (resource, action),
                    ("patients", "read")
                        | ("appointments", "read")
                        | ("clinical_notes", "read")
                        | ("billing", "read" | "write" | "create" | "submit" | "adjust")
                        | ("reports", "read_financial")
                )
            }
            UserRole::Patient => {
                matches!(
                    (resource, action),
                    ("own_data", "read")
                        | ("appointments", "read_own" | "create_own" | "cancel_own")
                        | ("messages", "read" | "create")
                        | ("billing", "read_own" | "pay")
                )
            }
        }
    }
}

/// User creation DTO
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateUser {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    
    pub role: UserRole,
    
    #[validate(length(min = 8))]
    pub password: String,
    
    pub department_id: Option<Id>,
    pub license_number: Option<String>,
    pub npi_number: Option<String>,
}

/// User update DTO
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateUser {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    
    pub role: Option<UserRole>,
    pub department_id: Option<Id>,
    pub license_number: Option<String>,
    pub npi_number: Option<String>,
    pub active: Option<bool>,
}
