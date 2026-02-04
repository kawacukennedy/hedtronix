-- HEDTRONIX Database Schema
-- SQLite with CRDT metadata support

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- ============================================================================
-- Core Tables
-- ============================================================================

-- Departments
CREATE TABLE IF NOT EXISTS departments (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    parent_id TEXT REFERENCES departments(id),
    manager_id TEXT,
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_departments_parent ON departments(parent_id);

-- Users
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('PHYSICIAN', 'NURSE', 'RECEPTIONIST', 'BILLING', 'ADMIN', 'PATIENT')),
    department_id TEXT REFERENCES departments(id),
    license_number TEXT,
    npi_number TEXT,
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_login_at TEXT,
    password_hash TEXT NOT NULL,
    version_json TEXT NOT NULL DEFAULT '{}',
    last_modified_by TEXT
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_department ON users(department_id);
CREATE INDEX idx_users_role ON users(role);

-- Devices
CREATE TABLE IF NOT EXISTS devices (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id),
    public_key TEXT NOT NULL,
    device_type TEXT NOT NULL CHECK (device_type IN ('DESKTOP', 'TABLET', 'MOBILE', 'KIOSK')),
    device_name TEXT,
    last_sync_at TEXT,
    ip_address TEXT,
    user_agent TEXT NOT NULL,
    revoked INTEGER NOT NULL DEFAULT 0,
    revoked_at TEXT,
    revoked_by TEXT REFERENCES users(id),
    created_at TEXT NOT NULL
);

CREATE INDEX idx_devices_user ON devices(user_id);
CREATE INDEX idx_devices_revoked ON devices(revoked);

-- Rooms
CREATE TABLE IF NOT EXISTS rooms (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    room_number TEXT NOT NULL,
    department_id TEXT REFERENCES departments(id),
    room_type TEXT NOT NULL,
    capacity INTEGER NOT NULL DEFAULT 1,
    equipment_json TEXT NOT NULL DEFAULT '[]',
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_rooms_department ON rooms(department_id);

-- ============================================================================
-- Patient Management
-- ============================================================================

-- Patients
CREATE TABLE IF NOT EXISTS patients (
    id TEXT PRIMARY KEY,
    medical_record_number TEXT NOT NULL UNIQUE,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    date_of_birth TEXT NOT NULL,
    gender TEXT NOT NULL CHECK (gender IN ('MALE', 'FEMALE', 'OTHER', 'UNKNOWN')),
    address_json TEXT NOT NULL DEFAULT '{}',
    phone TEXT NOT NULL DEFAULT '',
    email TEXT,
    emergency_contact_json TEXT NOT NULL DEFAULT '{}',
    primary_care_physician_id TEXT REFERENCES users(id),
    insurance_info_json TEXT NOT NULL DEFAULT '{}',
    allergies_json TEXT NOT NULL DEFAULT '[]',
    medications_json TEXT NOT NULL DEFAULT '[]',
    problems_json TEXT NOT NULL DEFAULT '[]',
    active INTEGER NOT NULL DEFAULT 1,
    deceased INTEGER NOT NULL DEFAULT 0,
    deceased_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version_json TEXT NOT NULL DEFAULT '{}',
    last_modified_by TEXT
);

CREATE INDEX idx_patients_mrn ON patients(medical_record_number);
CREATE INDEX idx_patients_name ON patients(last_name, first_name);
CREATE INDEX idx_patients_physician ON patients(primary_care_physician_id);

-- ============================================================================
-- Scheduling
-- ============================================================================

-- Appointments
CREATE TABLE IF NOT EXISTS appointments (
    id TEXT PRIMARY KEY,
    patient_id TEXT NOT NULL REFERENCES patients(id),
    provider_id TEXT NOT NULL REFERENCES users(id),
    room_id TEXT REFERENCES rooms(id),
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    duration INTEGER NOT NULL,
    appointment_type TEXT NOT NULL CHECK (appointment_type IN ('NEW_PATIENT', 'FOLLOW_UP', 'PROCEDURE', 'CONSULTATION', 'EMERGENCY')),
    status TEXT NOT NULL CHECK (status IN ('SCHEDULED', 'CHECKED_IN', 'IN_ROOM', 'COMPLETED', 'CANCELLED', 'NO_SHOW')),
    cancellation_reason TEXT,
    reason_for_visit TEXT NOT NULL,
    check_in_time TEXT,
    check_out_time TEXT,
    wait_time INTEGER,
    recurrence_rule_json TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    created_by TEXT NOT NULL REFERENCES users(id),
    version_json TEXT NOT NULL DEFAULT '{}',
    last_modified_by TEXT
);

CREATE INDEX idx_appointments_patient ON appointments(patient_id);
CREATE INDEX idx_appointments_provider ON appointments(provider_id);
CREATE INDEX idx_appointments_time ON appointments(start_time, end_time);
CREATE INDEX idx_appointments_status ON appointments(status);

-- ============================================================================
-- Clinical Documentation
-- ============================================================================

-- Encounters
CREATE TABLE IF NOT EXISTS encounters (
    id TEXT PRIMARY KEY,
    patient_id TEXT NOT NULL REFERENCES patients(id),
    provider_id TEXT NOT NULL REFERENCES users(id),
    appointment_id TEXT REFERENCES appointments(id),
    department_id TEXT REFERENCES departments(id),
    encounter_type TEXT NOT NULL CHECK (encounter_type IN ('OFFICE', 'INPATIENT', 'EMERGENCY', 'TELEHEALTH', 'HOME_VISIT')),
    status TEXT NOT NULL CHECK (status IN ('IN_PROGRESS', 'COMPLETED', 'CANCELLED')),
    start_time TEXT NOT NULL,
    end_time TEXT,
    chief_complaint TEXT,
    clinical_note_ids_json TEXT NOT NULL DEFAULT '[]',
    billing_entry_ids_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version_json TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_encounters_patient ON encounters(patient_id);
CREATE INDEX idx_encounters_provider ON encounters(provider_id);

-- Clinical Notes
CREATE TABLE IF NOT EXISTS clinical_notes (
    id TEXT PRIMARY KEY,
    patient_id TEXT NOT NULL REFERENCES patients(id),
    author_id TEXT NOT NULL REFERENCES users(id),
    encounter_id TEXT REFERENCES encounters(id),
    note_type TEXT NOT NULL CHECK (note_type IN ('PROGRESS_NOTE', 'CONSULTATION', 'DISCHARGE_SUMMARY', 'PROCEDURE_NOTE')),
    content TEXT NOT NULL DEFAULT '',
    subjective_json TEXT,
    objective_json TEXT,
    assessment_json TEXT,
    plan_json TEXT,
    signature_json TEXT,
    co_signer_id TEXT REFERENCES users(id),
    co_signature_json TEXT,
    status TEXT NOT NULL CHECK (status IN ('DRAFT', 'SIGNED', 'AMENDED', 'VOIDED')),
    amends_note_id TEXT REFERENCES clinical_notes(id),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    signed_at TEXT,
    version_json TEXT NOT NULL DEFAULT '{}',
    last_modified_by TEXT
);

CREATE INDEX idx_clinical_notes_patient ON clinical_notes(patient_id);
CREATE INDEX idx_clinical_notes_author ON clinical_notes(author_id);
CREATE INDEX idx_clinical_notes_encounter ON clinical_notes(encounter_id);

-- ============================================================================
-- Billing
-- ============================================================================

-- Billing Entries
CREATE TABLE IF NOT EXISTS billing_entries (
    id TEXT PRIMARY KEY,
    patient_id TEXT NOT NULL REFERENCES patients(id),
    encounter_id TEXT NOT NULL REFERENCES encounters(id),
    provider_id TEXT NOT NULL REFERENCES users(id),
    cpt_code TEXT NOT NULL,
    icd10_codes_json TEXT NOT NULL DEFAULT '[]',
    description TEXT NOT NULL,
    units INTEGER NOT NULL DEFAULT 1,
    unit_price TEXT NOT NULL,
    total_amount TEXT NOT NULL,
    insurance_estimated TEXT,
    patient_responsibility TEXT,
    status TEXT NOT NULL CHECK (status IN ('DRAFT', 'BILLED', 'SUBMITTED', 'PAID', 'DENIED', 'APPEALED')),
    submitted_at TEXT,
    paid_at TEXT,
    claim_number TEXT,
    denial_reason TEXT,
    adjustment_reason TEXT,
    adjustment_amount TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    created_by TEXT NOT NULL REFERENCES users(id),
    version_json TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_billing_patient ON billing_entries(patient_id);
CREATE INDEX idx_billing_encounter ON billing_entries(encounter_id);
CREATE INDEX idx_billing_status ON billing_entries(status);

-- ============================================================================
-- Audit & Sync
-- ============================================================================

-- Audit Logs (append-only)
CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL CHECK (event_type IN ('CREATE', 'READ', 'UPDATE', 'DELETE', 'LOGIN', 'LOGOUT', 'EXPORT', 'SYNC')),
    user_id TEXT REFERENCES users(id),
    device_id TEXT REFERENCES devices(id),
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    changes_json TEXT NOT NULL,
    ip_address TEXT,
    user_agent TEXT,
    timestamp TEXT NOT NULL,
    signature TEXT NOT NULL,
    previous_hash TEXT,
    hash TEXT NOT NULL
);

CREATE INDEX idx_audit_user ON audit_logs(user_id);
CREATE INDEX idx_audit_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_timestamp ON audit_logs(timestamp);

-- Sync Queue
CREATE TABLE IF NOT EXISTS sync_queue (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    operation TEXT NOT NULL CHECK (operation IN ('CREATE', 'UPDATE', 'DELETE')),
    data_json TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    device_id TEXT NOT NULL,
    version_json TEXT NOT NULL,
    synced INTEGER NOT NULL DEFAULT 0,
    synced_at TEXT,
    error_message TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_sync_queue_pending ON sync_queue(synced, timestamp);
CREATE INDEX idx_sync_queue_entity ON sync_queue(entity_type, entity_id);

-- Sync Metadata
CREATE TABLE IF NOT EXISTS sync_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Conflicts
CREATE TABLE IF NOT EXISTS conflicts (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    local_change_json TEXT NOT NULL,
    remote_change_json TEXT NOT NULL,
    resolved INTEGER NOT NULL DEFAULT 0,
    resolution_json TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_conflicts_unresolved ON conflicts(resolved, created_at);
