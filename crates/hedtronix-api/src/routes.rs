//! Route definitions

use axum::{
    routing::{get, post, put, delete},
    Router,
};

use crate::handlers;
use crate::state::AppState;

/// Authentication routes (public)
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(handlers::auth::login))
        .route("/refresh", post(handlers::auth::refresh))
        .route("/logout", post(handlers::auth::logout))
        .route("/register", post(handlers::auth::register))
}

/// Patient routes (protected)
pub fn patient_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::patients::list_patients))
        .route("/", post(handlers::patients::create_patient))
        .route("/:id", get(handlers::patients::get_patient))
        .route("/:id", put(handlers::patients::update_patient))
        .route("/:id", delete(handlers::patients::delete_patient))
        .route("/:id/allergies", post(handlers::patients::add_allergy))
        .route("/:id/medications", post(handlers::patients::add_medication))
        .route("/search", post(handlers::patients::search_patients))
}

/// Appointment routes (protected)
pub fn appointment_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::appointments::list_appointments))
        .route("/", post(handlers::appointments::create_appointment))
        .route("/:id", get(handlers::appointments::get_appointment))
        .route("/:id", put(handlers::appointments::update_appointment))
        .route("/:id", delete(handlers::appointments::cancel_appointment))
        .route("/:id/check-in", post(handlers::appointments::check_in))
        .route("/:id/complete", post(handlers::appointments::complete))
        .route("/conflicts", post(handlers::appointments::check_conflicts))
        .route("/calendar", get(handlers::appointments::get_calendar))
}

/// Sync routes (protected)
pub fn sync_routes() -> Router<AppState> {
    Router::new()
        .route("/push", post(handlers::sync::push_changes))
        .route("/pull", post(handlers::sync::pull_changes))
        .route("/status", get(handlers::sync::get_status))
        .route("/health", get(handlers::sync::get_health))
}

/// User management routes (admin only)
pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::users::list_users))
        .route("/", post(handlers::users::create_user))
        .route("/:id", get(handlers::users::get_user))
        .route("/:id", put(handlers::users::update_user))
        .route("/:id", delete(handlers::users::delete_user))
        .route("/me", get(handlers::users::get_current_user))
}

/// Clinical Note routes
pub fn clinical_note_routes() -> Router<AppState> {
    Router::new()
        .route("/patient/:id", get(handlers::clinical_notes::list_notes))
        .route("/", post(handlers::clinical_notes::create_note))
        .route("/:id", get(handlers::clinical_notes::get_note))
        .route("/:id", put(handlers::clinical_notes::update_note))
        .route("/:id/sign", post(handlers::clinical_notes::sign_note))
}

/// Billing routes
pub fn billing_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::billing::list_billing))
        .route("/", post(handlers::billing::create_billing))
}
