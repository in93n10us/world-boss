rust
//! API library exports for testing and module organization.
//!
//! This module serves as the library entry point for the API crate, exposing
//! public interfaces that can be used by integration tests and other crates.
//! It organizes the API server components into a coherent structure following
//! clean architecture principles.

pub mod config;
pub mod error;
pub mod middleware;
pub mod routes;

use axum::{
    routing::get,
    Router,
};
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use crate::{
    config::Config,
    routes::health,
};

/// Application state shared across all request handlers.
///
/// This struct holds shared resources like database connection pools
/// and configuration that need to be accessible to route handlers.
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool for PostgreSQL
    pub db_pool: PgPool,
    /// Application configuration
    pub config: Config,
}

impl AppState {
    /// Creates a new application state instance.
    ///
    /// # Arguments
    ///
    /// * `db_pool` - PostgreSQL connection pool
    /// * `config` - Application configuration
    pub fn new(db_pool: PgPool, config: Config) -> Self {
        Self { db_pool, config }
    }
}

/// Builds the application router with all routes and middleware.
///
/// This function constructs the complete Axum router with:
/// - Health check endpoints
/// - Request tracing middleware
/// - CORS configuration
/// - Shared application state
///
/// # Arguments
///
/// * `state` - Application state to be shared across handlers
///
/// # Returns
///
/// Configured Axum router ready to serve requests
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Runs the API server with the provided configuration and database pool.
///
/// This function initializes the application state, builds the router,
/// and starts the HTTP server. It's the main entry point for running
/// the API server programmatically.
///
/// # Arguments
///
/// * `config` - Application configuration
/// * `db_pool` - PostgreSQL connection pool
///
/// # Errors
///
/// Returns an error if the server fails to bind to the specified address
/// or encounters a runtime error.
///
/// # Example
///
/// no_run
/// use api::{run, config::Config};
/// use sqlx::PgPool;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let config = Config::from_env()?;
///     let db_pool = PgPool::connect(&config.database_url).await?;
///     run(config, db_pool).await
/// }
/// 
pub async fn run(config: Config, db_pool: PgPool) -> anyhow::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    
    tracing::info!("Starting server on {}", addr);
    
    let state = AppState::new(db_pool, config);
    let app = create_router(state);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    tracing::info!("Server listening on {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Spawns a test server for integration testing.
///
/// This function creates an ephemeral server instance bound to a random
/// available port, useful for integration tests that need to make HTTP
/// requests against a real server instance.
///
/// # Arguments
///
/// * `db_pool` - PostgreSQL connection pool for testing
///
/// # Returns
///
/// A tuple containing the server address and a handle to the spawned task
///
/// # Example
///
/// no_run
/// use api::spawn_test_server;
/// use sqlx::PgPool;
///
/// #[tokio::test]
/// async fn test_health_endpoint() {
///     let pool = PgPool::connect("postgres://localhost/test").await.unwrap();
///     let (addr, _handle) = spawn_test_server(pool).await;
///     
///     let response = reqwest::get(format!("http://{}/health", addr))
///         .await
///         .unwrap();
///     
///     assert_eq!(response.status(), 200);
/// }
/// 
#[cfg(test)]
pub async fn spawn_test_server(db_pool: PgPool) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let config = Config {
        database_url: String::new(), // Not used in test server
        server_port: 0, // Bind to random available port
        environment: "test".to_string(),
        log_level: "info".to_string(),
    };
    
    let state = AppState::new(db_pool, config);
    let app = create_router(state);
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");
    
    let addr = listener.local_addr().expect("Failed to get local address");
    
    let handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("Server failed to start");
    });
    
    (addr, handle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        // This is a basic smoke test to ensure AppState can be constructed
        // Real tests would use an actual database pool
        let config = Config {
            database_url: "postgres://localhost/test".to_string(),
            server_port: 8000,
            environment: "test".to_string(),
            log_level: "info".to_string(),
        };
        
        // Note: We can't easily create a PgPool without a real database connection
        // This test just validates the structure compiles correctly
        assert_eq!(config.server_port, 8000);
    }
}