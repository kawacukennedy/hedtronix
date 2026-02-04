//! Patient handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use hedtronix_core::{
    Patient, CreatePatient, UpdatePatient, PatientSearchFilters,
    Gender, Id, Allergy, Medication, AllergySeverity,
};
use hedtronix_db::PatientRepository;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;

/// List patients with pagination
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub active_only: Option<bool>,
}

pub async fn list_patients(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListPatientsResponse>, ApiError> {
    let repo = PatientRepository::new(state.db.clone());
    let filters = PatientSearchFilters {
        page: query.page.unwrap_or(0),
        limit: query.limit.unwrap_or(20).min(100),
        active_only: query.active_only.unwrap_or(true),
        ..Default::default()
    };
    
    let patients = repo.search(&filters)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    let total = repo.count()
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    Ok(Json(ListPatientsResponse {
        patients: patients.into_iter().map(PatientDto::from).collect(),
        total,
        page: filters.page,
        limit: filters.limit,
    }))
}

#[derive(Debug, Serialize)]
pub struct ListPatientsResponse {
    pub patients: Vec<PatientDto>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
}

/// Get patient by ID
pub async fn get_patient(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PatientDto>, ApiError> {
    let patient_id = Id::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid patient ID"))?;
    
    let repo = PatientRepository::new(state.db.clone());
    let patient = repo.find_by_id(patient_id)
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Patient"))?;
    
    Ok(Json(PatientDto::from(patient)))
}

/// Create new patient
pub async fn create_patient(
    State(state): State<AppState>,
    Json(req): Json<CreatePatientRequest>,
) -> Result<Json<PatientDto>, ApiError> {
    let gender = parse_gender(&req.gender)?;
    let dob = chrono::NaiveDate::parse_from_str(&req.date_of_birth, "%Y-%m-%d")
        .map_err(|_| ApiError::bad_request("Invalid date format, use YYYY-MM-DD"))?;
    
    let repo = PatientRepository::new(state.db.clone());
    let mrn = repo.generate_mrn()
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    let patient = Patient::new(mrn, req.first_name, req.last_name, dob, gender);
    
    repo.create(&patient)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    // Track for sync
    let sync_engine = state.sync_engine();
    let _ = sync_engine.track_create(
        "Patient",
        patient.id,
        serde_json::to_value(&patient).unwrap_or_default(),
    );
    
    Ok(Json(PatientDto::from(patient)))
}

#[derive(Debug, Deserialize)]
pub struct CreatePatientRequest {
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: String,
    pub gender: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

/// Update patient
pub async fn update_patient(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdatePatientRequest>,
) -> Result<Json<PatientDto>, ApiError> {
    let patient_id = Id::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid patient ID"))?;
    
    let repo = PatientRepository::new(state.db.clone());
    let mut patient = repo.find_by_id(patient_id)
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Patient"))?;
    
    // Update fields
    if let Some(first_name) = req.first_name {
        patient.first_name = first_name;
    }
    if let Some(last_name) = req.last_name {
        patient.last_name = last_name;
    }
    if let Some(phone) = req.phone {
        patient.phone = phone;
    }
    if let Some(email) = req.email {
        patient.email = Some(email);
    }
    patient.updated_at = chrono::Utc::now();
    
    repo.update(&patient)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    // Track for sync
    let sync_engine = state.sync_engine();
    let _ = sync_engine.track_update(
        "Patient",
        patient.id,
        serde_json::to_value(&patient).unwrap_or_default(),
    );
    
    Ok(Json(PatientDto::from(patient)))
}

#[derive(Debug, Deserialize)]
pub struct UpdatePatientRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

/// Delete patient (soft delete)
pub async fn delete_patient(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DeleteResponse>, ApiError> {
    let patient_id = Id::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid patient ID"))?;
    
    let repo = PatientRepository::new(state.db.clone());
    let mut patient = repo.find_by_id(patient_id)
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Patient"))?;
    
    patient.active = false;
    patient.updated_at = chrono::Utc::now();
    
    repo.update(&patient)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    // Track for sync
    let sync_engine = state.sync_engine();
    let _ = sync_engine.track_delete("Patient", patient.id);
    
    Ok(Json(DeleteResponse { success: true }))
}

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub success: bool,
}

/// Search patients
pub async fn search_patients(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<ListPatientsResponse>, ApiError> {
    let repo = PatientRepository::new(state.db.clone());
    let filters = PatientSearchFilters {
        query: Some(req.query),
        page: req.page.unwrap_or(0),
        limit: req.limit.unwrap_or(20).min(100),
        active_only: req.active_only.unwrap_or(true),
        ..Default::default()
    };
    
    let patients = repo.search(&filters)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    Ok(Json(ListPatientsResponse {
        patients: patients.into_iter().map(PatientDto::from).collect(),
        total: 0, // Would need count query
        page: filters.page,
        limit: filters.limit,
    }))
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub active_only: Option<bool>,
}

/// Add allergy to patient
pub async fn add_allergy(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<AddAllergyRequest>,
) -> Result<Json<PatientDto>, ApiError> {
    let patient_id = Id::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid patient ID"))?;
    
    let repo = PatientRepository::new(state.db.clone());
    let mut patient = repo.find_by_id(patient_id)
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Patient"))?;
    
    let severity = match req.severity.to_uppercase().as_str() {
        "MILD" => AllergySeverity::Mild,
        "MODERATE" => AllergySeverity::Moderate,
        "SEVERE" => AllergySeverity::Severe,
        "LIFE_THREATENING" => AllergySeverity::LifeThreatening,
        _ => AllergySeverity::Moderate,
    };
    
    let allergy = Allergy {
        id: Id::new_v4(),
        name: req.name,
        severity,
        reaction: req.reaction,
        onset_date: req.onset_date,
        created_at: chrono::Utc::now(),
    };
    
    patient.add_allergy(allergy);
    
    repo.update(&patient)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    Ok(Json(PatientDto::from(patient)))
}

#[derive(Debug, Deserialize)]
pub struct AddAllergyRequest {
    pub name: String,
    pub severity: String,
    pub reaction: Option<String>,
    pub onset_date: Option<String>,
}

/// Add medication to patient
pub async fn add_medication(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<AddMedicationRequest>,
) -> Result<Json<PatientDto>, ApiError> {
    let patient_id = Id::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid patient ID"))?;
    
    let repo = PatientRepository::new(state.db.clone());
    let mut patient = repo.find_by_id(patient_id)
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Patient"))?;
    
    let medication = Medication {
        id: Id::new_v4(),
        name: req.name,
        dosage: req.dosage,
        frequency: req.frequency,
        start_date: req.start_date,
        end_date: None,
        prescriber_id: None,
        active: true,
    };
    
    patient.add_medication(medication);
    
    repo.update(&patient)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    Ok(Json(PatientDto::from(patient)))
}

#[derive(Debug, Deserialize)]
pub struct AddMedicationRequest {
    pub name: String,
    pub dosage: String,
    pub frequency: String,
    pub start_date: Option<String>,
}

// Helper functions
fn parse_gender(s: &str) -> Result<Gender, ApiError> {
    match s.to_uppercase().as_str() {
        "MALE" | "M" => Ok(Gender::Male),
        "FEMALE" | "F" => Ok(Gender::Female),
        "OTHER" | "O" => Ok(Gender::Other),
        "UNKNOWN" | "U" => Ok(Gender::Unknown),
        _ => Err(ApiError::bad_request("Invalid gender")),
    }
}

/// Patient DTO for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct PatientDto {
    pub id: String,
    pub medical_record_number: String,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: String,
    pub gender: String,
    pub phone: String,
    pub email: Option<String>,
    pub allergies: Vec<AllergyDto>,
    pub medications: Vec<MedicationDto>,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Patient> for PatientDto {
    fn from(p: Patient) -> Self {
        Self {
            id: p.id.to_string(),
            medical_record_number: p.medical_record_number,
            first_name: p.first_name,
            last_name: p.last_name,
            date_of_birth: p.date_of_birth.format("%Y-%m-%d").to_string(),
            gender: format!("{:?}", p.gender).to_uppercase(),
            phone: p.phone,
            email: p.email,
            allergies: p.allergies.into_iter().map(AllergyDto::from).collect(),
            medications: p.medications.into_iter().map(MedicationDto::from).collect(),
            active: p.active,
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllergyDto {
    pub id: String,
    pub name: String,
    pub severity: String,
    pub reaction: Option<String>,
}

impl From<Allergy> for AllergyDto {
    fn from(a: Allergy) -> Self {
        Self {
            id: a.id.to_string(),
            name: a.name,
            severity: format!("{:?}", a.severity).to_uppercase(),
            reaction: a.reaction,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MedicationDto {
    pub id: String,
    pub name: String,
    pub dosage: String,
    pub frequency: String,
    pub active: bool,
}

impl From<Medication> for MedicationDto {
    fn from(m: Medication) -> Self {
        Self {
            id: m.id.to_string(),
            name: m.name,
            dosage: m.dosage,
            frequency: m.frequency,
            active: m.active,
        }
    }
}
