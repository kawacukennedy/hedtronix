//! Billing model

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::types::{BillingStatus, Id, Timestamp, VersionVector};

/// Billing Entry entity
/// CRDT Type: LWW_REGISTER
/// Conflict Resolution: Timestamp-based with validation against code master
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct BillingEntry {
    pub id: Id,
    pub patient_id: Id,
    pub encounter_id: Id,
    pub provider_id: Id,
    
    /// CPT procedure code
    #[validate(length(min = 1, max = 10))]
    pub cpt_code: String,
    
    /// ICD-10 diagnosis codes
    pub icd10_codes: Vec<String>,
    
    #[validate(length(min = 1, max = 500))]
    pub description: String,
    
    /// Number of units
    pub units: i32,
    
    /// Price per unit (stored as string for decimal precision)
    pub unit_price: String,
    
    /// Total amount (units * unit_price)
    pub total_amount: String,
    
    /// Estimated insurance payment
    pub insurance_estimated: Option<String>,
    
    /// Patient responsibility after insurance
    pub patient_responsibility: Option<String>,
    
    pub status: BillingStatus,
    
    pub submitted_at: Option<Timestamp>,
    pub paid_at: Option<Timestamp>,
    
    /// Claim reference number
    pub claim_number: Option<String>,
    
    /// Denial reason if claim was denied
    pub denial_reason: Option<String>,
    
    /// Adjustment reason for write-offs
    pub adjustment_reason: Option<String>,
    pub adjustment_amount: Option<String>,
    
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: Id,
    
    /// CRDT version tracking
    pub version: VersionVector,
}

impl BillingEntry {
    pub fn new(
        patient_id: Id,
        encounter_id: Id,
        provider_id: Id,
        cpt_code: String,
        description: String,
        unit_price: String,
        created_by: Id,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Id::new_v4(),
            patient_id,
            encounter_id,
            provider_id,
            cpt_code,
            icd10_codes: Vec::new(),
            description,
            units: 1,
            unit_price: unit_price.clone(),
            total_amount: unit_price,
            insurance_estimated: None,
            patient_responsibility: None,
            status: BillingStatus::Draft,
            submitted_at: None,
            paid_at: None,
            claim_number: None,
            denial_reason: None,
            adjustment_reason: None,
            adjustment_amount: None,
            created_at: now,
            updated_at: now,
            created_by,
            version: VersionVector::new(),
        }
    }

    /// Mark as billed
    pub fn bill(&mut self) {
        self.status = BillingStatus::Billed;
        self.updated_at = chrono::Utc::now();
    }

    /// Submit claim to insurance
    pub fn submit(&mut self, claim_number: String) {
        self.status = BillingStatus::Submitted;
        self.claim_number = Some(claim_number);
        self.submitted_at = Some(chrono::Utc::now());
        self.updated_at = chrono::Utc::now();
    }

    /// Mark as paid
    pub fn mark_paid(&mut self) {
        self.status = BillingStatus::Paid;
        self.paid_at = Some(chrono::Utc::now());
        self.updated_at = chrono::Utc::now();
    }

    /// Mark as denied
    pub fn deny(&mut self, reason: String) {
        self.status = BillingStatus::Denied;
        self.denial_reason = Some(reason);
        self.updated_at = chrono::Utc::now();
    }

    /// Appeal denial
    pub fn appeal(&mut self) {
        self.status = BillingStatus::Appealed;
        self.updated_at = chrono::Utc::now();
    }

    /// Add diagnosis code
    pub fn add_diagnosis(&mut self, icd10_code: String) {
        if !self.icd10_codes.contains(&icd10_code) {
            self.icd10_codes.push(icd10_code);
            self.updated_at = chrono::Utc::now();
        }
    }
}

/// Billing entry creation DTO
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateBillingEntry {
    pub patient_id: Id,
    pub encounter_id: Id,
    pub provider_id: Id,
    
    #[validate(length(min = 1, max = 10))]
    pub cpt_code: String,
    
    pub icd10_codes: Option<Vec<String>>,
    
    #[validate(length(min = 1, max = 500))]
    pub description: String,
    
    pub units: Option<i32>,
    pub unit_price: String,
}

/// Payment recording DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordPayment {
    pub billing_entry_id: Id,
    pub amount: String,
    pub payment_method: PaymentMethod,
    pub reference_number: Option<String>,
}

/// Payment methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentMethod {
    Cash,
    Check,
    CreditCard,
    Insurance,
    Eft,
}
