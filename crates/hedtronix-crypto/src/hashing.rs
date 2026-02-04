//! Password hashing using Argon2

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use thiserror::Error;

/// Hashing error types
#[derive(Error, Debug)]
pub enum HashingError {
    #[error("Hash generation failed: {0}")]
    Generation(String),
    
    #[error("Hash verification failed")]
    Verification,
    
    #[error("Invalid hash format")]
    InvalidFormat,
}

/// Result type for hashing operations
pub type Result<T> = std::result::Result<T, HashingError>;

/// Hash a password using Argon2id
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| HashingError::Generation(e.to_string()))
}

/// Verify a password against a hash
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|_| HashingError::InvalidFormat)?;
    
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Simple SHA-256 hash for non-password data
pub fn sha256_hash(data: &[u8]) -> Vec<u8> {
    use ring::digest::{digest, SHA256};
    digest(&SHA256, data).as_ref().to_vec()
}

/// SHA-256 hash as hex string
pub fn sha256_hex(data: &[u8]) -> String {
    sha256_hash(data)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "secure_password_123";
        let hash = hash_password(password).unwrap();
        
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_sha256() {
        let data = b"test data";
        let hash = sha256_hex(data);
        assert_eq!(hash.len(), 64); // 32 bytes = 64 hex chars
    }
}
