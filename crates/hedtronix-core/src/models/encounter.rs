//! Encounter model - a patient visit/interaction

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::types::{Id, Timestamp, VersionVector};

/// Encounter entity representing a patient visit
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Encounter {
    pub id: Id,
    pub patient_id: Id,
    pub provider_id: Id,
    pub appointment_id: Option<Id>,
    pub department_id: Option<Id>,
    
    pub encounter_type: EncounterType,
    pub status: EncounterStatus,
    
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    
    #[validate(length(max = 500))]
    pub chief_complaint: Option<String>,
    
    /// Associated clinical note IDs
    pub clinical_note_ids: Vec<Id>,
    
    /// Associated billing entry IDs
    pub billing_entry_ids: Vec<Id>,
    
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    
    /// CRDT version tracking
    pub version: VersionVector,
}

/// Encounter types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EncounterType {
    Office,
    Inpatient,
    Emergency,
    Telehealth,
    HomeVisit,
}

/// Encounter status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EncounterStatus {
    InProgress,
    Completed,
    Cancelled,
}

impl Encounter {
    pub fn new(
        patient_id: Id,
        provider_id: Id,
        encounter_type: EncounterType,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Id::new_v4(),
            patient_id,
            provider_id,
            appointment_id: None,
            department_id: None,
            encounter_type,
            status: EncounterStatus::InProgress,
            start_time: now,
            end_time: None,
            chief_complaint: None,
            clinical_note_ids: Vec::new(),
            billing_entry_ids: Vec::new(),
            created_at: now,
            updated_at: now,
            version: VersionVector::new(),
        }
    }

    pub fn complete(&mut self) {
        self.status = EncounterStatus::Completed;
        self.end_time = Some(chrono::Utc::now());
        self.updated_at = chrono::Utc::now();
    }

    pub fn add_clinical_note(&mut self, note_id: Id) {
        if !self.clinical_note_ids.contains(&note_id) {
            self.clinical_note_ids.push(note_id);
            self.updated_at = chrono::Utc::now();
        }
    }

    pub fn add_billing_entry(&mut self, entry_id: Id) {
        if !self.billing_entry_ids.contains(&entry_id) {
            self.billing_entry_ids.push(entry_id);
            self.updated_at = chrono::Utc::now();
        }
    }
}
