//! Patient repository

use rusqlite::{params, Row};
use hedtronix_core::{Patient, CreatePatient, UpdatePatient, PatientSearchFilters, Gender, Id, Address, EmergencyContact, InsuranceInfo, Allergy, Medication, VersionVector};
use crate::{Database, DbError, Result};

pub struct PatientRepository {
    db: Database,
}

impl PatientRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn row_to_patient(row: &Row) -> rusqlite::Result<Patient> {
        let id: String = row.get(0)?;
        let mrn: String = row.get(1)?;
        let first_name: String = row.get(2)?;
        let last_name: String = row.get(3)?;
        let dob: String = row.get(4)?;
        let gender_str: String = row.get(5)?;
        let address_json: String = row.get(6)?;
        let phone: String = row.get(7)?;
        let email: Option<String> = row.get(8)?;
        let emergency_contact_json: String = row.get(9)?;
        let pcp_id: Option<String> = row.get(10)?;
        let insurance_json: String = row.get(11)?;
        let allergies_json: String = row.get(12)?;
        let medications_json: String = row.get(13)?;
        let problems_json: String = row.get(14)?;
        let active: i32 = row.get(15)?;
        let deceased: i32 = row.get(16)?;
        let deceased_at: Option<String> = row.get(17)?;
        let created_at: String = row.get(18)?;
        let updated_at: String = row.get(19)?;
        let version_json: String = row.get(20)?;
        let last_modified_by: Option<String> = row.get(21)?;

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
                patient.medical_record_number,
                patient.first_name,
                patient.last_name,
                patient.date_of_birth.format("%Y-%m-%d").to_string(),
                gender_str,
                serde_json::to_string(&patient.address).unwrap_or_default(),
                patient.phone,
                patient.email,
                serde_json::to_string(&patient.emergency_contact).unwrap_or_default(),
                patient.primary_care_physician_id.map(|id| id.to_string()),
                serde_json::to_string(&patient.insurance_info).unwrap_or_default(),
                serde_json::to_string(&patient.allergies).unwrap_or_default(),
                serde_json::to_string(&patient.medications).unwrap_or_default(),
                serde_json::to_string(&patient.problems).unwrap_or_default(),
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

        let patient = stmt.query_row([id.to_string()], Self::row_to_patient).ok();
        Ok(patient)
    }

    pub fn find_by_mrn(&self, mrn: &str) -> Result<Option<Patient>> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, medical_record_number, first_name, last_name, date_of_birth,
                   gender, address_json, phone, email, emergency_contact_json,
                   primary_care_physician_id, insurance_info_json, allergies_json,
                   medications_json, problems_json, active, deceased, deceased_at,
                   created_at, updated_at, version_json, last_modified_by
            FROM patients WHERE medical_record_number = ?
            "#
        )?;

        let patient = stmt.query_row([mrn], Self::row_to_patient).ok();
        Ok(patient)
    }

    pub fn search(&self, filters: &PatientSearchFilters) -> Result<Vec<Patient>> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

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

        if let Some(ref query) = filters.query {
            sql.push_str(&format!(
                " AND (first_name LIKE '%{}%' OR last_name LIKE '%{}%' OR medical_record_number LIKE '%{}%')",
                query, query, query
            ));
        }

        if let Some(physician_id) = filters.physician_id {
            sql.push_str(&format!(" AND primary_care_physician_id = '{}'", physician_id));
        }

        sql.push_str(&format!(
            " ORDER BY last_name, first_name LIMIT {} OFFSET {}",
            filters.limit.max(1).min(100),
            filters.page * filters.limit
        ));

        let mut stmt = conn.prepare(&sql)?;
        let patients = stmt
            .query_map([], Self::row_to_patient)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(patients)
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
                patient.first_name,
                patient.last_name,
                patient.date_of_birth.format("%Y-%m-%d").to_string(),
                gender_str,
                serde_json::to_string(&patient.address).unwrap_or_default(),
                patient.phone,
                patient.email,
                serde_json::to_string(&patient.emergency_contact).unwrap_or_default(),
                patient.primary_care_physician_id.map(|id| id.to_string()),
                serde_json::to_string(&patient.insurance_info).unwrap_or_default(),
                serde_json::to_string(&patient.allergies).unwrap_or_default(),
                serde_json::to_string(&patient.medications).unwrap_or_default(),
                serde_json::to_string(&patient.problems).unwrap_or_default(),
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
