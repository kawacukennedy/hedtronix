//! Clinical Note repository

use hedtronix_core::{ClinicalNote, Id, NoteType, NoteStatus, SoapSection, SoapItem, SignatureData};
use crate::{Database, DbError, Result};
use rusqlite::{params, Row};
use std::sync::Arc;
use hedtronix_crypto::{encrypt_field, decrypt_field};

pub struct ClinicalNoteRepository {
    db: Database,
    encryption_key: Vec<u8>,
}

impl ClinicalNoteRepository {
    pub fn new(db: Database, encryption_key: Vec<u8>) -> Self {
        Self { db, encryption_key }
    }

    pub fn create(&self, note: &ClinicalNote) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let key = &self.encryption_key;
        let content_enc = encrypt_field(&note.content, key)
            .map_err(|e| DbError::Serialization(format!("Encryption failed: {}", e)))?;

        conn.execute(
            r#"
            INSERT INTO clinical_notes (
                id, patient_id, author_id, encounter_id, note_type,
                content, status, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                note.id.to_string(),
                note.patient_id.to_string(),
                note.author_id.to_string(),
                note.encounter_id.map(|id| id.to_string()),
                format!("{:?}", note.note_type),
                content_enc,
                format!("{:?}", note.status),
                note.created_at.to_rfc3339(),
                note.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    pub fn find_by_id(&self, id: Id) -> Result<Option<ClinicalNote>> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, patient_id, author_id, encounter_id, note_type,
                   content, status, created_at, updated_at
            FROM clinical_notes
            WHERE id = ?
            "#,
        )?;

        let key = &self.encryption_key;
        let note = stmt.query_row([id.to_string()], |row| {
             Self::map_row_to_note(row, key)
        }).ok();

        Ok(note)
    }

    pub fn update(&self, note: &ClinicalNote) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let key = &self.encryption_key;
        let content_enc = encrypt_field(&note.content, key)
            .map_err(|e| DbError::Serialization(format!("Encryption failed: {}", e)))?;

        conn.execute(
            r#"
            UPDATE clinical_notes
            SET content = ?, status = ?, updated_at = ?
            WHERE id = ?
            "#,
            params![
                content_enc,
                format!("{:?}", note.status),
                note.updated_at.to_rfc3339(),
                note.id.to_string(),
            ],
        )?;

        Ok(())
    }
    
    pub fn find_by_patient(&self, patient_id: Id) -> Result<Vec<ClinicalNote>> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, patient_id, author_id, encounter_id, note_type,
                   content, status, created_at, updated_at
            FROM clinical_notes
            WHERE patient_id = ?
            ORDER BY created_at DESC
            "#,
        )?;

        let key = &self.encryption_key;
        let notes = stmt.query_map([patient_id.to_string()], |row| {
            Self::map_row_to_note(row, key)
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(notes)
    }

    fn map_row_to_note(row: &Row, key: &[u8]) -> rusqlite::Result<ClinicalNote> {
        let id: String = row.get(0)?;
        let patient_id: String = row.get(1)?;
        let author_id: String = row.get(2)?;
        let encounter_id: Option<String> = row.get(3)?;
        let note_type: String = row.get(4).unwrap_or("ProgressNote".to_string());
        let content_enc: String = row.get(5).unwrap_or_default();
        let status: String = row.get(6).unwrap_or("Draft".to_string());
        let created_at: String = row.get(7)?;
        let updated_at: String = row.get(8)?;

        let content = if content_enc.is_empty() {
             String::new()
        } else {
             decrypt_field(&content_enc, key).unwrap_or_else(|_| "[Decryption Failed]".to_string())
        };

        let nt = match note_type.to_uppercase().as_str() {
            "PROGRESS_NOTE" => NoteType::ProgressNote,
            "CONSULTATION" => NoteType::Consultation,
            "DISCHARGE_SUMMARY" => NoteType::DischargeSummary,
            _ => NoteType::ProgressNote,
        };
        
        let st = match status.to_uppercase().as_str() {
            "DRAFT" => NoteStatus::Draft,
            "SIGNED" => NoteStatus::Signed,
            _ => NoteStatus::Draft,
        };

        Ok(ClinicalNote {
            id: Id::parse_str(&id).unwrap_or_else(|_| Id::new_v4()),
            patient_id: Id::parse_str(&patient_id).unwrap_or_else(|_| Id::new_v4()),
            author_id: Id::parse_str(&author_id).unwrap_or_else(|_| Id::new_v4()),
            encounter_id: encounter_id.and_then(|s| Id::parse_str(&s).ok()),
            note_type: nt,
            content,
            // SOAP sections would be in a separate table in production
            subjective: None,
            objective: None,
            assessment: None,
            plan: None,
            signature: None,
            co_signer_id: None,
            co_signature: None,
            status: st,
            amends_note_id: None,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            signed_at: None,
            version: Default::default(),
            last_modified_by: None,
        })
    }


