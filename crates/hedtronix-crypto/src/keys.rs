//! Key derivation and management

use ring::rand::{SecureRandom, SystemRandom};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use thiserror::Error;

/// Key management error types
#[derive(Error, Debug)]
pub enum KeyError {
    #[error("Key generation failed: {0}")]
    Generation(String),
    
    #[error("Key derivation failed: {0}")]
    Derivation(String),
    
    #[error("Invalid key")]
    Invalid,
}

/// Result type for key operations
pub type Result<T> = std::result::Result<T, KeyError>;

/// Generate a random key of the specified length
pub fn generate_random_bytes(length: usize) -> Result<Vec<u8>> {
    let rng = SystemRandom::new();
    let mut key = vec![0u8; length];
    rng.fill(&mut key)
        .map_err(|_| KeyError::Generation("Failed to generate random bytes".into()))?;
    Ok(key)
}

/// Generate a 256-bit (32 byte) encryption key
pub fn generate_encryption_key() -> Result<Vec<u8>> {
    generate_random_bytes(32)
}

/// Generate a base64-encoded key for storage
pub fn generate_encoded_key() -> Result<String> {
    let key = generate_encryption_key()?;
    Ok(BASE64.encode(&key))
}

/// Decode a base64-encoded key
pub fn decode_key(encoded: &str) -> Result<Vec<u8>> {
    BASE64.decode(encoded).map_err(|_| KeyError::Invalid)
}

/// Derive a key from a password using HKDF
pub fn derive_key_from_password(password: &str, salt: &[u8], key_length: usize) -> Result<Vec<u8>> {
    use ring::hkdf::{self, KeyType, Prk, Salt, HKDF_SHA256};
    
    let salt = Salt::new(HKDF_SHA256, salt);
    let prk = salt.extract(password.as_bytes());
    
    let mut output = vec![0u8; key_length];
    
    struct MyKeyType(usize);
    impl KeyType for MyKeyType {
        fn len(&self) -> usize { self.0 }
    }
    
    prk.expand(&[b"hedtronix-key"], MyKeyType(key_length))
        .map_err(|_| KeyError::Derivation("HKDF expansion failed".into()))?
        .fill(&mut output)
        .map_err(|_| KeyError::Derivation("Key fill failed".into()))?;
    
    Ok(output)
}

/// Per-device key derivation
pub fn derive_device_key(master_key: &[u8], device_id: &str) -> Result<Vec<u8>> {
    derive_key_from_password(
        &BASE64.encode(master_key),
        device_id.as_bytes(),
        32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key = generate_encryption_key().unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_encoded_key() {
        let encoded = generate_encoded_key().unwrap();
        let decoded = decode_key(&encoded).unwrap();
        assert_eq!(decoded.len(), 32);
    }

    #[test]
    fn test_key_derivation() {
        let password = "test_password";
        let salt = b"test_salt_value";
        let key = derive_key_from_password(password, salt, 32).unwrap();
        assert_eq!(key.len(), 32);
        
        // Same inputs should produce same output
        let key2 = derive_key_from_password(password, salt, 32).unwrap();
        assert_eq!(key, key2);
    }
}
