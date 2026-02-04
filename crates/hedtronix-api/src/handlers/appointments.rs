//! Appointment handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use hedtronix_core::{
    Appointment, AppointmentType, AppointmentStatus, CalendarFilters, Id,
};
use hedtronix_db::AppointmentRepository;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;

/// List appointments
pub async fn list_appointments(
    State(state): State<AppState>,
    Query(query): Query<CalendarQuery>,
) -> Result<Json<ListAppointmentsResponse>, ApiError> {
    let repo = AppointmentRepository::new(state.db.clone());
    
    // Default to today's appointments if no date range specified
    let now = chrono::Utc::now();
    let start = query.start.unwrap_or_else(|| now.date_naive().and_hms_opt(0, 0, 0).unwrap());
    let end = query.end.unwrap_or_else(|| now.date_naive().and_hms_opt(23, 59, 59).unwrap());
    
    let filters = CalendarFilters {
        start_date: chrono::DateTime::from_naive_utc_and_offset(start, chrono::Utc),
        end_date: chrono::DateTime::from_naive_utc_and_offset(end, chrono::Utc),
        ..Default::default()
    };
    
    // For simplicity, getting all appointments (in production, would filter by provider)
    let provider_id = Id::parse_str(&query.provider_id.unwrap_or_default())
        .unwrap_or_else(|_| Id::new_v4());
    
    let appointments = repo.find_by_provider(provider_id, &filters)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    Ok(Json(ListAppointmentsResponse {
        appointments: appointments.into_iter().map(AppointmentDto::from).collect(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct CalendarQuery {
    pub start: Option<chrono::NaiveDateTime>,
    pub end: Option<chrono::NaiveDateTime>,
    pub provider_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListAppointmentsResponse {
    pub appointments: Vec<AppointmentDto>,
}

/// Get appointment by ID
pub async fn get_appointment(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AppointmentDto>, ApiError> {
    let apt_id = Id::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid appointment ID"))?;
    
    let repo = AppointmentRepository::new(state.db.clone());
    let appointment = repo.find_by_id(apt_id)
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Appointment"))?;
    
    Ok(Json(AppointmentDto::from(appointment)))
}

/// Create new appointment
pub async fn create_appointment(
    State(state): State<AppState>,
    Json(req): Json<CreateAppointmentRequest>,
) -> Result<Json<AppointmentDto>, ApiError> {
    let patient_id = Id::parse_str(&req.patient_id)
        .map_err(|_| ApiError::bad_request("Invalid patient ID"))?;
    let provider_id = Id::parse_str(&req.provider_id)
        .map_err(|_| ApiError::bad_request("Invalid provider ID"))?;
    let created_by = Id::parse_str(&req.created_by.unwrap_or_default())
        .unwrap_or_else(|_| Id::new_v4());
    
    let start_time = chrono::DateTime::parse_from_rfc3339(&req.start_time)
        .map_err(|_| ApiError::bad_request("Invalid start time format"))?
        .with_timezone(&chrono::Utc);
    
    let apt_type = parse_appointment_type(&req.appointment_type)?;
    
    // Check for conflicts
    let repo = AppointmentRepository::new(state.db.clone());
    let end_time = start_time + chrono::Duration::minutes(req.duration as i64);
    let conflicts = repo.check_conflicts(provider_id, start_time, end_time, None)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    if !conflicts.is_empty() {
        return Err(ApiError::conflict("Provider has conflicting appointments"));
    }
    
    let appointment = Appointment::new(
        patient_id,
        provider_id,
        start_time,
        req.duration,
        apt_type,
        req.reason_for_visit,
        created_by,
    );
    
    repo.create(&appointment)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    // Track for sync
    let sync_engine = state.sync_engine();
    let _ = sync_engine.track_create(
        "Appointment",
        appointment.id,
        serde_json::to_value(&appointment).unwrap_or_default(),
    );
    
    Ok(Json(AppointmentDto::from(appointment)))
}

#[derive(Debug, Deserialize)]
pub struct CreateAppointmentRequest {
    pub patient_id: String,
    pub provider_id: String,
    pub start_time: String,
    pub duration: i32,
    pub appointment_type: String,
    pub reason_for_visit: String,
    pub created_by: Option<String>,
}

/// Update appointment
pub async fn update_appointment(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateAppointmentRequest>,
) -> Result<Json<AppointmentDto>, ApiError> {
    let apt_id = Id::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid appointment ID"))?;
    
    let repo = AppointmentRepository::new(state.db.clone());
    let mut appointment = repo.find_by_id(apt_id)
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Appointment"))?;
    
    if let Some(notes) = req.notes {
        appointment.notes = Some(notes);
    }
    if let Some(reason) = req.reason_for_visit {
        appointment.reason_for_visit = reason;
    }
    appointment.updated_at = chrono::Utc::now();
    
    repo.update(&appointment)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    Ok(Json(AppointmentDto::from(appointment)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateAppointmentRequest {
    pub notes: Option<String>,
    pub reason_for_visit: Option<String>,
}

/// Cancel appointment
pub async fn cancel_appointment(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CancelRequest>,
) -> Result<Json<AppointmentDto>, ApiError> {
    let apt_id = Id::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid appointment ID"))?;
    
    let repo = AppointmentRepository::new(state.db.clone());
    let mut appointment = repo.find_by_id(apt_id)
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Appointment"))?;
    
    appointment.cancel(req.reason.unwrap_or_else(|| "Cancelled".to_string()));
    
    repo.update(&appointment)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    Ok(Json(AppointmentDto::from(appointment)))
}

#[derive(Debug, Deserialize)]
pub struct CancelRequest {
    pub reason: Option<String>,
}

/// Check in patient
pub async fn check_in(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AppointmentDto>, ApiError> {
    let apt_id = Id::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid appointment ID"))?;
    
    let repo = AppointmentRepository::new(state.db.clone());
    let mut appointment = repo.find_by_id(apt_id)
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Appointment"))?;
    
    appointment.check_in();
    
    repo.update(&appointment)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    Ok(Json(AppointmentDto::from(appointment)))
}

/// Complete appointment
pub async fn complete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AppointmentDto>, ApiError> {
    let apt_id = Id::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid appointment ID"))?;
    
    let repo = AppointmentRepository::new(state.db.clone());
    let mut appointment = repo.find_by_id(apt_id)
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Appointment"))?;
    
    appointment.complete();
    
    repo.update(&appointment)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    Ok(Json(AppointmentDto::from(appointment)))
}

/// Check for conflicts
pub async fn check_conflicts(
    State(state): State<AppState>,
    Json(req): Json<ConflictCheckRequest>,
) -> Result<Json<ConflictCheckResponse>, ApiError> {
    let provider_id = Id::parse_str(&req.provider_id)
        .map_err(|_| ApiError::bad_request("Invalid provider ID"))?;
    
    let start_time = chrono::DateTime::parse_from_rfc3339(&req.start_time)
        .map_err(|_| ApiError::bad_request("Invalid start time"))?
        .with_timezone(&chrono::Utc);
    
    let end_time = chrono::DateTime::parse_from_rfc3339(&req.end_time)
        .map_err(|_| ApiError::bad_request("Invalid end time"))?
        .with_timezone(&chrono::Utc);
    
    let exclude_id = req.exclude_id.and_then(|s| Id::parse_str(&s).ok());
    
    let repo = AppointmentRepository::new(state.db.clone());
    let conflicts = repo.check_conflicts(provider_id, start_time, end_time, exclude_id)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    Ok(Json(ConflictCheckResponse {
        has_conflicts: !conflicts.is_empty(),
        conflicts: conflicts.into_iter().map(AppointmentDto::from).collect(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct ConflictCheckRequest {
    pub provider_id: String,
    pub start_time: String,
    pub end_time: String,
    pub exclude_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConflictCheckResponse {
    pub has_conflicts: bool,
    pub conflicts: Vec<AppointmentDto>,
}

/// Get calendar view
pub async fn get_calendar(
    State(state): State<AppState>,
    Query(query): Query<CalendarQuery>,
) -> Result<Json<CalendarResponse>, ApiError> {
    // Same as list_appointments but formatted for calendar
    let response = list_appointments(State(state), Query(query)).await?;
    
    Ok(Json(CalendarResponse {
        appointments: response.0.appointments,
    }))
}

#[derive(Debug, Serialize)]
pub struct CalendarResponse {
    pub appointments: Vec<AppointmentDto>,
}

// Helper functions
fn parse_appointment_type(s: &str) -> Result<AppointmentType, ApiError> {
    match s.to_uppercase().as_str() {
        "NEW_PATIENT" => Ok(AppointmentType::NewPatient),
        "FOLLOW_UP" => Ok(AppointmentType::FollowUp),
        "PROCEDURE" => Ok(AppointmentType::Procedure),
        "CONSULTATION" => Ok(AppointmentType::Consultation),
        "EMERGENCY" => Ok(AppointmentType::Emergency),
        _ => Err(ApiError::bad_request("Invalid appointment type")),
    }
}

/// Appointment DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct AppointmentDto {
    pub id: String,
    pub patient_id: String,
    pub provider_id: String,
    pub start_time: String,
    pub end_time: String,
    pub duration: i32,
    pub appointment_type: String,
    pub status: String,
    pub reason_for_visit: String,
    pub notes: Option<String>,
    pub check_in_time: Option<String>,
    pub wait_time: Option<i32>,
}

impl From<Appointment> for AppointmentDto {
    fn from(a: Appointment) -> Self {
        Self {
            id: a.id.to_string(),
            patient_id: a.patient_id.to_string(),
            provider_id: a.provider_id.to_string(),
            start_time: a.start_time.to_rfc3339(),
            end_time: a.end_time.to_rfc3339(),
            duration: a.duration,
            appointment_type: format!("{:?}", a.appointment_type).to_uppercase(),
            status: format!("{:?}", a.status).to_uppercase(),
            reason_for_visit: a.reason_for_visit,
            notes: a.notes,
            check_in_time: a.check_in_time.map(|t| t.to_rfc3339()),
            wait_time: a.wait_time,
        }
    }
}
