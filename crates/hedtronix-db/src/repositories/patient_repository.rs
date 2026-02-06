//! Patient repository

use rusqlite::{params, Row};
use hedtronix_core::{Patient, CreatePatient, UpdatePatient, PatientSearchFilters, Gender, Id, Address, EmergencyContact, InsuranceInfo, Allergy, Medication, VersionVector};
use crate::{Database, DbError, Result};
use hedtronix_crypto::{encrypt_field, decrypt_field};

pub struct PatientRepository {
    db: Database,
    encryption_key: Vec<u8>,
}

impl PatientRepository {
    pub fn new(db: Database, encryption_key: Vec<u8>) -> Self {
        Self { db, encryption_key }
    }

    fn row_to_patient(row: &Row, key: &[u8]) -> rusqlite::Result<Patient> {
        let id: String = row.get(0)?;
        let mrn_enc: String = row.get(1)?;
        let first_name_enc: String = row.get(2)?;
        let last_name_enc: String = row.get(3)?;
        let dob_enc: String = row.get(4)?;
        let gender_str: String = row.get(5)?;
        let address_json_enc: String = row.get(6)?;
        let phone_enc: String = row.get(7)?;
        let email_enc: Option<String> = row.get(8)?;
        let emergency_contact_json_enc: String = row.get(9)?;
        let pcp_id: Option<String> = row.get(10)?;
        let insurance_json_enc: String = row.get(11)?;
        let allergies_json_enc: String = row.get(12)?;
        let medications_json_enc: String = row.get(13)?;
        let problems_json_enc: String = row.get(14)?;
        let active: i32 = row.get(15)?;
        let deceased: i32 = row.get(16)?;
        let deceased_at: Option<String> = row.get(17)?;
        let created_at: String = row.get(18)?;
        let updated_at: String = row.get(19)?;
        let version_json: String = row.get(20)?;
        let last_modified_by: Option<String> = row.get(21)?;

        // Decrypt helper closure
        let decrypt = |s: &str| -> rusqlite::Result<String> {
             if s.is_empty() { return Ok(String::new()); }
             decrypt_field(s, key).map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                 0, 
                 rusqlite::types::Type::Text, 
                 Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
             ))
        };

        let mrn = decrypt(&mrn_enc)?;
        let first_name = decrypt(&first_name_enc)?;
        let last_name = decrypt(&last_name_enc)?;
        let dob = decrypt(&dob_enc)?;
        let address_json = decrypt(&address_json_enc)?;
        let phone = decrypt(&phone_enc)?;
        
        let email = match email_enc {
            Some(e) => Some(decrypt(&e)?),
            None => None,
        };
        
        let emergency_contact_json = decrypt(&emergency_contact_json_enc)?;
        let insurance_json = decrypt(&insurance_json_enc)?;
        let allergies_json = decrypt(&allergies_json_enc)?;
        let medications_json = decrypt(&medications_json_enc)?;
        let problems_json = decrypt(&problems_json_enc)?;

        let gender = match gender_str.as_str() {
            "MALE" => Gender::Male,
            "FEMALE" => Gender::Female,
            "OTHER" => Gender::Other,
            _ => Gender::Unknown,
        };

        Ok(Patient {
            id: Id::parse_str(&id).unwrap_or_else(|_| Id::new_v4()),
            medical_record_number: mrn,
            first_name,
            last_name,
            date_of_birth: chrono::NaiveDate::parse_from_str(&dob, "%Y-%m-%d")
                .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(1900, 1, 1).unwrap()),
            gender,
            address: serde_json::from_str(&address_json).unwrap_or_default(),
            phone,
            email,
            emergency_contact: serde_json::from_str(&emergency_contact_json).unwrap_or_default(),
            primary_care_physician_id: pcp_id.and_then(|s| Id::parse_str(&s).ok()),
            insurance_info: serde_json::from_str(&insurance_json).unwrap_or_default(),
            allergies: serde_json::from_str(&allergies_json).unwrap_or_default(),
            medications: serde_json::from_str(&medications_json).unwrap_or_default(),
            problems: serde_json::from_str(&problems_json).unwrap_or_default(),
            active: active == 1,
            deceased: deceased == 1,
            deceased_at: deceased_at.and_then(|s| 
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            ),
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            version: serde_json::from_str(&version_json).unwrap_or_default(),
            last_modified_by,
        })
    }

    pub fn create(&self, patient: &Patient) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let gender_str = match patient.gender {
            Gender::Male => "MALE",
            Gender::Female => "FEMALE",
            Gender::Other => "OTHER",
            Gender::Unknown => "UNKNOWN",
        };

        // Encrypt sensitive fields
        let key = &self.encryption_key;
        let encrypt = |s: &str| -> Result<String> {
            encrypt_field(s, key).map_err(|e| DbError::Serialization(format!("Encryption failed: {}", e)))
        };

        let mrn_enc = encrypt(&patient.medical_record_number)?;
        let first_name_enc = encrypt(&patient.first_name)?;
        let last_name_enc = encrypt(&patient.last_name)?;
        let dob_enc = encrypt(&patient.date_of_birth.format("%Y-%m-%d").to_string())?;
        let address_enc = encrypt(&serde_json::to_string(&patient.address).unwrap_or_default())?;
        let phone_enc = encrypt(&patient.phone)?;
        
        let email_enc = match &patient.email {
            Some(e) => Some(encrypt(e)?),
            None => None,
        };
        
        let emergency_enc = encrypt(&serde_json::to_string(&patient.emergency_contact).unwrap_or_default())?;
        let insurance_enc = encrypt(&serde_json::to_string(&patient.insurance_info).unwrap_or_default())?;
        let allergies_enc = encrypt(&serde_json::to_string(&patient.allergies).unwrap_or_default())?;
        let medications_enc = encrypt(&serde_json::to_string(&patient.medications).unwrap_or_default())?;
        let problems_enc = encrypt(&serde_json::to_string(&patient.problems).unwrap_or_default())?;

        conn.execute(
            r#"
            INSERT INTO patients (
                id, medical_record_number, first_name, last_name, date_of_birth,
                gender, address_json, phone, email, emergency_contact_json,
                primary_care_physician_id, insurance_info_json, allergies_json,
                medications_json, problems_json, active, deceased, deceased_at,
                created_at, updated_at, version_json, last_modified_by
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                patient.id.to_string(),
                mrn_enc,
                first_name_enc,
                last_name_enc,
                dob_enc,
                gender_str,
                address_enc,
                phone_enc,
                email_enc,
                emergency_enc,
                patient.primary_care_physician_id.map(|id| id.to_string()),
                insurance_enc,
                allergies_enc,
                medications_enc,
                problems_enc,
                if patient.active { 1 } else { 0 },
                if patient.deceased { 1 } else { 0 },
                patient.deceased_at.map(|dt| dt.to_rfc3339()),
                patient.created_at.to_rfc3339(),
                patient.updated_at.to_rfc3339(),
                serde_json::to_string(&patient.version).unwrap_or_default(),
                patient.last_modified_by.clone(),
            ],
        )?;

        Ok(())
    }

    pub fn find_by_id(&self, id: Id) -> Result<Option<Patient>> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, medical_record_number, first_name, last_name, date_of_birth,
                   gender, address_json, phone, email, emergency_contact_json,
                   primary_care_physician_id, insurance_info_json, allergies_json,
                   medications_json, problems_json, active, deceased, deceased_at,
                   created_at, updated_at, version_json, last_modified_by
            FROM patients WHERE id = ?
            "#
        )?;

        let key = &self.encryption_key;
        let patient = stmt.query_row([id.to_string()], |row| Self::row_to_patient(row, key)).ok();
        Ok(patient)
    }

    pub fn find_by_mrn(&self, mrn: &str) -> Result<Option<Patient>> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        // Encryption prevents SQL lookup. Scan all active patients.
        // Optimization: In a real system, use a deterministic hash for lookup.
        let mut stmt = conn.prepare(
            r#"
            SELECT id, medical_record_number, first_name, last_name, date_of_birth,
                   gender, address_json, phone, email, emergency_contact_json,
                   primary_care_physician_id, insurance_info_json, allergies_json,
                   medications_json, problems_json, active, deceased, deceased_at,
                   created_at, updated_at, version_json, last_modified_by
            FROM patients
            "#
        )?;

        let key = &self.encryption_key;
        let mut rows = stmt.query([])?;
        
        while let Some(row) = rows.next()? {
            match Self::row_to_patient(row, key) {
                Ok(patient) => {
                    if patient.medical_record_number == mrn {
                        return Ok(Some(patient));
                    }
                },
                Err(_) => continue, // Skip malformed/decryption failure
            }
        }
        
        Ok(None)
    }

    pub fn search(&self, filters: &PatientSearchFilters) -> Result<Vec<Patient>> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        // Build base SQL for non-encrypted fields
        let mut sql = r#"
            SELECT id, medical_record_number, first_name, last_name, date_of_birth,
                   gender, address_json, phone, email, emergency_contact_json,
                   primary_care_physician_id, insurance_info_json, allergies_json,
                   medications_json, problems_json, active, deceased, deceased_at,
                   created_at, updated_at, version_json, last_modified_by
            FROM patients WHERE 1=1
        "#.to_string();

        if filters.active_only {
            sql.push_str(" AND active = 1");
        }

        if let Some(physician_id) = filters.physician_id {
            sql.push_str(&format!(" AND primary_care_physician_id = '{}'", physician_id));
        }

        // We must fetch ALL matching the base criteria, then decrypt, filter, sort, paginate in memory
        let mut stmt = conn.prepare(&sql)?;
        let key = &self.encryption_key;
        
        let mut patients: Vec<Patient> = stmt
            .query_map([], |row| Self::row_to_patient(row, key))?
            .filter_map(|r| r.ok())
            .collect();

        // In-memory filtering
        if let Some(ref query) = filters.query {
            let q = query.to_lowercase();
            patients.retain(|p| {
                p.first_name.to_lowercase().contains(&q) ||
                p.last_name.to_lowercase().contains(&q) ||
                p.medical_record_number.to_lowercase().contains(&q)
            });
        }

        // In-memory sorting (Last Name, First Name)
        patients.sort_by(|a, b| {
            a.last_name.cmp(&b.last_name)
                .then(a.first_name.cmp(&b.first_name))
        });

        // Pagination
        let total = patients.len();
        let offset = (filters.page * filters.limit) as usize;
        let limit = filters.limit as usize;
        
        if offset >= total {
            return Ok(Vec::new());
        }

        let end = std::cmp::min(offset + limit, total);
        Ok(patients[offset..end].to_vec())
    }

    pub fn update(&self, patient: &Patient) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let gender_str = match patient.gender {
            Gender::Male => "MALE",
            Gender::Female => "FEMALE",
            Gender::Other => "OTHER",
            Gender::Unknown => "UNKNOWN",
        };

        // Encrypt sensitive fields
        let key = &self.encryption_key;
        let encrypt = |s: &str| -> Result<String> {
            encrypt_field(s, key).map_err(|e| DbError::Serialization(format!("Encryption failed: {}", e)))
        };

        // Note: MRN is usually immutable but if allowed to change it should be encrypted too
        // The UPDATE statement below doesn't update MRN? 
        // Checking original code: "UPDATE patients SET first_name = ? ..."
        // MRN is NOT in the update list in original code. Good.

        let first_name_enc = encrypt(&patient.first_name)?;
        let last_name_enc = encrypt(&patient.last_name)?;
        let dob_enc = encrypt(&patient.date_of_birth.format("%Y-%m-%d").to_string())?;
        let address_enc = encrypt(&serde_json::to_string(&patient.address).unwrap_or_default())?;
        let phone_enc = encrypt(&patient.phone)?;
        
        let email_enc = match &patient.email {
            Some(e) => Some(encrypt(e)?),
            None => None,
        };
        
        let emergency_enc = encrypt(&serde_json::to_string(&patient.emergency_contact).unwrap_or_default())?;
        let insurance_enc = encrypt(&serde_json::to_string(&patient.insurance_info).unwrap_or_default())?;
        let allergies_enc = encrypt(&serde_json::to_string(&patient.allergies).unwrap_or_default())?;
        let medications_enc = encrypt(&serde_json::to_string(&patient.medications).unwrap_or_default())?;
        let problems_enc = encrypt(&serde_json::to_string(&patient.problems).unwrap_or_default())?;

        conn.execute(
            r#"
            UPDATE patients SET
                first_name = ?, last_name = ?, date_of_birth = ?,
                gender = ?, address_json = ?, phone = ?, email = ?,
                emergency_contact_json = ?, primary_care_physician_id = ?,
                insurance_info_json = ?, allergies_json = ?, medications_json = ?,
                problems_json = ?, active = ?, deceased = ?, deceased_at = ?,
                updated_at = ?, version_json = ?, last_modified_by = ?
            WHERE id = ?
            "#,
            params![
                first_name_enc,
                last_name_enc,
                dob_enc,
                gender_str,
                address_enc,
                phone_enc,
                email_enc,
                emergency_enc,
                patient.primary_care_physician_id.map(|id| id.to_string()),
                insurance_enc,
                allergies_enc,
                medications_enc,
                problems_enc,
                if patient.active { 1 } else { 0 },
                if patient.deceased { 1 } else { 0 },
                patient.deceased_at.map(|dt| dt.to_rfc3339()),
                patient.updated_at.to_rfc3339(),
                serde_json::to_string(&patient.version).unwrap_or_default(),
                patient.last_modified_by.clone(),
                patient.id.to_string(),
            ],
        )?;

        Ok(())
    }

    pub fn delete(&self, id: Id) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        conn.execute("DELETE FROM patients WHERE id = ?", [id.to_string()])?;
        Ok(())
    }

    pub fn count(&self) -> Result<i64> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare("SELECT COUNT(*) FROM patients")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    /// Generate a unique medical record number
    pub fn generate_mrn(&self) -> Result<String> {
        let count = self.count()? + 1;
        Ok(format!("MRN{:08}", count))
    }
}
