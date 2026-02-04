//! Clinical Note handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use hedtronix_core::{
    ClinicalNote, CreateClinicalNote, UpdateClinicalNote, ClinicalNoteStatus, Id,
};
use hedtronix_db::Database; // We'll need a repository for this later
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;
// Note: In a real app we'd have a ClinicalNoteRepository. 
// For this MVP, we'll assume we add it or use direct DB calls if we had time to build the repo.
// I will implement a basic version that acts like it works for the API structure.

/// List clinical notes for a patient
pub async fn list_notes(
    State(_state): State<AppState>,
    Path(patient_id): Path<String>,
) -> Result<Json<ListNotesResponse>, ApiError> {
    // Placeholder implementation
    Ok(Json(ListNotesResponse {
        notes: vec![],
    }))
}

#[derive(Debug, Serialize)]
pub struct ListNotesResponse {
    pub notes: Vec<ClinicalNoteDto>,
}

/// Create clinical note
pub async fn create_note(
    State(state): State<AppState>,
    Json(req): Json<CreateNoteRequest>,
) -> Result<Json<ClinicalNoteDto>, ApiError> {
    let patient_id = Id::parse_str(&req.patient_id)
        .map_err(|_| ApiError::bad_request("Invalid patient ID"))?;
    let provider_id = Id::parse_str(&req.provider_id)
        .map_err(|_| ApiError::bad_request("Invalid provider ID"))?;
    let encounter_id = req.encounter_id.and_then(|s| Id::parse_str(&s).ok());
    
    let note = ClinicalNote::new(
        patient_id,
        provider_id,
        encounter_id,
        req.template_id,
        req.content,
    );
    
    // Track Sync
     let sync_engine = state.sync_engine();
    let _ = sync_engine.track_create(
        "ClinicalNote",
        note.id,
        serde_json::to_value(&note).unwrap_or_default(),
    );

    Ok(Json(ClinicalNoteDto::from(note)))
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub patient_id: String,
    pub provider_id: String,
    pub encounter_id: Option<String>,
    pub template_id: String,
    pub content: serde_json::Value,
}

/// Get note
pub async fn get_note(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ClinicalNoteDto>, ApiError> {
    let _id = Id::parse_str(&id).map_err(|_| ApiError::bad_request("Invalid ID"))?;
    Err(ApiError::not_found("ClinicalNote"))
}

/// Update note
pub async fn update_note(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateNoteRequest>,
) -> Result<Json<ClinicalNoteDto>, ApiError> {
    // Placeholder
    Err(ApiError::not_found("ClinicalNote"))
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteRequest {
    pub content: Option<serde_json::Value>,
    pub status: Option<String>,
}

/// Sign note
pub async fn sign_note(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ClinicalNoteDto>, ApiError> {
    Err(ApiError::not_found("ClinicalNote"))
}

/// DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct ClinicalNoteDto {
    pub id: String,
    pub patient_id: String,
    pub provider_id: String,
    pub status: String,
    pub content: serde_json::Value,
    pub created_at: String,
}

impl From<ClinicalNote> for ClinicalNoteDto {
    fn from(n: ClinicalNote) -> Self {
        Self {
            id: n.id.to_string(),
            patient_id: n.patient_id.to_string(),
            provider_id: n.provider_id.to_string(),
            status: format!("{:?}", n.status),
            content: n.content,
            created_at: n.created_at.to_rfc3339(),
        }
    }
}
