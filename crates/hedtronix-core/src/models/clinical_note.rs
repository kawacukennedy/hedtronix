//! Clinical Note model with RGA-based rich text

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::types::{Id, NoteStatus, NoteType, SignatureData, Timestamp, VersionVector};

/// Clinical Note entity with SOAP structure
/// CRDT Type: Composite (RGA for content, LWW_MAP for structured fields)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ClinicalNote {
    pub id: Id,
    pub patient_id: Id,
    pub author_id: Id,
    pub encounter_id: Option<Id>,
    
    pub note_type: NoteType,
    
    /// Rich text content - RGA CRDT type
    pub content: String,
    
    /// Subjective section (SOAP)
    pub subjective: Option<SoapSection>,
    
    /// Objective section (SOAP)
    pub objective: Option<SoapSection>,
    
    /// Assessment section (SOAP)
    pub assessment: Option<SoapSection>,
    
    /// Plan section (SOAP)
    pub plan: Option<SoapSection>,
    
    /// Digital signature
    pub signature: Option<SignatureData>,
    
    /// Co-signer for notes requiring supervision
    pub co_signer_id: Option<Id>,
    pub co_signature: Option<SignatureData>,
    
    pub status: NoteStatus,
    
    /// Previous version ID for amendments
    pub amends_note_id: Option<Id>,
    
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub signed_at: Option<Timestamp>,
    
    /// CRDT version tracking
    pub version: VersionVector,
    
    /// Last modification device ID
    pub last_modified_by: Option<String>,
}

/// SOAP section structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SoapSection {
    pub content: String,
    pub items: Vec<SoapItem>,
}

/// Individual item in a SOAP section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoapItem {
    pub id: Id,
    pub text: String,
    pub code: Option<String>, // ICD-10, CPT, etc.
    pub order: i32,
}

impl ClinicalNote {
    pub fn new(
        patient_id: Id,
        author_id: Id,
        note_type: NoteType,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Id::new_v4(),
            patient_id,
            author_id,
            encounter_id: None,
            note_type,
            content: String::new(),
            subjective: None,
            objective: None,
            assessment: None,
            plan: None,
            signature: None,
            co_signer_id: None,
            co_signature: None,
            status: NoteStatus::Draft,
            amends_note_id: None,
            created_at: now,
            updated_at: now,
            signed_at: None,
            version: VersionVector::new(),
            last_modified_by: None,
        }
    }

    /// Sign the note
    pub fn sign(&mut self, signer_id: Id, signature_data: String) -> Result<(), &'static str> {
        if self.status != NoteStatus::Draft {
            return Err("Can only sign draft notes");
        }
        
        let now = chrono::Utc::now();
        self.signature = Some(SignatureData {
            signature_data,
            signed_at: now,
            signer_id,
        });
        self.status = NoteStatus::Signed;
        self.signed_at = Some(now);
        self.updated_at = now;
        Ok(())
    }

    /// Add co-signature
    pub fn co_sign(&mut self, co_signer_id: Id, signature_data: String) -> Result<(), &'static str> {
        if self.status != NoteStatus::Signed {
            return Err("Note must be signed before co-signing");
        }
        
        let now = chrono::Utc::now();
        self.co_signer_id = Some(co_signer_id);
        self.co_signature = Some(SignatureData {
            signature_data,
            signed_at: now,
            signer_id: co_signer_id,
        });
        self.updated_at = now;
        Ok(())
    }

    /// Create an amendment
    pub fn amend(&self, author_id: Id) -> ClinicalNote {
        let mut amended = self.clone();
        amended.id = Id::new_v4();
        amended.amends_note_id = Some(self.id);
        amended.status = NoteStatus::Draft;
        amended.signature = None;
        amended.co_signature = None;
        amended.signed_at = None;
        amended.created_at = chrono::Utc::now();
        amended.updated_at = chrono::Utc::now();
        amended.author_id = author_id;
        amended
    }

    /// Void the note
    pub fn void(&mut self) -> Result<(), &'static str> {
        if self.status == NoteStatus::Voided {
            return Err("Note already voided");
        }
        self.status = NoteStatus::Voided;
        self.updated_at = chrono::Utc::now();
        Ok(())
    }

    pub fn is_signed(&self) -> bool {
        self.status == NoteStatus::Signed || self.status == NoteStatus::Amended
    }
}

/// Clinical note creation DTO
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateClinicalNote {
    pub patient_id: Id,
    pub encounter_id: Option<Id>,
    pub note_type: NoteType,
    pub content: Option<String>,
    pub subjective: Option<SoapSection>,
    pub objective: Option<SoapSection>,
    pub assessment: Option<SoapSection>,
    pub plan: Option<SoapSection>,
}

/// Clinical note update DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClinicalNote {
    pub content: Option<String>,
    pub subjective: Option<SoapSection>,
    pub objective: Option<SoapSection>,
    pub assessment: Option<SoapSection>,
    pub plan: Option<SoapSection>,
}
