//! Database migrations

use crate::{Database, DbError, Result};

/// Run all migrations
pub fn run_migrations(db: &mut Database) -> Result<()> {
    db.initialize()?;
    
    // Add any additional migrations here
    // For now, the schema.sql contains the initial migration
    
    Ok(())
}

/// Check if migrations are up to date
pub fn check_migrations(db: &Database) -> Result<bool> {
    db.table_exists("users")
}
