//! Common types used throughout HEDTRONIX

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier type alias
pub type Id = Uuid;

/// Timestamp type alias
pub type Timestamp = DateTime<Utc>;

/// User roles as defined in specs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserRole {
    Physician,
    Nurse,
    Receptionist,
    Billing,
    Admin,
    Patient,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Physician => "PHYSICIAN",
            UserRole::Nurse => "NURSE",
            UserRole::Receptionist => "RECEPTIONIST",
            UserRole::Billing => "BILLING",
            UserRole::Admin => "ADMIN",
            UserRole::Patient => "PATIENT",
        }
    }
}

/// Device types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeviceType {
    Desktop,
    Tablet,
    Mobile,
    Kiosk,
}

/// Patient gender options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Gender {
    Male,
    Female,
    Other,
    Unknown,
}

impl Gender {
    pub fn as_str(&self) -> &'static str {
        match self {
            Gender::Male => "MALE",
            Gender::Female => "FEMALE",
            Gender::Other => "OTHER",
            Gender::Unknown => "UNKNOWN",
        }
    }
}


/// Clinical note types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NoteType {
    ProgressNote,
    Consultation,
    DischargeSummary,
    ProcedureNote,
}

/// Clinical note status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NoteStatus {
    Draft,
    Signed,
    Amended,
    Voided,
}

/// Appointment types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppointmentType {
    NewPatient,
    FollowUp,
    Procedure,
    Consultation,
    Emergency,
}

/// Appointment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppointmentStatus {
    Scheduled,
    CheckedIn,
    InRoom,
    Completed,
    Cancelled,
    NoShow,
}

/// Billing entry status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BillingStatus {
    Draft,
    Billed,
    Submitted,
    Paid,
    Denied,
    Appealed,
}

/// Audit log event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditEventType {
    Create,
    Read,
    Update,
    Delete,
    Login,
    Logout,
    Export,
    Sync,
}

/// Sync health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SyncHealth {
    Healthy,
    Warning,
    Error,
}

/// Address structure for patient and contact info
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Address {
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

/// Emergency contact information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmergencyContact {
    pub name: String,
    pub relationship: String,
    pub phone: String,
}

/// Insurance information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InsuranceInfo {
    pub provider: Option<String>,
    pub policy_number: Option<String>,
    pub group_number: Option<String>,
    pub subscriber_name: Option<String>,
    pub subscriber_dob: Option<String>,
}

/// Allergy entry for CRDT list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allergy {
    pub id: Id,
    pub name: String,
    pub severity: AllergySeverity,
    pub reaction: Option<String>,
    pub onset_date: Option<String>,
    pub created_at: Timestamp,
}

/// Allergy severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AllergySeverity {
    Mild,
    Moderate,
    Severe,
    LifeThreatening,
}

/// Medication entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medication {
    pub id: Id,
    pub name: String,
    pub dosage: String,
    pub frequency: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub prescriber_id: Option<Id>,
    pub active: bool,
}

/// Signature data for clinical notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureData {
    pub signature_data: String,
    pub signed_at: Timestamp,
    pub signer_id: Id,
}

/// Recurrence rule for appointments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurrenceRule {
    pub frequency: RecurrenceFrequency,
    pub interval: u32,
    pub count: Option<u32>,
    pub until: Option<Timestamp>,
    pub by_day: Option<Vec<String>>,
}

/// Recurrence frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecurrenceFrequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

/// Version vector for CRDT sync
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VersionVector {
    pub versions: std::collections::HashMap<String, u64>,
}

impl VersionVector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(&mut self, device_id: &str) {
        let counter = self.versions.entry(device_id.to_string()).or_insert(0);
        *counter += 1;
    }

    pub fn get(&self, device_id: &str) -> u64 {
        *self.versions.get(device_id).unwrap_or(&0)
    }

    pub fn merge(&mut self, other: &VersionVector) {
        for (device_id, version) in &other.versions {
            let current = self.versions.entry(device_id.clone()).or_insert(0);
            *current = (*current).max(*version);
        }
    }
}
