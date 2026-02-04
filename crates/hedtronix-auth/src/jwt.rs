//! JWT token management with offline support

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use hedtronix_core::{Id, UserRole};

/// JWT error types
#[derive(Error, Debug)]
pub enum JwtError {
    #[error("Token creation failed: {0}")]
    Creation(String),
    
    #[error("Token validation failed: {0}")]
    Validation(String),
    
    #[error("Token expired")]
    Expired,
    
    #[error("Invalid token")]
    Invalid,
    
    #[error("Missing claim: {0}")]
    MissingClaim(String),
}

/// Result type for JWT operations
pub type Result<T> = std::result::Result<T, JwtError>;

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    
    /// User email
    pub email: String,
    
    /// User role
    pub role: String,
    
    /// Device ID
    pub device_id: String,
    
    /// Department ID (optional)
    pub department_id: Option<String>,
    
    /// Issued at timestamp
    pub iat: i64,
    
    /// Expiration timestamp
    pub exp: i64,
    
    /// Token ID for revocation tracking
    pub jti: String,
    
    /// Offline-capable flag
    pub offline: bool,
}

impl Claims {
    pub fn user_id(&self) -> Option<Id> {
        Id::parse_str(&self.sub).ok()
    }

    pub fn device_id(&self) -> Option<Id> {
        Id::parse_str(&self.device_id).ok()
    }

    pub fn user_role(&self) -> UserRole {
        match self.role.as_str() {
            "PHYSICIAN" => UserRole::Physician,
            "NURSE" => UserRole::Nurse,
            "RECEPTIONIST" => UserRole::Receptionist,
            "BILLING" => UserRole::Billing,
            "ADMIN" => UserRole::Admin,
            "PATIENT" => UserRole::Patient,
            _ => UserRole::Patient,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}

/// JWT token manager
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_expiry: Duration,
    refresh_token_expiry: Duration,
    offline_token_expiry: Duration,
}

impl JwtManager {
    /// Create a new JWT manager with the given secret
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            access_token_expiry: Duration::minutes(15),
            refresh_token_expiry: Duration::days(7),
            offline_token_expiry: Duration::hours(24),
        }
    }

    /// Create an access token
    pub fn create_access_token(
        &self,
        user_id: Id,
        email: &str,
        role: UserRole,
        device_id: Id,
        department_id: Option<Id>,
    ) -> Result<String> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            role: role.as_str().to_string(),
            device_id: device_id.to_string(),
            department_id: department_id.map(|id| id.to_string()),
            iat: now.timestamp(),
            exp: (now + self.access_token_expiry).timestamp(),
            jti: Id::new_v4().to_string(),
            offline: false,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| JwtError::Creation(e.to_string()))
    }

    /// Create an offline-capable token (longer validity)
    pub fn create_offline_token(
        &self,
        user_id: Id,
        email: &str,
        role: UserRole,
        device_id: Id,
        department_id: Option<Id>,
    ) -> Result<String> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            role: role.as_str().to_string(),
            device_id: device_id.to_string(),
            department_id: department_id.map(|id| id.to_string()),
            iat: now.timestamp(),
            exp: (now + self.offline_token_expiry).timestamp(),
            jti: Id::new_v4().to_string(),
            offline: true,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| JwtError::Creation(e.to_string()))
    }

    /// Create a refresh token
    pub fn create_refresh_token(&self, user_id: Id, device_id: Id) -> Result<String> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id.to_string(),
            email: String::new(),
            role: String::new(),
            device_id: device_id.to_string(),
            department_id: None,
            iat: now.timestamp(),
            exp: (now + self.refresh_token_expiry).timestamp(),
            jti: Id::new_v4().to_string(),
            offline: false,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| JwtError::Creation(e.to_string()))
    }

    /// Validate and decode a token
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::default();
        
        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| {
                if e.to_string().contains("ExpiredSignature") {
                    JwtError::Expired
                } else {
                    JwtError::Validation(e.to_string())
                }
            })
    }

    /// Decode a token without validation (for expired token inspection)
    pub fn decode_without_validation(&self, token: &str) -> Result<Claims> {
        let mut validation = Validation::default();
        validation.validate_exp = false;
        
        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| JwtError::Validation(e.to_string()))
    }

    /// Check if token needs refresh
    pub fn needs_refresh(&self, claims: &Claims) -> bool {
        let remaining = claims.exp - Utc::now().timestamp();
        remaining < 300 // Less than 5 minutes remaining
    }
}

/// Token pair for authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: String,
}

impl TokenPair {
    pub fn new(access_token: String, refresh_token: String, expires_in: i64) -> Self {
        Self {
            access_token,
            refresh_token,
            expires_in,
            token_type: "Bearer".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_validate_token() {
        let manager = JwtManager::new(b"test-secret-key-32-bytes-long!!");
        
        let user_id = Id::new_v4();
        let device_id = Id::new_v4();
        
        let token = manager.create_access_token(
            user_id,
            "test@example.com",
            UserRole::Physician,
            device_id,
            None,
        ).unwrap();

        let claims = manager.validate_token(&token).unwrap();
        
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.role, "PHYSICIAN");
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_offline_token() {
        let manager = JwtManager::new(b"test-secret-key-32-bytes-long!!");
        
        let token = manager.create_offline_token(
            Id::new_v4(),
            "test@example.com",
            UserRole::Nurse,
            Id::new_v4(),
            None,
        ).unwrap();

        let claims = manager.validate_token(&token).unwrap();
        assert!(claims.offline);
    }
}
