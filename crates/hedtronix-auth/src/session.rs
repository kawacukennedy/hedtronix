//! Session management for authentication

use hedtronix_core::{Device, Id, User, UserRole};
use hedtronix_db::{Database, UserRepository};
use hedtronix_crypto::hashing::{hash_password, verify_password};
use thiserror::Error;

use crate::jwt::{JwtManager, TokenPair, Claims};

/// Session error types
#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("User disabled")]
    UserDisabled,
    
    #[error("Device not registered")]
    DeviceNotRegistered,
    
    #[error("Device revoked")]
    DeviceRevoked,
    
    #[error("Token error: {0}")]
    Token(String),
    
    #[error("Database error: {0}")]
    Database(String),
}

/// Result type for session operations
pub type Result<T> = std::result::Result<T, SessionError>;

/// Authentication service
pub struct AuthService {
    jwt_manager: JwtManager,
    db: Database,
}

impl AuthService {
    pub fn new(jwt_secret: &[u8], db: Database) -> Self {
        Self {
            jwt_manager: JwtManager::new(jwt_secret),
            db,
        }
    }

    /// Authenticate with email and password
    pub fn login(
        &self,
        email: &str,
        password: &str,
        device_id: Id,
    ) -> Result<AuthResponse> {
        let user_repo = UserRepository::new(self.db.clone());
        
        // Find user by email
        let user = user_repo.find_by_email(email)
            .map_err(|e| SessionError::Database(e.to_string()))?
            .ok_or(SessionError::UserNotFound)?;

        // Check if user is active
        if !user.active {
            return Err(SessionError::UserDisabled);
        }

        // Verify password
        let valid = verify_password(password, &user.password_hash)
            .map_err(|_| SessionError::InvalidCredentials)?;
        
        if !valid {
            return Err(SessionError::InvalidCredentials);
        }

        // Create tokens
        let access_token = self.jwt_manager.create_access_token(
            user.id,
            &user.email,
            user.role,
            device_id,
            user.department_id,
        ).map_err(|e| SessionError::Token(e.to_string()))?;

        let refresh_token = self.jwt_manager.create_refresh_token(user.id, device_id)
            .map_err(|e| SessionError::Token(e.to_string()))?;

        let offline_token = self.jwt_manager.create_offline_token(
            user.id,
            &user.email,
            user.role,
            device_id,
            user.department_id,
        ).map_err(|e| SessionError::Token(e.to_string()))?;

        Ok(AuthResponse {
            tokens: TokenPair::new(access_token, refresh_token, 900),
            offline_token,
            user: UserInfo::from(user),
        })
    }

    /// Refresh access token using refresh token
    pub fn refresh(&self, refresh_token: &str) -> Result<TokenPair> {
        let claims = self.jwt_manager.validate_token(refresh_token)
            .map_err(|e| SessionError::Token(e.to_string()))?;

        let user_repo = UserRepository::new(self.db.clone());
        let user = user_repo.find_by_id(claims.user_id().unwrap())
            .map_err(|e| SessionError::Database(e.to_string()))?
            .ok_or(SessionError::UserNotFound)?;

        if !user.active {
            return Err(SessionError::UserDisabled);
        }

        let device_id = claims.device_id().unwrap();

        let access_token = self.jwt_manager.create_access_token(
            user.id,
            &user.email,
            user.role,
            device_id,
            user.department_id,
        ).map_err(|e| SessionError::Token(e.to_string()))?;

        let new_refresh_token = self.jwt_manager.create_refresh_token(user.id, device_id)
            .map_err(|e| SessionError::Token(e.to_string()))?;

        Ok(TokenPair::new(access_token, new_refresh_token, 900))
    }

    /// Validate an access token and return claims
    pub fn validate(&self, token: &str) -> Result<Claims> {
        self.jwt_manager.validate_token(token)
            .map_err(|e| SessionError::Token(e.to_string()))
    }

    /// Get current user from token
    pub fn get_current_user(&self, token: &str) -> Result<User> {
        let claims = self.validate(token)?;
        let user_repo = UserRepository::new(self.db.clone());
        
        user_repo.find_by_id(claims.user_id().unwrap())
            .map_err(|e| SessionError::Database(e.to_string()))?
            .ok_or(SessionError::UserNotFound)
    }

    /// Register a new user (admin only)
    pub fn register_user(
        &self,
        email: &str,
        name: &str,
        password: &str,
        role: UserRole,
    ) -> Result<User> {
        let password_hash = hash_password(password)
            .map_err(|e| SessionError::Token(e.to_string()))?;

        let user = User::new(
            email.to_string(),
            name.to_string(),
            role,
            password_hash,
        );

        let user_repo = UserRepository::new(self.db.clone());
        user_repo.create(&user)
            .map_err(|e| SessionError::Database(e.to_string()))?;

        Ok(user)
    }
}

/// Authentication response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthResponse {
    pub tokens: TokenPair,
    pub offline_token: String,
    pub user: UserInfo,
}

/// Public user information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub department_id: Option<String>,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            email: user.email,
            name: user.name,
            role: user.role.as_str().to_string(),
            department_id: user.department_id.map(|id| id.to_string()),
        }
    }
}

/// Login request DTO
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub device_id: Option<String>,
}

/// Refresh request DTO
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}
