rust
//! Database connection pool setup and repository implementations.
//!
//! This module provides the database layer for the StudyFlow application,
//! including connection pool management and repository trait implementations
//! for data access.

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub mod connection;
pub mod repositories;

pub use connection::create_pool;

/// Database error type wrapping SQLx errors.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// Database connection or query error
    #[error("Database error: {0}")]
    SqlxError(#[from] sqlx::Error),

    /// Entity not found error
    #[error("Entity not found: {0}")]
    NotFound(String),

    /// Constraint violation error (e.g., unique constraint)
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    /// Migration error
    #[error("Migration error: {0}")]
    MigrationError(String),
}

/// Result type alias for database operations.
pub type DbResult<T> = Result<T, DbError>;

/// Database configuration options.
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// Database connection URL
    pub database_url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections in the pool
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Maximum lifetime of a connection in seconds
    pub max_lifetime: u64,
    /// Idle timeout in seconds
    pub idle_timeout: u64,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            database_url: String::from("postgres://localhost/studyflow_dev"),
            max_connections: 10,
            min_connections: 2,
            connect_timeout: 30,
            max_lifetime: 1800,
            idle_timeout: 600,
        }
    }
}

impl DbConfig {
    /// Creates a new database configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are missing or invalid.
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL environment variable not set")?;

        let max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        let min_connections = std::env::var("DB_MIN_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2);

        let connect_timeout = std::env::var("DB_CONNECT_TIMEOUT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        let max_lifetime = std::env::var("DB_MAX_LIFETIME")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1800);

        let idle_timeout = std::env::var("DB_IDLE_TIMEOUT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(600);

        Ok(Self {
            database_url,
            max_connections,
            min_connections,
            connect_timeout,
            max_lifetime,
            idle_timeout,
        })
    }

    /// Creates a connection pool with this configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the pool cannot be created or the database is unreachable.
    pub async fn create_pool(&self) -> DbResult<PgPool> {
        let pool = PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .acquire_timeout(Duration::from_secs(self.connect_timeout))
            .max_lifetime(Duration::from_secs(self.max_lifetime))
            .idle_timeout(Duration::from_secs(self.idle_timeout))
            .connect(&self.database_url)
            .await?;

        Ok(pool)
    }
}

/// Runs database migrations.
///
/// # Arguments
///
/// * `pool` - Database connection pool
///
/// # Errors
///
/// Returns an error if migrations fail to run.
pub async fn run_migrations(pool: &PgPool) -> DbResult<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| DbError::MigrationError(e.to_string()))?;

    Ok(())
}

/// Checks if the database connection is healthy.
///
/// # Arguments
///
/// * `pool` - Database connection pool
///
/// # Errors
///
/// Returns an error if the health check query fails.
pub async fn health_check(pool: &PgPool) -> DbResult<()> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(DbError::from)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_config_default() {
        let config = DbConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 2);
        assert_eq!(config.connect_timeout, 30);
        assert_eq!(config.max_lifetime, 1800);
        assert_eq!(config.idle_timeout, 600);
    }

    #[test]
    fn test_db_error_display() {
        let error = DbError::NotFound("User".to_string());
        assert_eq!(error.to_string(), "Entity not found: User");

        let error = DbError::ConstraintViolation("unique_email".to_string());
        assert_eq!(error.to_string(), "Constraint violation: unique_email");
    }

    #[tokio::test]
    async fn test_db_config_from_env_missing_url() {
        std::env::remove_var("DATABASE_URL");
        let result = DbConfig::from_env();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_db_config_from_env_with_defaults() {
        std::env::set_var("DATABASE_URL", "postgres://localhost/test");
        std::env::remove_var("DB_MAX_CONNECTIONS");

        let config = DbConfig::from_env().expect("Failed to create config");
        assert_eq!(config.database_url, "postgres://localhost/test");
        assert_eq!(config.max_connections, 10); // default value

        std::env::remove_var("DATABASE_URL");
    }

    #[tokio::test]
    async fn test_db_config_from_env_with_custom_values() {
        std::env::set_var("DATABASE_URL", "postgres://localhost/test");
        std::env::set_var("DB_MAX_CONNECTIONS", "20");
        std::env::set_var("DB_MIN_CONNECTIONS", "5");
        std::env::set_var("DB_CONNECT_TIMEOUT", "60");

        let config = DbConfig::from_env().expect("Failed to create config");
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.connect_timeout, 60);

        // Cleanup
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("DB_MAX_CONNECTIONS");
        std::env::remove_var("DB_MIN_CONNECTIONS");
        std::env::remove_var("DB_CONNECT_TIMEOUT");
    }
}