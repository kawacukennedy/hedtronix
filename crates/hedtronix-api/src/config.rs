//! Server configuration

use serde::{Deserialize, Serialize};

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Address to bind to (e.g., "0.0.0.0:8080")
    pub bind_address: String,
    
    /// Path to SQLite database file
    pub database_path: String,
    
    /// JWT secret (32 bytes)
    pub jwt_secret: Vec<u8>,
    
    /// Log level
    pub log_level: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:8080".to_string(),
            database_path: "./hedtronix.db".to_string(),
            jwt_secret: vec![0u8; 32], // Should be generated or loaded from env
            log_level: "info".to_string(),
        }
    }
}

impl ServerConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let bind_address = std::env::var("BIND_ADDRESS")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        
        let database_path = std::env::var("DATABASE_PATH")
            .unwrap_or_else(|_| "./hedtronix.db".to_string());
        
        let jwt_secret = std::env::var("JWT_SECRET")
            .map(|s| s.into_bytes())
            .unwrap_or_else(|_| {
                // Generate a random secret if not provided
                use hedtronix_crypto::keys::generate_encryption_key;
                generate_encryption_key().unwrap_or_else(|_| vec![0u8; 32])
            });
        
        let log_level = std::env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string());
        
        Self {
            bind_address,
            database_path,
            jwt_secret,
            log_level,
        }
    }
}
