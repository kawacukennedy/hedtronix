//! Appointment repository

use rusqlite::{params, Row};
use hedtronix_core::{Appointment, AppointmentStatus, AppointmentType, CalendarFilters, Id, RecurrenceRule, VersionVector};
use crate::{Database, DbError, Result};

pub struct AppointmentRepository {
    db: Database,
}

impl AppointmentRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn row_to_appointment(row: &Row) -> rusqlite::Result<Appointment> {
        let id: String = row.get(0)?;
        let patient_id: String = row.get(1)?;
        let provider_id: String = row.get(2)?;
        let room_id: Option<String> = row.get(3)?;
        let start_time: String = row.get(4)?;
        let end_time: String = row.get(5)?;
        let duration: i32 = row.get(6)?;
        let appointment_type: String = row.get(7)?;
        let status: String = row.get(8)?;
        let cancellation_reason: Option<String> = row.get(9)?;
        let reason_for_visit: String = row.get(10)?;
        let check_in_time: Option<String> = row.get(11)?;
        let check_out_time: Option<String> = row.get(12)?;
        let wait_time: Option<i32> = row.get(13)?;
        let recurrence_json: Option<String> = row.get(14)?;
        let notes: Option<String> = row.get(15)?;
        let created_at: String = row.get(16)?;
        let updated_at: String = row.get(17)?;
        let created_by: String = row.get(18)?;
        let version_json: String = row.get(19)?;
        let last_modified_by: Option<String> = row.get(20)?;

        let apt_type = match appointment_type.as_str() {
            "NEW_PATIENT" => AppointmentType::NewPatient,
            "FOLLOW_UP" => AppointmentType::FollowUp,
            "PROCEDURE" => AppointmentType::Procedure,
            "CONSULTATION" => AppointmentType::Consultation,
            "EMERGENCY" => AppointmentType::Emergency,
            _ => AppointmentType::FollowUp,
        };

        let apt_status = match status.as_str() {
            "SCHEDULED" => AppointmentStatus::Scheduled,
            "CHECKED_IN" => AppointmentStatus::CheckedIn,
            "IN_ROOM" => AppointmentStatus::InRoom,
            "COMPLETED" => AppointmentStatus::Completed,
            "CANCELLED" => AppointmentStatus::Cancelled,
            "NO_SHOW" => AppointmentStatus::NoShow,
            _ => AppointmentStatus::Scheduled,
        };

        Ok(Appointment {
            id: Id::parse_str(&id).unwrap_or_else(|_| Id::new_v4()),
            patient_id: Id::parse_str(&patient_id).unwrap_or_else(|_| Id::new_v4()),
            provider_id: Id::parse_str(&provider_id).unwrap_or_else(|_| Id::new_v4()),
            room_id: room_id.and_then(|s| Id::parse_str(&s).ok()),
            start_time: chrono::DateTime::parse_from_rfc3339(&start_time)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            end_time: chrono::DateTime::parse_from_rfc3339(&end_time)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            duration,
            appointment_type: apt_type,
            status: apt_status,
            cancellation_reason,
            reason_for_visit,
            check_in_time: check_in_time.and_then(|s| 
                chrono::DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&chrono::Utc)).ok()
            ),
            check_out_time: check_out_time.and_then(|s| 
                chrono::DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&chrono::Utc)).ok()
            ),
            wait_time,
            recurrence_rule: recurrence_json.and_then(|s| serde_json::from_str(&s).ok()),
            notes,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            created_by: Id::parse_str(&created_by).unwrap_or_else(|_| Id::new_v4()),
            version: serde_json::from_str(&version_json).unwrap_or_default(),
            last_modified_by,
        })
    }

    fn status_to_str(status: &AppointmentStatus) -> &'static str {
        match status {
            AppointmentStatus::Scheduled => "SCHEDULED",
            AppointmentStatus::CheckedIn => "CHECKED_IN",
            AppointmentStatus::InRoom => "IN_ROOM",
            AppointmentStatus::Completed => "COMPLETED",
            AppointmentStatus::Cancelled => "CANCELLED",
            AppointmentStatus::NoShow => "NO_SHOW",
        }
    }

    fn type_to_str(apt_type: &AppointmentType) -> &'static str {
        match apt_type {
            AppointmentType::NewPatient => "NEW_PATIENT",
            AppointmentType::FollowUp => "FOLLOW_UP",
            AppointmentType::Procedure => "PROCEDURE",
            AppointmentType::Consultation => "CONSULTATION",
            AppointmentType::Emergency => "EMERGENCY",
        }
    }

    pub fn create(&self, appointment: &Appointment) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        conn.execute(
            r#"
            INSERT INTO appointments (
                id, patient_id, provider_id, room_id, start_time, end_time,
                duration, appointment_type, status, cancellation_reason,
                reason_for_visit, check_in_time, check_out_time, wait_time,
                recurrence_rule_json, notes, created_at, updated_at, created_by,
                version_json, last_modified_by
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                appointment.id.to_string(),
                appointment.patient_id.to_string(),
                appointment.provider_id.to_string(),
                appointment.room_id.map(|id| id.to_string()),
                appointment.start_time.to_rfc3339(),
                appointment.end_time.to_rfc3339(),
                appointment.duration,
                Self::type_to_str(&appointment.appointment_type),
                Self::status_to_str(&appointment.status),
                appointment.cancellation_reason,
                appointment.reason_for_visit,
                appointment.check_in_time.map(|dt| dt.to_rfc3339()),
                appointment.check_out_time.map(|dt| dt.to_rfc3339()),
                appointment.wait_time,
                appointment.recurrence_rule.as_ref().and_then(|r| serde_json::to_string(r).ok()),
                appointment.notes,
                appointment.created_at.to_rfc3339(),
                appointment.updated_at.to_rfc3339(),
                appointment.created_by.to_string(),
                serde_json::to_string(&appointment.version).unwrap_or_default(),
                appointment.last_modified_by.clone(),
            ],
        )?;

        Ok(())
    }

    pub fn find_by_id(&self, id: Id) -> Result<Option<Appointment>> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, patient_id, provider_id, room_id, start_time, end_time,
                   duration, appointment_type, status, cancellation_reason,
                   reason_for_visit, check_in_time, check_out_time, wait_time,
                   recurrence_rule_json, notes, created_at, updated_at, created_by,
                   version_json, last_modified_by
            FROM appointments WHERE id = ?
            "#
        )?;

        let appointment = stmt.query_row([id.to_string()], Self::row_to_appointment).ok();
        Ok(appointment)
    }

    pub fn find_by_provider(&self, provider_id: Id, filters: &CalendarFilters) -> Result<Vec<Appointment>> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, patient_id, provider_id, room_id, start_time, end_time,
                   duration, appointment_type, status, cancellation_reason,
                   reason_for_visit, check_in_time, check_out_time, wait_time,
                   recurrence_rule_json, notes, created_at, updated_at, created_by,
                   version_json, last_modified_by
            FROM appointments 
            WHERE provider_id = ?
              AND start_time >= ?
              AND end_time <= ?
            ORDER BY start_time
            "#
        )?;

        let appointments = stmt
            .query_map([
                provider_id.to_string(),
                filters.start_date.to_rfc3339(),
                filters.end_date.to_rfc3339(),
            ], Self::row_to_appointment)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(appointments)
    }

    pub fn find_by_patient(&self, patient_id: Id) -> Result<Vec<Appointment>> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, patient_id, provider_id, room_id, start_time, end_time,
                   duration, appointment_type, status, cancellation_reason,
                   reason_for_visit, check_in_time, check_out_time, wait_time,
                   recurrence_rule_json, notes, created_at, updated_at, created_by,
                   version_json, last_modified_by
            FROM appointments 
            WHERE patient_id = ?
            ORDER BY start_time DESC
            "#
        )?;

        let appointments = stmt
            .query_map([patient_id.to_string()], Self::row_to_appointment)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(appointments)
    }

    /// Check for scheduling conflicts
    pub fn check_conflicts(
        &self,
        provider_id: Id,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
        exclude_id: Option<Id>,
    ) -> Result<Vec<Appointment>> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let sql = if let Some(exclude) = exclude_id {
            format!(
                r#"
                SELECT id, patient_id, provider_id, room_id, start_time, end_time,
                       duration, appointment_type, status, cancellation_reason,
                       reason_for_visit, check_in_time, check_out_time, wait_time,
                       recurrence_rule_json, notes, created_at, updated_at, created_by,
                       version_json, last_modified_by
                FROM appointments 
                WHERE provider_id = ?
                  AND id != '{}'
                  AND status NOT IN ('CANCELLED', 'NO_SHOW')
                  AND start_time < ?
                  AND end_time > ?
                "#,
                exclude
            )
        } else {
            r#"
            SELECT id, patient_id, provider_id, room_id, start_time, end_time,
                   duration, appointment_type, status, cancellation_reason,
                   reason_for_visit, check_in_time, check_out_time, wait_time,
                   recurrence_rule_json, notes, created_at, updated_at, created_by,
                   version_json, last_modified_by
            FROM appointments 
            WHERE provider_id = ?
              AND status NOT IN ('CANCELLED', 'NO_SHOW')
              AND start_time < ?
              AND end_time > ?
            "#.to_string()
        };

        let mut stmt = conn.prepare(&sql)?;
        let conflicts = stmt
            .query_map([
                provider_id.to_string(),
                end_time.to_rfc3339(),
                start_time.to_rfc3339(),
            ], Self::row_to_appointment)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(conflicts)
    }

    pub fn update(&self, appointment: &Appointment) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        conn.execute(
            r#"
            UPDATE appointments SET
                patient_id = ?, provider_id = ?, room_id = ?,
                start_time = ?, end_time = ?, duration = ?,
                appointment_type = ?, status = ?, cancellation_reason = ?,
                reason_for_visit = ?, check_in_time = ?, check_out_time = ?,
                wait_time = ?, recurrence_rule_json = ?, notes = ?,
                updated_at = ?, version_json = ?, last_modified_by = ?
            WHERE id = ?
            "#,
            params![
                appointment.patient_id.to_string(),
                appointment.provider_id.to_string(),
                appointment.room_id.map(|id| id.to_string()),
                appointment.start_time.to_rfc3339(),
                appointment.end_time.to_rfc3339(),
                appointment.duration,
                Self::type_to_str(&appointment.appointment_type),
                Self::status_to_str(&appointment.status),
                appointment.cancellation_reason,
                appointment.reason_for_visit,
                appointment.check_in_time.map(|dt| dt.to_rfc3339()),
                appointment.check_out_time.map(|dt| dt.to_rfc3339()),
                appointment.wait_time,
                appointment.recurrence_rule.as_ref().and_then(|r| serde_json::to_string(r).ok()),
                appointment.notes,
                appointment.updated_at.to_rfc3339(),
                serde_json::to_string(&appointment.version).unwrap_or_default(),
                appointment.last_modified_by.clone(),
                appointment.id.to_string(),
            ],
        )?;

        Ok(())
    }

    pub fn delete(&self, id: Id) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        conn.execute("DELETE FROM appointments WHERE id = ?", [id.to_string()])?;
        Ok(())
    }

    /// Get today's appointments for a provider
    pub fn get_todays_appointments(&self, provider_id: Id) -> Result<Vec<Appointment>> {
        let today = chrono::Utc::now().date_naive();
        let start = today.and_hms_opt(0, 0, 0).unwrap();
        let end = today.and_hms_opt(23, 59, 59).unwrap();

        let filters = CalendarFilters {
            start_date: chrono::DateTime::from_naive_utc_and_offset(start, chrono::Utc),
            end_date: chrono::DateTime::from_naive_utc_and_offset(end, chrono::Utc),
            ..Default::default()
        };

        self.find_by_provider(provider_id, &filters)
    }
}
