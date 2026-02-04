//! Database connection management

use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Database error types
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Query error: {0}")]
    Query(String),
    
    #[error("Migration error: {0}")]
    Migration(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// Result type for database operations
pub type Result<T> = std::result::Result<T, DbError>;

/// Database connection wrapper
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    initialized: bool,
}

impl Database {
    /// Open or create a database at the specified path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        
        // Enable foreign keys
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            initialized: false,
        })
    }

    /// Create an in-memory database (for testing)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        
        // Enable foreign keys
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            initialized: false,
        })
    }

    /// Initialize the database with the schema
    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        let schema = include_str!("schema.sql");
        let conn = self.conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;
        conn.execute_batch(schema)?;
        
        drop(conn);
        self.initialized = true;
        Ok(())
    }

    /// Get a connection for executing queries
    pub fn connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }

    /// Execute a query that doesn't return rows
    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<usize> {
        let conn = self.conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;
        let changed = conn.execute(sql, params)?;
        Ok(changed)
    }

    /// Execute a query and return the last inserted rowid
    pub fn insert(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<i64> {
        let conn = self.conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;
        conn.execute(sql, params)?;
        Ok(conn.last_insert_rowid())
    }

    /// Check if a table exists
    pub fn table_exists(&self, table_name: &str) -> Result<bool> {
        let conn = self.conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name=?"
        )?;
        let exists = stmt.exists([table_name])?;
        Ok(exists)
    }

    /// Get database statistics
    pub fn stats(&self) -> Result<DatabaseStats> {
        let conn = self.conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;
        
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM users")?;
        let user_count: i64 = stmt.query_row([], |row| row.get(0)).unwrap_or(0);
        
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM patients")?;
        let patient_count: i64 = stmt.query_row([], |row| row.get(0)).unwrap_or(0);
        
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM appointments")?;
        let appointment_count: i64 = stmt.query_row([], |row| row.get(0)).unwrap_or(0);
        
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM sync_queue WHERE synced = 0")?;
        let pending_sync: i64 = stmt.query_row([], |row| row.get(0)).unwrap_or(0);
        
        Ok(DatabaseStats {
            user_count,
            patient_count,
            appointment_count,
            pending_sync,
        })
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
            initialized: self.initialized,
        }
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub user_count: i64,
    pub patient_count: i64,
    pub appointment_count: i64,
    pub pending_sync: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory() {
        let mut db = Database::in_memory().unwrap();
        db.initialize().unwrap();
        assert!(db.table_exists("users").unwrap());
    }

    #[test]
    fn test_stats() {
        let mut db = Database::in_memory().unwrap();
        db.initialize().unwrap();
        let stats = db.stats().unwrap();
        assert_eq!(stats.user_count, 0);
    }
}
