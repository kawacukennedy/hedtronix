//! Appointment model with conflict detection

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::types::{AppointmentStatus, AppointmentType, Id, RecurrenceRule, Timestamp, VersionVector};

/// Appointment entity for scheduling
/// CRDT Type: MV_REGISTER
/// Conflict Resolution: Time-based with resource conflict detection
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Appointment {
    pub id: Id,
    pub patient_id: Id,
    pub provider_id: Id,
    pub room_id: Option<Id>,
    
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    
    /// Duration in minutes
    pub duration: i32,
    
    pub appointment_type: AppointmentType,
    pub status: AppointmentStatus,
    
    pub cancellation_reason: Option<String>,
    
    #[validate(length(min = 1, max = 500))]
    pub reason_for_visit: String,
    
    pub check_in_time: Option<Timestamp>,
    pub check_out_time: Option<Timestamp>,
    
    /// Wait time in minutes
    pub wait_time: Option<i32>,
    
    pub recurrence_rule: Option<RecurrenceRule>,
    
    /// Notes for the appointment
    pub notes: Option<String>,
    
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: Id,
    
    /// CRDT version tracking
    pub version: VersionVector,
    
    /// Last modification device ID
    pub last_modified_by: Option<String>,
}

impl Appointment {
    pub fn new(
        patient_id: Id,
        provider_id: Id,
        start_time: Timestamp,
        duration: i32,
        appointment_type: AppointmentType,
        reason_for_visit: String,
        created_by: Id,
    ) -> Self {
        let now = chrono::Utc::now();
        let end_time = start_time + chrono::Duration::minutes(duration as i64);
        
        Self {
            id: Id::new_v4(),
            patient_id,
            provider_id,
            room_id: None,
            start_time,
            end_time,
            duration,
            appointment_type,
            status: AppointmentStatus::Scheduled,
            cancellation_reason: None,
            reason_for_visit,
            check_in_time: None,
            check_out_time: None,
            wait_time: None,
            recurrence_rule: None,
            notes: None,
            created_at: now,
            updated_at: now,
            created_by,
            version: VersionVector::new(),
            last_modified_by: None,
        }
    }

    /// Check if this appointment overlaps with a time range
    pub fn overlaps(&self, start: Timestamp, end: Timestamp) -> bool {
        self.start_time < end && self.end_time > start
    }

    /// Check in the patient
    pub fn check_in(&mut self) {
        let now = chrono::Utc::now();
        self.check_in_time = Some(now);
        self.status = AppointmentStatus::CheckedIn;
        
        // Calculate wait time from appointment start
        if now > self.start_time {
            self.wait_time = Some(0);
        } else {
            self.wait_time = Some(0);
        }
        self.updated_at = now;
    }

    /// Move patient to room
    pub fn move_to_room(&mut self, room_id: Id) {
        self.room_id = Some(room_id);
        self.status = AppointmentStatus::InRoom;
        
        // Update wait time
        if let Some(check_in) = self.check_in_time {
            let now = chrono::Utc::now();
            self.wait_time = Some((now - check_in).num_minutes() as i32);
        }
        self.updated_at = chrono::Utc::now();
    }

    /// Complete the appointment
    pub fn complete(&mut self) {
        self.check_out_time = Some(chrono::Utc::now());
        self.status = AppointmentStatus::Completed;
        self.updated_at = chrono::Utc::now();
    }

    /// Cancel the appointment
    pub fn cancel(&mut self, reason: String) {
        self.status = AppointmentStatus::Cancelled;
        self.cancellation_reason = Some(reason);
        self.updated_at = chrono::Utc::now();
    }

    /// Mark as no-show
    pub fn mark_no_show(&mut self) {
        self.status = AppointmentStatus::NoShow;
        self.updated_at = chrono::Utc::now();
    }

    /// Check if appointment is in the past
    pub fn is_past(&self) -> bool {
        self.end_time < chrono::Utc::now()
    }

    /// Check if appointment is upcoming
    pub fn is_upcoming(&self) -> bool {
        self.start_time > chrono::Utc::now()
    }
}

/// Appointment creation DTO
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateAppointment {
    pub patient_id: Id,
    pub provider_id: Id,
    pub room_id: Option<Id>,
    pub start_time: Timestamp,
    
    /// Duration in minutes
    pub duration: i32,
    
    pub appointment_type: AppointmentType,
    
    #[validate(length(min = 1, max = 500))]
    pub reason_for_visit: String,
    
    pub notes: Option<String>,
    pub recurrence_rule: Option<RecurrenceRule>,
}

/// Appointment update DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAppointment {
    pub provider_id: Option<Id>,
    pub room_id: Option<Id>,
    pub start_time: Option<Timestamp>,
    pub duration: Option<i32>,
    pub appointment_type: Option<AppointmentType>,
    pub status: Option<AppointmentStatus>,
    pub reason_for_visit: Option<String>,
    pub notes: Option<String>,
}

/// Conflict check request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictCheckRequest {
    pub resource_id: Id,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub exclude_appointment_id: Option<Id>,
}

/// Conflict check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictCheckResponse {
    pub has_conflict: bool,
    pub conflicting_appointments: Vec<Appointment>,
}

/// Calendar query filters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalendarFilters {
    pub start_date: Timestamp,
    pub end_date: Timestamp,
    pub provider_ids: Option<Vec<Id>>,
    pub room_ids: Option<Vec<Id>>,
    pub statuses: Option<Vec<AppointmentStatus>>,
}
