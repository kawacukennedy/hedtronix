//! User repository

use rusqlite::{params, Row};
use hedtronix_core::{User, CreateUser, UpdateUser, UserRole, Id, VersionVector};
use crate::{Database, DbError, Result};

pub struct UserRepository {
    db: Database,
}

impl UserRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn row_to_user(row: &Row) -> rusqlite::Result<User> {
        let id: String = row.get(0)?;
        let email: String = row.get(1)?;
        let name: String = row.get(2)?;
        let role_str: String = row.get(3)?;
        let department_id: Option<String> = row.get(4)?;
        let license_number: Option<String> = row.get(5)?;
        let npi_number: Option<String> = row.get(6)?;
        let active: i32 = row.get(7)?;
        let created_at: String = row.get(8)?;
        let updated_at: String = row.get(9)?;
        let last_login_at: Option<String> = row.get(10)?;
        let password_hash: String = row.get(11)?;
        let version_json: String = row.get(12)?;
        let last_modified_by: Option<String> = row.get(13)?;

        let role = match role_str.as_str() {
            "PHYSICIAN" => UserRole::Physician,
            "NURSE" => UserRole::Nurse,
            "RECEPTIONIST" => UserRole::Receptionist,
            "BILLING" => UserRole::Billing,
            "ADMIN" => UserRole::Admin,
            "PATIENT" => UserRole::Patient,
            _ => UserRole::Patient,
        };

        Ok(User {
            id: Id::parse_str(&id).unwrap_or_else(|_| Id::new_v4()),
            email,
            name,
            role,
            department_id: department_id.and_then(|s| Id::parse_str(&s).ok()),
            license_number,
            npi_number,
            active: active == 1,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            last_login_at: last_login_at.and_then(|s| 
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            ),
            password_hash,
            version: serde_json::from_str(&version_json).unwrap_or_default(),
            last_modified_by,
        })
    }

    pub fn create(&self, user: &User) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        conn.execute(
            r#"
            INSERT INTO users (
                id, email, name, role, department_id, license_number, npi_number,
                active, created_at, updated_at, last_login_at, password_hash,
                version_json, last_modified_by
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                user.id.to_string(),
                user.email,
                user.name,
                user.role.as_str(),
                user.department_id.map(|id| id.to_string()),
                user.license_number,
                user.npi_number,
                if user.active { 1 } else { 0 },
                user.created_at.to_rfc3339(),
                user.updated_at.to_rfc3339(),
                user.last_login_at.map(|dt| dt.to_rfc3339()),
                user.password_hash,
                serde_json::to_string(&user.version).unwrap_or_default(),
                user.last_modified_by,
            ],
        )?;

        Ok(())
    }

    pub fn find_by_id(&self, id: Id) -> Result<Option<User>> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, email, name, role, department_id, license_number, npi_number,
                   active, created_at, updated_at, last_login_at, password_hash,
                   version_json, last_modified_by
            FROM users WHERE id = ?
            "#
        )?;

        let user = stmt.query_row([id.to_string()], Self::row_to_user).ok();
        Ok(user)
    }

    pub fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, email, name, role, department_id, license_number, npi_number,
                   active, created_at, updated_at, last_login_at, password_hash,
                   version_json, last_modified_by
            FROM users WHERE email = ?
            "#
        )?;

        let user = stmt.query_row([email], Self::row_to_user).ok();
        Ok(user)
    }

    pub fn find_all(&self, limit: u32, offset: u32) -> Result<Vec<User>> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, email, name, role, department_id, license_number, npi_number,
                   active, created_at, updated_at, last_login_at, password_hash,
                   version_json, last_modified_by
            FROM users
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#
        )?;

        let users = stmt
            .query_map([limit, offset], Self::row_to_user)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(users)
    }

    pub fn update(&self, user: &User) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        conn.execute(
            r#"
            UPDATE users SET
                email = ?, name = ?, role = ?, department_id = ?,
                license_number = ?, npi_number = ?, active = ?,
                updated_at = ?, last_login_at = ?,
                version_json = ?, last_modified_by = ?
            WHERE id = ?
            "#,
            params![
                user.email,
                user.name,
                user.role.as_str(),
                user.department_id.map(|id| id.to_string()),
                user.license_number,
                user.npi_number,
                if user.active { 1 } else { 0 },
                user.updated_at.to_rfc3339(),
                user.last_login_at.map(|dt| dt.to_rfc3339()),
                serde_json::to_string(&user.version).unwrap_or_default(),
                user.last_modified_by,
                user.id.to_string(),
            ],
        )?;

        Ok(())
    }

    pub fn delete(&self, id: Id) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        conn.execute("DELETE FROM users WHERE id = ?", [id.to_string()])?;
        Ok(())
    }

    pub fn count(&self) -> Result<i64> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare("SELECT COUNT(*) FROM users")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }
}
