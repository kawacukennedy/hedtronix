//! Authentication handlers

use axum::{extract::State, Json};
use hedtronix_core::{Id, UserRole};
use hedtronix_auth::{AuthService, LoginRequest, RefreshRequest, AuthResponse, TokenPair};
use hedtronix_db::Database;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;

/// Login request
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let device_id = req.device_id
        .and_then(|s| Id::parse_str(&s).ok())
        .unwrap_or_else(Id::new_v4);
    
    let auth_service = AuthService::new(&state.auth_state.jwt_secret, state.db.clone());
    let response = auth_service.login(&req.email, &req.password, device_id)?;
    
    Ok(Json(response))
}

/// Refresh token
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<TokenPair>, ApiError> {
    let auth_service = AuthService::new(&state.auth_state.jwt_secret, state.db.clone());
    let tokens = auth_service.refresh(&req.refresh_token)?;
    
    Ok(Json(tokens))
}

/// Logout (invalidate token - currently just a placeholder)
pub async fn logout() -> Result<Json<LogoutResponse>, ApiError> {
    // In a production system, we would add the token to a blacklist
    Ok(Json(LogoutResponse { success: true }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutResponse {
    pub success: bool,
}

/// Register new user (admin only in production)
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub password: String,
    pub role: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, ApiError> {
    let role = match req.role.to_uppercase().as_str() {
        "PHYSICIAN" => UserRole::Physician,
        "NURSE" => UserRole::Nurse,
        "RECEPTIONIST" => UserRole::Receptionist,
        "BILLING" => UserRole::Billing,
        "ADMIN" => UserRole::Admin,
        "PATIENT" => UserRole::Patient,
        _ => return Err(ApiError::bad_request("Invalid role")),
    };

    let auth_service = AuthService::new(&state.auth_state.jwt_secret, state.db.clone());
    let user = auth_service.register_user(&req.email, &req.name, &req.password, role)?;

    Ok(Json(RegisterResponse {
        id: user.id.to_string(),
        email: user.email,
        name: user.name,
        role: user.role.as_str().to_string(),
    }))
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
}
