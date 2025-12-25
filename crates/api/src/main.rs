rust
//! Application entry point for the StudyFlow API server.
//!
//! This module initializes the server, sets up graceful shutdown handling,
//! and starts the HTTP server with all configured routes and middleware.

use anyhow::Context;
use std::net::SocketAddr;
use tokio::signal;
use tracing::info;

use api::config::Config;
use api::routes;
use db::connection::create_pool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber for structured logging
    init_tracing();

    info!("Starting StudyFlow API server");

    // Load configuration from environment
    let config = Config::from_env().context("Failed to load configuration")?;

    info!(
        "Configuration loaded: host={}, port={}, database_url={}",
        config.host,
        config.port,
        mask_database_url(&config.database_url)
    );

    // Initialize database connection pool
    let db_pool = create_pool(&config.database_url)
        .await
        .context("Failed to create database connection pool")?;

    info!("Database connection pool established");

    // Run database migrations
    sqlx::migrate!("../db/migrations")
        .run(&db_pool)
        .await
        .context("Failed to run database migrations")?;

    info!("Database migrations completed successfully");

    // Build application router with all routes and middleware
    let app = routes::create_router(db_pool.clone());

    // Parse socket address
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .context("Failed to parse server address")?;

    info!("Server listening on {}", addr);

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("Failed to bind to address")?;

    // Start server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Server error")?;

    info!("Server shutdown complete");

    Ok(())
}

/// Initialize the tracing subscriber for structured logging.
///
/// Configures JSON formatting for production and pretty printing for development.
/// Log level is controlled via the `RUST_LOG` environment variable.
fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,api=debug,core=debug,db=debug"));

    let formatting_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .json();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(formatting_layer)
        .init();
}

/// Wait for a shutdown signal (CTRL+C or SIGTERM).
///
/// This enables graceful shutdown, allowing in-flight requests to complete
/// and resources to be cleaned up properly.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received CTRL+C signal, initiating graceful shutdown");
        },
        _ = terminate => {
            info!("Received SIGTERM signal, initiating graceful shutdown");
        },
    }
}

/// Mask sensitive parts of the database URL for logging.
///
/// Replaces the password portion with asterisks to prevent credential leakage.
fn mask_database_url(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let mut masked = url.to_string();
            masked.replace_range(colon_pos + 1..at_pos, "****");
            return masked;
        }
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_database_url() {
        let url = "postgres://user:password@localhost:5432/db";
        let masked = mask_database_url(url);
        assert_eq!(masked, "postgres://user:****@localhost:5432/db");
    }

    #[test]
    fn test_mask_database_url_no_password() {
        let url = "postgres://localhost:5432/db";
        let masked = mask_database_url(url);
        assert_eq!(masked, "postgres://localhost:5432/db");
    }

    #[test]
    fn test_mask_database_url_empty() {
        let url = "";
        let masked = mask_database_url(url);
        assert_eq!(masked, "");
    }
}