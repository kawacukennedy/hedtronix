//! Application state

use std::sync::Arc;
use hedtronix_db::Database;
use hedtronix_auth::AuthState;
use hedtronix_sync::SyncEngine;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub auth_state: AuthState,
    pub encryption_key: Vec<u8>,
    pub device_id: String,
}

impl AppState {
    pub fn new(db: Database, jwt_secret: Vec<u8>, encryption_key: Vec<u8>) -> Self {
        Self {
            db,
            auth_state: AuthState::new(jwt_secret),
            encryption_key,
            device_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn sync_engine(&self) -> SyncEngine {
        SyncEngine::new(self.db.clone(), self.device_id.clone())
    }
}
