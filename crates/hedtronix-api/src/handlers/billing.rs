//! Billing handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use hedtronix_core::{BillingEntry, Id, Money};
use serde::{Deserialize, Serialize};
use crate::error::ApiError;
use crate::state::AppState;

pub async fn list_billing(
    State(_state): State<AppState>,
) -> Result<Json<ListBillingResponse>, ApiError> {
    Ok(Json(ListBillingResponse { entries: vec![] }))
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
    let provider_id = Id::parse_str(&req.provider_id)
        .map_err(|_| ApiError::bad_request("Invalid provider ID"))?;
        
    let entry = BillingEntry {
        id: Id::new_v4(),
        patient_id,
        provider_id,
        encounter_id: None,
        amount: Money::new(req.amount_cents, "USD".to_string()),
        status: hedtronix_core::BillingStatus::Pending,
        codes: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: Default::default(),
    };
    
    // Sync tracking would go here
    
    Ok(Json(BillingDto::from(entry)))
}

#[derive(Debug, Deserialize)]
pub struct CreateBillingRequest {
    pub patient_id: String,
    pub provider_id: String,
    pub amount_cents: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BillingDto {
    pub id: String,
    pub status: String,
    pub amount: String,
}

impl From<BillingEntry> for BillingDto {
    fn from(b: BillingEntry) -> Self {
        Self {
            id: b.id.to_string(),
            status: format!("{:?}", b.status),
            amount: format!("{} {}", b.amount.amount, b.amount.currency),
        }
    }
}
