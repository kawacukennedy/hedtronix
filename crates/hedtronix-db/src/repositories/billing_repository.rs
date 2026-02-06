//! Billing repository

use hedtronix_core::{BillingEntry, BillingStatus, Id, Money};
use crate::{Database, DbError, Result};
use rusqlite::{params, Row};
use std::sync::Arc;

pub struct BillingRepository {
    db: Database,
}

impl BillingRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn create(&self, entry: &BillingEntry) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        conn.execute(
            r#"
            INSERT INTO billing_entries (
                id, patient_id, encounter_id, provider_id, cpt_code,
                description, unit_price, total_amount, status, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                entry.id.to_string(),
                entry.patient_id.to_string(),
                entry.encounter_id.to_string(),
                entry.provider_id.to_string(),
                entry.cpt_code,
                entry.description,
                entry.unit_price,
                entry.total_amount,
                format!("{:?}", entry.status),
                entry.created_at.to_rfc3339(),
                entry.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    pub fn find_by_id(&self, id: Id) -> Result<Option<BillingEntry>> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, patient_id, encounter_id, provider_id, cpt_code,
                   description, unit_price, total_amount, status, created_at, updated_at
            FROM billing_entries
            WHERE id = ?
            "#,
        )?;

        let entry = stmt.query_row([id.to_string()], |row| {
             Ok(map_row_to_billing(row))
        }).ok();

        Ok(entry)
    }

    pub fn update(&self, entry: &BillingEntry) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        conn.execute(
            r#"
            UPDATE billing_entries
            SET status = ?, updated_at = ?
            WHERE id = ?
            "#,
            params![
                format!("{:?}", entry.status),
                entry.updated_at.to_rfc3339(),
                entry.id.to_string(),
            ],
        )?;

        Ok(())
    }
    
    pub fn find_all(&self) -> Result<Vec<BillingEntry>> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, patient_id, encounter_id, provider_id, cpt_code,
                   description, unit_price, total_amount, status, created_at, updated_at
            FROM billing_entries
            ORDER BY created_at DESC
            "#,
        )?;

        let entries = stmt.query_map([], |row| {
            Ok(map_row_to_billing(row))
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(entries)
    }
}

fn map_row_to_billing(row: &Row) -> BillingEntry {
    let id: String = row.get(0).unwrap();
    let patient_id: String = row.get(1).unwrap();
    let encounter_id: String = row.get(2).unwrap();
    let provider_id: String = row.get(3).unwrap();
    let cpt_code: String = row.get(4).unwrap();
    let description: String = row.get(5).unwrap();
    let unit_price: String = row.get(6).unwrap();
    let total_amount: String = row.get(7).unwrap();
    let status: String = row.get(8).unwrap();
    let created_at: String = row.get(9).unwrap();
    let updated_at: String = row.get(10).unwrap();

    let st = match status.to_uppercase().as_str() {
        "DRAFT" => BillingStatus::Draft,
        "SUBMITTED" => BillingStatus::Submitted,
        "PAID" => BillingStatus::Paid,
        "DENIED" => BillingStatus::Denied,
        _ => BillingStatus::Draft,
    };

    BillingEntry {
        id: Id::parse_str(&id).unwrap(),
        patient_id: Id::parse_str(&patient_id).unwrap(),
        encounter_id: Id::parse_str(&encounter_id).unwrap(),
        provider_id: Id::parse_str(&provider_id).unwrap(),
        cpt_code,
        icd10_codes: vec![], // In separate table in real app
        description,
        units: 1,
        unit_price,
        total_amount,
        insurance_estimated: None,
        patient_responsibility: None,
        status: st,
        submitted_at: None,
        paid_at: None,
        claim_number: None,
        denial_reason: None,
        adjustment_reason: None,
        adjustment_amount: None,
        created_at: chrono::DateTime::parse_from_rfc3339(&created_at).unwrap().with_timezone(&chrono::Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at).unwrap().with_timezone(&chrono::Utc),
        created_by: Id::parse_str(&provider_id).unwrap(), // Approximated
        version: Default::default(),
    }
}
