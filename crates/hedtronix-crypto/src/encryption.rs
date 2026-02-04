//! AES-256-GCM encryption for sensitive data

use ring::aead::{self, Aad, BoundKey, Nonce, NonceSequence, SealingKey, OpeningKey, UnboundKey};
use ring::rand::{SecureRandom, SystemRandom};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use thiserror::Error;

/// Encryption error types
#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Key generation failed: {0}")]
    KeyGeneration(String),
    
    #[error("Encryption failed: {0}")]
    Encryption(String),
    
    #[error("Decryption failed: {0}")]
    Decryption(String),
    
    #[error("Invalid key length")]
    InvalidKeyLength,
    
    #[error("Invalid data format")]
    InvalidFormat,
}

/// Result type for encryption operations
pub type Result<T> = std::result::Result<T, EncryptionError>;

/// Counter-based nonce sequence for AES-GCM
struct CounterNonceSequence {
    counter: u64,
    prefix: [u8; 4],
}

impl CounterNonceSequence {
    fn new(prefix: [u8; 4]) -> Self {
        Self { counter: 0, prefix }
    }
}

impl NonceSequence for CounterNonceSequence {
    fn advance(&mut self) -> std::result::Result<Nonce, ring::error::Unspecified> {
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[..4].copy_from_slice(&self.prefix);
        nonce_bytes[4..].copy_from_slice(&self.counter.to_be_bytes());
        self.counter += 1;
        Nonce::try_assume_unique_for_key(&nonce_bytes)
    }
}

/// AES-256-GCM encryptor for field-level encryption
pub struct Encryptor {
    key: Vec<u8>,
    rng: SystemRandom,
}

impl Encryptor {
    /// Create a new encryptor with a 256-bit key
    pub fn new(key: &[u8]) -> Result<Self> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength);
        }
        Ok(Self {
            key: key.to_vec(),
            rng: SystemRandom::new(),
        })
    }

    /// Generate a new random 256-bit key
    pub fn generate_key() -> Result<Vec<u8>> {
        let rng = SystemRandom::new();
        let mut key = vec![0u8; 32];
        rng.fill(&mut key)
            .map_err(|_| EncryptionError::KeyGeneration("Failed to generate random key".into()))?;
        Ok(key)
    }

    /// Encrypt plaintext and return base64-encoded ciphertext
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let mut nonce_bytes = [0u8; 12];
        self.rng.fill(&mut nonce_bytes)
            .map_err(|_| EncryptionError::Encryption("Failed to generate nonce".into()))?;

        let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &self.key)
            .map_err(|_| EncryptionError::Encryption("Failed to create key".into()))?;

        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);
        
        let mut in_out = plaintext.as_bytes().to_vec();
        
        // Use seal_in_place_append_tag for simpler one-shot encryption
        let algorithm = &aead::AES_256_GCM;
        let key = aead::LessSafeKey::new(
            aead::UnboundKey::new(algorithm, &self.key)
                .map_err(|_| EncryptionError::Encryption("Failed to create key".into()))?
        );
        
        key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| EncryptionError::Encryption("Encryption failed".into()))?;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend(in_out);

        Ok(BASE64.encode(&result))
    }

    /// Decrypt base64-encoded ciphertext
    pub fn decrypt(&self, ciphertext: &str) -> Result<String> {
        let data = BASE64.decode(ciphertext)
            .map_err(|_| EncryptionError::InvalidFormat)?;

        if data.len() < 12 {
            return Err(EncryptionError::InvalidFormat);
        }

        let nonce_bytes: [u8; 12] = data[..12].try_into()
            .map_err(|_| EncryptionError::InvalidFormat)?;
        let mut ciphertext_with_tag = data[12..].to_vec();

        let algorithm = &aead::AES_256_GCM;
        let key = aead::LessSafeKey::new(
            aead::UnboundKey::new(algorithm, &self.key)
                .map_err(|_| EncryptionError::Decryption("Failed to create key".into()))?
        );

        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);
        
        let plaintext = key.open_in_place(nonce, Aad::empty(), &mut ciphertext_with_tag)
            .map_err(|_| EncryptionError::Decryption("Decryption failed".into()))?;

        String::from_utf8(plaintext.to_vec())
            .map_err(|_| EncryptionError::Decryption("Invalid UTF-8".into()))
    }
}

/// Encrypt a field value with the given key
pub fn encrypt_field(plaintext: &str, key: &[u8]) -> Result<String> {
    let encryptor = Encryptor::new(key)?;
    encryptor.encrypt(plaintext)
}

/// Decrypt a field value with the given key
pub fn decrypt_field(ciphertext: &str, key: &[u8]) -> Result<String> {
    let encryptor = Encryptor::new(key)?;
    encryptor.decrypt(ciphertext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = Encryptor::generate_key().unwrap();
        let encryptor = Encryptor::new(&key).unwrap();
        
        let plaintext = "Sensitive patient data";
        let ciphertext = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_invalid_key_length() {
        let result = Encryptor::new(&[0u8; 16]);
        assert!(result.is_err());
    }
}
