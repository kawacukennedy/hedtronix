//! Authentication middleware for Axum

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use hedtronix_core::UserRole;

use crate::jwt::Claims;
use crate::permissions::PermissionChecker;

/// Authentication state for middleware
#[derive(Clone)]
pub struct AuthState {
    pub jwt_secret: Vec<u8>,
}

impl AuthState {
    pub fn new(jwt_secret: Vec<u8>) -> Self {
        Self { jwt_secret }
    }
}

/// Extract and validate JWT from request
pub async fn auth_middleware(
    State(state): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let jwt_manager = crate::jwt::JwtManager::new(&state.jwt_secret);
    
    match jwt_manager.validate_token(token) {
        Ok(claims) => {
            // Store claims in request extensions for later use
            request.extensions_mut().insert(claims);
            Ok(next.run(request).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Permission checking middleware generator
pub fn require_permission(
    resource: &'static str,
    action: &'static str,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    move |request: Request, next: Next| {
        Box::pin(async move {
            let claims = request
                .extensions()
                .get::<Claims>()
                .ok_or(StatusCode::UNAUTHORIZED)?;

            let role = claims.user_role();
            
            if PermissionChecker::has_permission(role, resource, action) {
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        })
    }
}

/// Extract claims from request
pub fn get_claims(request: &Request) -> Option<&Claims> {
    request.extensions().get::<Claims>()
}

/// Macro for permission guards
#[macro_export]
macro_rules! require {
    ($resource:expr, $action:expr) => {
        axum::middleware::from_fn(move |request, next| {
            $crate::middleware::require_permission($resource, $action)(request, next)
        })
    };
}

/// Response type for auth errors
pub struct AuthError {
    pub status: StatusCode,
    pub message: String,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (self.status, self.message).into_response()
    }
}
