//! Billing handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use hedtronix_core::{BillingEntry, BillingStatus, Id};
use hedtronix_db::BillingRepository;
use serde::{Deserialize, Serialize};
use crate::error::ApiError;
use crate::state::AppState;

pub async fn list_billing(
    State(state): State<AppState>,
) -> Result<Json<ListBillingResponse>, ApiError> {
    let repo = BillingRepository::new(state.db.clone());
    let entries = repo.find_all()
        .map_err(|e| ApiError::internal(&e.to_string()))?;
        
    Ok(Json(ListBillingResponse { 
        entries: entries.into_iter().map(BillingDto::from).collect() 
    }))
}

#[derive(Debug, Serialize)]
pub struct ListBillingResponse {
    pub entries: Vec<BillingDto>,
}

pub async fn create_billing(
    State(state): State<AppState>,
    Json(req): Json<CreateBillingRequest>,
) -> Result<Json<BillingDto>, ApiError> {
    let patient_id = Id::parse_str(&req.patient_id)
        .map_err(|_| ApiError::bad_request("Invalid patient ID"))?;
    let encounter_id = Id::parse_str(&req.encounter_id)
        .map_err(|_| ApiError::bad_request("Invalid encounter ID"))?;
    let provider_id = Id::parse_str(&req.provider_id)
        .map_err(|_| ApiError::bad_request("Invalid provider ID"))?;
    let created_by = provider_id;
    
    let entry = BillingEntry::new(
        patient_id,
        encounter_id,
        provider_id,
        req.cpt_code.clone(),
        req.description.clone(),
        req.unit_price.clone(),
        created_by,
    );
    
    let repo = BillingRepository::new(state.db.clone());
    repo.create(&entry)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    // Sync tracking
    let sync_engine = state.sync_engine();
    let _ = sync_engine.track_create(
        "BillingEntry",
        entry.id,
        serde_json::to_value(&entry).unwrap_or_default(),
    );
    
    Ok(Json(BillingDto::from(entry)))
}

#[derive(Debug, Deserialize)]
pub struct CreateBillingRequest {
    pub patient_id: String,
    pub encounter_id: String,
    pub provider_id: String,
    pub cpt_code: String,
    pub description: String,
    pub unit_price: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BillingDto {
    pub id: String,
    pub patient_id: String,
    pub cpt_code: String,
    pub description: String,
    pub total_amount: String,
    pub status: String,
}

impl From<BillingEntry> for BillingDto {
    fn from(b: BillingEntry) -> Self {
        Self {
            id: b.id.to_string(),
            patient_id: b.patient_id.to_string(),
            cpt_code: b.cpt_code,
            description: b.description,
            total_amount: b.total_amount,
            status: format!("{:?}", b.status),
        }
    }
}

pub async fn update_billing(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateBillingRequest>,
) -> Result<Json<BillingDto>, ApiError> {
    let entry_id = Id::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid ID"))?;
        
    let repo = BillingRepository::new(state.db.clone());
    let mut entry = repo.find_by_id(entry_id)
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("BillingEntry"))?;
        
    if let Some(status) = req.status {
        match status.to_uppercase().as_str() {
             "SUBMITTED" => entry.status = BillingStatus::Submitted,
             "PAID" => entry.status = BillingStatus::Paid,
             "DENIED" => entry.status = BillingStatus::Denied,
             _ => {},
        }
    }
    
    entry.updated_at = chrono::Utc::now();
    
    repo.update(&entry)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
        
    // Sync tracking
    let sync_engine = state.sync_engine();
    let _ = sync_engine.track_update(
        "BillingEntry",
        entry.id,
        serde_json::to_value(&entry).unwrap_or_default(),
    );
    
    Ok(Json(BillingDto::from(entry)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateBillingRequest {
    pub status: Option<String>,
}

pub async fn submit_billing(
    State(_state): State<AppState>,
    Json(_req): Json<SubmitBillingRequest>,
) -> Result<Json<SubmitBillingResponse>, ApiError> {
    Ok(Json(SubmitBillingResponse {
        submitted_count: 0,
        claim_numbers: vec![],
    }))
}

#[derive(Debug, Deserialize)]
pub struct SubmitBillingRequest {
    pub entry_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SubmitBillingResponse {
    pub submitted_count: i32,
    pub claim_numbers: Vec<String>,
}
