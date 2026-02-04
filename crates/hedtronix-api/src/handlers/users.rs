//! User management handlers

use axum::{
    extract::{Path, Query, State, Request},
    Json,
};
use hedtronix_core::{User, UserRole, Id};
use hedtronix_db::UserRepository;
use hedtronix_auth::Claims;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;

/// List users (admin only)
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

pub async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListUsersResponse>, ApiError> {
    let repo = UserRepository::new(state.db.clone());
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.page.unwrap_or(0) * limit;
    
    let users = repo.find_all(limit, offset)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    let total = repo.count()
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    Ok(Json(ListUsersResponse {
        users: users.into_iter().map(UserDto::from).collect(),
        total,
        page: query.page.unwrap_or(0),
        limit,
    }))
}

#[derive(Debug, Serialize)]
pub struct ListUsersResponse {
    pub users: Vec<UserDto>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
}

/// Get user by ID
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<UserDto>, ApiError> {
    let user_id = Id::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid user ID"))?;
    
    let repo = UserRepository::new(state.db.clone());
    let user = repo.find_by_id(user_id)
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("User"))?;
    
    Ok(Json(UserDto::from(user)))
}

/// Create user (admin only)
pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserDto>, ApiError> {
    let role = parse_role(&req.role)?;
    
    let password_hash = hedtronix_crypto::hashing::hash_password(&req.password)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    let user = User::new(req.email, req.name, role, password_hash);
    
    let repo = UserRepository::new(state.db.clone());
    repo.create(&user)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    Ok(Json(UserDto::from(user)))
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
    pub password: String,
    pub role: String,
}

/// Update user
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserDto>, ApiError> {
    let user_id = Id::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid user ID"))?;
    
    let repo = UserRepository::new(state.db.clone());
    let mut user = repo.find_by_id(user_id)
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("User"))?;
    
    if let Some(name) = req.name {
        user.name = name;
    }
    if let Some(email) = req.email {
        user.email = email;
    }
    if let Some(active) = req.active {
        user.active = active;
    }
    if let Some(role) = req.role {
        user.role = parse_role(&role)?;
    }
    user.updated_at = chrono::Utc::now();
    
    repo.update(&user)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    Ok(Json(UserDto::from(user)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub active: Option<bool>,
}

/// Delete user (soft delete)
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DeleteResponse>, ApiError> {
    let user_id = Id::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid user ID"))?;
    
    let repo = UserRepository::new(state.db.clone());
    let mut user = repo.find_by_id(user_id)
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("User"))?;
    
    user.active = false;
    user.updated_at = chrono::Utc::now();
    
    repo.update(&user)
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    
    Ok(Json(DeleteResponse { success: true }))
}

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub success: bool,
}

/// Get current user from token
pub async fn get_current_user(
    State(state): State<AppState>,
    request: Request,
) -> Result<Json<UserDto>, ApiError> {
    let claims = request.extensions()
        .get::<Claims>()
        .ok_or_else(|| ApiError::unauthorized("Missing claims"))?;
    
    let user_id = claims.user_id()
        .ok_or_else(|| ApiError::unauthorized("Invalid user ID in token"))?;
    
    let repo = UserRepository::new(state.db.clone());
    let user = repo.find_by_id(user_id)
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("User"))?;
    
    Ok(Json(UserDto::from(user)))
}

// Helper
fn parse_role(s: &str) -> Result<UserRole, ApiError> {
    match s.to_uppercase().as_str() {
        "PHYSICIAN" => Ok(UserRole::Physician),
        "NURSE" => Ok(UserRole::Nurse),
        "RECEPTIONIST" => Ok(UserRole::Receptionist),
        "BILLING" => Ok(UserRole::Billing),
        "ADMIN" => Ok(UserRole::Admin),
        "PATIENT" => Ok(UserRole::Patient),
        _ => Err(ApiError::bad_request("Invalid role")),
    }
}

/// User DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct UserDto {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub department_id: Option<String>,
    pub active: bool,
    pub created_at: String,
    pub last_login_at: Option<String>,
}

impl From<User> for UserDto {
    fn from(u: User) -> Self {
        Self {
            id: u.id.to_string(),
            email: u.email,
            name: u.name,
            role: u.role.as_str().to_string(),
            department_id: u.department_id.map(|id| id.to_string()),
            active: u.active,
            created_at: u.created_at.to_rfc3339(),
            last_login_at: u.last_login_at.map(|t| t.to_rfc3339()),
        }
    }
}
