//! HEDTRONIX API Server
//!
//! REST API for the healthcare operating system.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use hedtronix_db::Database;
use hedtronix_auth::AuthState;

mod routes;
mod handlers;
mod state;
mod error;
mod config;

pub use state::AppState;
pub use error::ApiError;

/// Start the API server
pub async fn start_server(config: config::ServerConfig) -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "hedtronix_api=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database
    let mut db = Database::open(&config.database_path)?;
    db.initialize()?;

    // Create app state
    let state = AppState::new(db, config.jwt_secret.clone());

    // Build router
    let app = create_router(state);

    // Start server
    let addr: SocketAddr = config.bind_address.parse()?;
    tracing::info!("Starting HEDTRONIX server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Create the API router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(handlers::health::health_check))
        
        // Authentication routes
        .nest("/api/v1/auth", routes::auth_routes())
        
        // Patient routes
        .nest("/api/v1/patients", routes::patient_routes())
        
        // Appointment routes
        .nest("/api/v1/appointments", routes::appointment_routes())
        
        // Sync routes
        .nest("/api/v1/sync", routes::sync_routes())
        
        // User routes (admin)
        .nest("/api/v1/users", routes::user_routes())
        
        // Clinical Notes routes
        .nest("/api/v1/clinical-notes", routes::clinical_note_routes())
        
        // Billing routes
        .nest("/api/v1/billing", routes::billing_routes())
        
        // CORS and tracing
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
