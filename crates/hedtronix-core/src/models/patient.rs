//! Patient model with CRDT support

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::types::{
    Address, Allergy, EmergencyContact, Gender, Id, InsuranceInfo, Medication, Timestamp,
    VersionVector,
};

/// Patient entity with comprehensive medical record support
/// CRDT Type: Composite (LWW_REGISTER for scalars, CRDT_LIST for collections)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Patient {
    pub id: Id,
    
    /// Medical Record Number (unique per system)
    #[validate(length(min = 1, max = 50))]
    pub medical_record_number: String,
    
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
    
    pub date_of_birth: NaiveDate,
    
    pub gender: Gender,
    
    pub address: Address,
    
    #[validate(length(max = 20))]
    pub phone: String,
    
    #[validate(email)]
    pub email: Option<String>,
    
    pub emergency_contact: EmergencyContact,
    
    /// Primary care physician
    pub primary_care_physician_id: Option<Id>,
    
    pub insurance_info: InsuranceInfo,
    
    /// Allergies - CRDT_LIST type for conflict-free merging
    pub allergies: Vec<Allergy>,
    
    /// Current medications - CRDT_LIST type
    pub medications: Vec<Medication>,
    
    /// Problem list / diagnoses
    pub problems: Vec<String>,
    
    pub active: bool,
    pub deceased: bool,
    pub deceased_at: Option<Timestamp>,
    
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    
    /// CRDT version tracking
    pub version: VersionVector,
    
    /// Last modification device ID
    pub last_modified_by: Option<String>,
}

impl Patient {
    pub fn new(
        medical_record_number: String,
        first_name: String,
        last_name: String,
        date_of_birth: NaiveDate,
        gender: Gender,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Id::new_v4(),
            medical_record_number,
            first_name,
            last_name,
            date_of_birth,
            gender,
            address: Address::default(),
            phone: String::new(),
            email: None,
            emergency_contact: EmergencyContact::default(),
            primary_care_physician_id: None,
            insurance_info: InsuranceInfo::default(),
            allergies: Vec::new(),
            medications: Vec::new(),
            problems: Vec::new(),
            active: true,
            deceased: false,
            deceased_at: None,
            created_at: now,
            updated_at: now,
            version: VersionVector::new(),
            last_modified_by: None,
        }
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn age(&self) -> i32 {
        let today = chrono::Utc::now().date_naive();
        let years = today.years_since(self.date_of_birth).unwrap_or(0) as i32;
        years
    }

    pub fn add_allergy(&mut self, allergy: Allergy) {
        // Check for duplicates by name
        if !self.allergies.iter().any(|a| a.name == allergy.name) {
            self.allergies.push(allergy);
            self.updated_at = chrono::Utc::now();
        }
    }

    pub fn remove_allergy(&mut self, allergy_id: Id) {
        self.allergies.retain(|a| a.id != allergy_id);
        self.updated_at = chrono::Utc::now();
    }

    pub fn add_medication(&mut self, medication: Medication) {
        self.medications.push(medication);
        self.updated_at = chrono::Utc::now();
    }

    pub fn has_allergy(&self, name: &str) -> bool {
        self.allergies.iter().any(|a| a.name.to_lowercase() == name.to_lowercase())
    }
}

/// Patient creation DTO
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePatient {
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
    
    pub date_of_birth: NaiveDate,
    pub gender: Gender,
    
    pub phone: Option<String>,
    
    #[validate(email)]
    pub email: Option<String>,
    
    pub address: Option<Address>,
    pub emergency_contact: Option<EmergencyContact>,
    pub insurance_info: Option<InsuranceInfo>,
}

/// Patient update DTO
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdatePatient {
    #[validate(length(min = 1, max = 100))]
    pub first_name: Option<String>,
    
    #[validate(length(min = 1, max = 100))]
    pub last_name: Option<String>,
    
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<Gender>,
    pub phone: Option<String>,
    
    #[validate(email)]
    pub email: Option<String>,
    
    pub address: Option<Address>,
    pub emergency_contact: Option<EmergencyContact>,
    pub insurance_info: Option<InsuranceInfo>,
    pub primary_care_physician_id: Option<Id>,
    pub active: Option<bool>,
}

/// Patient search filters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatientSearchFilters {
    pub query: Option<String>,
    pub active_only: bool,
    pub physician_id: Option<Id>,
    pub department_id: Option<Id>,
    pub page: u32,
    pub limit: u32,
}
