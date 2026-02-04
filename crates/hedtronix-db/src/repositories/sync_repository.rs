//! Sync queue repository for offline-first operations

use rusqlite::{params, Row};
use hedtronix_core::{Id, VersionVector};
use hedtronix_core::crdt::{Change, ChangeOperation};
use crate::{Database, DbError, Result};

pub struct SyncRepository {
    db: Database,
}

impl SyncRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Add a change to the sync queue
    pub fn queue_change(&self, change: &Change) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let operation = match change.operation {
            ChangeOperation::Create => "CREATE",
            ChangeOperation::Update => "UPDATE",
            ChangeOperation::Delete => "DELETE",
        };

        conn.execute(
            r#"
            INSERT INTO sync_queue (
                id, entity_type, entity_id, operation, data_json,
                timestamp, device_id, version_json, synced
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0)
            "#,
            params![
                change.id.to_string(),
                change.entity_type,
                change.entity_id.to_string(),
                operation,
                change.data.to_string(),
                change.timestamp.to_rfc3339(),
                change.device_id,
                serde_json::to_string(&change.version).unwrap_or_default(),
            ],
        )?;

        Ok(())
    }

    /// Get pending (unsynced) changes
    pub fn get_pending_changes(&self, limit: u32) -> Result<Vec<Change>> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, entity_type, entity_id, operation, data_json,
                   timestamp, device_id, version_json
            FROM sync_queue
            WHERE synced = 0
            ORDER BY timestamp ASC
            LIMIT ?
            "#
        )?;

        let changes = stmt
            .query_map([limit], |row| {
                let id: String = row.get(0)?;
                let entity_type: String = row.get(1)?;
                let entity_id: String = row.get(2)?;
                let operation: String = row.get(3)?;
                let data_json: String = row.get(4)?;
                let timestamp: String = row.get(5)?;
                let device_id: String = row.get(6)?;
                let version_json: String = row.get(7)?;

                let op = match operation.as_str() {
                    "CREATE" => ChangeOperation::Create,
                    "UPDATE" => ChangeOperation::Update,
                    "DELETE" => ChangeOperation::Delete,
                    _ => ChangeOperation::Update,
                };

                Ok(Change {
                    id: Id::parse_str(&id).unwrap_or_else(|_| Id::new_v4()),
                    entity_type,
                    entity_id: Id::parse_str(&entity_id).unwrap_or_else(|_| Id::new_v4()),
                    operation: op,
                    data: serde_json::from_str(&data_json).unwrap_or(serde_json::Value::Null),
                    timestamp: chrono::DateTime::parse_from_rfc3339(&timestamp)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                    device_id,
                    version: serde_json::from_str(&version_json).unwrap_or_default(),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(changes)
    }

    /// Mark changes as synced
    pub fn mark_synced(&self, change_ids: &[Id]) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let now = chrono::Utc::now().to_rfc3339();
        for id in change_ids {
            conn.execute(
                "UPDATE sync_queue SET synced = 1, synced_at = ? WHERE id = ?",
                params![now, id.to_string()],
            )?;
        }

        Ok(())
    }

    /// Record a sync error
    pub fn record_sync_error(&self, change_id: Id, error: &str) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        conn.execute(
            "UPDATE sync_queue SET error_message = ?, retry_count = retry_count + 1 WHERE id = ?",
            params![error, change_id.to_string()],
        )?;

        Ok(())
    }

    /// Get sync metadata
    pub fn get_metadata(&self, key: &str) -> Result<Option<String>> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare("SELECT value FROM sync_metadata WHERE key = ?")?;
        let value: Option<String> = stmt.query_row([key], |row| row.get(0)).ok();
        Ok(value)
    }

    /// Set sync metadata
    pub fn set_metadata(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            r#"
            INSERT OR REPLACE INTO sync_metadata (key, value, updated_at)
            VALUES (?, ?, ?)
            "#,
            params![key, value, now],
        )?;

        Ok(())
    }

    /// Get the last sync timestamp
    pub fn get_last_sync_time(&self) -> Result<Option<chrono::DateTime<chrono::Utc>>> {
        let value = self.get_metadata("last_sync_time")?;
        Ok(value.and_then(|s| 
            chrono::DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .ok()
        ))
    }

    /// Set the last sync timestamp
    pub fn set_last_sync_time(&self, time: chrono::DateTime<chrono::Utc>) -> Result<()> {
        self.set_metadata("last_sync_time", &time.to_rfc3339())
    }

    /// Get pending sync count
    pub fn pending_count(&self) -> Result<i64> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare("SELECT COUNT(*) FROM sync_queue WHERE synced = 0")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    /// Clean up old synced changes (older than 7 days)
    pub fn cleanup_old_changes(&self) -> Result<usize> {
        let conn = self.db.connection();
        let conn = conn.lock().map_err(|e| DbError::Connection(e.to_string()))?;

        let cutoff = (chrono::Utc::now() - chrono::Duration::days(7)).to_rfc3339();
        let deleted = conn.execute(
            "DELETE FROM sync_queue WHERE synced = 1 AND synced_at < ?",
            [cutoff],
        )?;

        Ok(deleted)
    }
}
