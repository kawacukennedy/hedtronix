//! Clinical Note handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use hedtronix_core::{ClinicalNote, NoteType, NoteStatus, Id, ClinicalNoteDto};
use hedtronix_db::ClinicalNoteRepository;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;

/// List clinical notes for a patient
pub async fn list_notes(
    State(state): State<AppState>,
    Path(patient_id): Path<String>,
) -> Result<Json<ListNotesResponse>, ApiError> {
    let pid = Id::parse_str(&patient_id)
        .map_err(|_| ApiError::bad_request("Invalid patient ID"))?;
        
    let repo = ClinicalNoteRepository::new(state.db.clone(), state.encryption_key.clone());
    let notes = repo.find_by_patient(pid)
        .map_err(|e| ApiError::internal(&e.to_string()))?;

    Ok(Json(ListNotesResponse {
        notes: notes.into_iter().map(ClinicalNoteDto::from).collect(),
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
    let author_id = Id::parse_str(&req.provider_id)
        .map_err(|_| ApiError::bad_request("Invalid provider ID"))?;
    
    let note_type = match req.note_type.to_uppercase().as_str() {
        "PROGRESS_NOTE" => NoteType::ProgressNote,
        "CONSULTATION" => NoteType::Consultation,
        "DISCHARGE_SUMMARY" => NoteType::DischargeSummary,
        "PROCEDURE_NOTE" => NoteType::ProcedureNote,
        _ => NoteType::ProgressNote,
    };
    
    let mut note = ClinicalNote::new(patient_id, author_id, note_type);
    note.content = req.content.unwrap_or_default();
    
    if let Some(encounter_id) = req.encounter_id {
        note.encounter_id = Id::parse_str(&encounter_id).ok();
    }
    
    let repo = ClinicalNoteRepository::new(state.db.clone(), state.encryption_key.clone());
    repo.create(&note)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
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
    pub note_type: String,
    pub content: Option<String>,
}

/// Get note
pub async fn get_note(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ClinicalNoteDto>, ApiError> {
    let note_id = Id::parse_str(&id).map_err(|_| ApiError::bad_request("Invalid ID"))?;
    
    let repo = ClinicalNoteRepository::new(state.db.clone(), state.encryption_key.clone());
    let note = repo.find_by_id(note_id)
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("ClinicalNote"))?;
        
    Ok(Json(ClinicalNoteDto::from(note)))
}

/// Update note
pub async fn update_note(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateNoteRequest>,
) -> Result<Json<ClinicalNoteDto>, ApiError> {
    let note_id = Id::parse_str(&id).map_err(|_| ApiError::bad_request("Invalid ID"))?;
    
    let repo = ClinicalNoteRepository::new(state.db.clone(), state.encryption_key.clone());
    let mut note = repo.find_by_id(note_id)
         .map_err(|e| ApiError::internal(&e.to_string()))?
         .ok_or_else(|| ApiError::not_found("ClinicalNote"))?;
         
    if let Some(content) = req.content {
        note.content = content;
    }
    
    if let Some(status) = req.status {
        match status.to_uppercase().as_str() {
            "DRAFT" => note.status = NoteStatus::Draft,
            "SIGNED" => note.status = NoteStatus::Signed, // Should use sign_note endpoint
            _ => {},
        }
    }
    
    note.updated_at = chrono::Utc::now();
    
    repo.update(&note)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
        
    // Track Sync
    let sync_engine = state.sync_engine();
    let _ = sync_engine.track_update(
        "ClinicalNote",
        note.id,
        serde_json::to_value(&note).unwrap_or_default(),
    );
     
    Ok(Json(ClinicalNoteDto::from(note)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteRequest {
    pub content: Option<String>,
    pub status: Option<String>,
}

/// Sign note
pub async fn sign_note(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<SignNoteRequest>,
) -> Result<Json<ClinicalNoteDto>, ApiError> {
    let note_id = Id::parse_str(&id).map_err(|_| ApiError::bad_request("Invalid ID"))?;
    let signer_id = Id::parse_str(&req.signer_id).map_err(|_| ApiError::bad_request("Invalid signer ID"))?;
    
    let repo = ClinicalNoteRepository::new(state.db.clone(), state.encryption_key.clone());
    let mut note = repo.find_by_id(note_id)
         .map_err(|e| ApiError::internal(&e.to_string()))?
         .ok_or_else(|| ApiError::not_found("ClinicalNote"))?;
         
    note.sign(signer_id, req.signature_data)
        .map_err(|e| ApiError::bad_request(e))?;
        
    repo.update(&note)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
        
    // Track Sync
    let sync_engine = state.sync_engine();
    let _ = sync_engine.track_update(
        "ClinicalNote",
        note.id,
        serde_json::to_value(&note).unwrap_or_default(),
    );
        
    Ok(Json(ClinicalNoteDto::from(note)))
}

#[derive(Debug, Deserialize)]
pub struct SignNoteRequest {
    pub signer_id: String,
    pub signature_data: String,
}
