rust
//! Database connection pool initialization and management.
//!
//! This module provides functionality to create and configure SQLx connection pools
//! for PostgreSQL databases. It handles connection pooling, timeouts, and health checks.

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, PgPool};
use std::str::FromStr;
use std::time::Duration;
use tracing::log::LevelFilter;

/// Default maximum number of connections in the pool
const DEFAULT_MAX_CONNECTIONS: u32 = 10;

/// Default connection timeout in seconds
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 30;

/// Default idle connection timeout in seconds
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 600;

/// Default maximum lifetime for a connection in seconds
const DEFAULT_MAX_LIFETIME_SECS: u64 = 1800;

/// Configuration for database connection pool
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Database connection URL
    pub database_url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Connection timeout duration
    pub connect_timeout: Duration,
    /// Idle connection timeout duration
    pub idle_timeout: Duration,
    /// Maximum lifetime for a connection
    pub max_lifetime: Duration,
}

impl ConnectionConfig {
    /// Creates a new connection configuration with the given database URL
    ///
    /// # Arguments
    ///
    /// * `database_url` - PostgreSQL connection URL
    ///
    /// # Examples
    ///
    /// no_run
    /// use db::connection::ConnectionConfig;
    ///
    /// let config = ConnectionConfig::new(
    ///     "postgres://user:pass@localhost/dbname".to_string()
    /// );
    /// 
    pub fn new(database_url: String) -> Self {
        Self {
            database_url,
            max_connections: DEFAULT_MAX_CONNECTIONS,
            connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
            idle_timeout: Duration::from_secs(DEFAULT_IDLE_TIMEOUT_SECS),
            max_lifetime: Duration::from_secs(DEFAULT_MAX_LIFETIME_SECS),
        }
    }

    /// Sets the maximum number of connections in the pool
    pub fn with_max_connections(mut self, max_connections: u32) -> Self {
        self.max_connections = max_connections;
        self
    }

    /// Sets the connection timeout duration
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Sets the idle connection timeout duration
    pub fn with_idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// Sets the maximum lifetime for a connection
    pub fn with_max_lifetime(mut self, lifetime: Duration) -> Self {
        self.max_lifetime = lifetime;
        self
    }
}

/// Creates a PostgreSQL connection pool with the given configuration
///
/// # Arguments
///
/// * `config` - Connection configuration
///
/// # Returns
///
/// Returns a `Result` containing the connection pool or an error
///
/// # Errors
///
/// This function will return an error if:
/// * The database URL is invalid
/// * The connection to the database fails
/// * The pool cannot be created
///
/// # Examples
///
/// no_run
/// use db::connection::{create_pool, ConnectionConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = ConnectionConfig::new(
///         std::env::var("DATABASE_URL")?
///     );
///     let pool = create_pool(&config).await?;
///     Ok(())
/// }
/// 
pub async fn create_pool(config: &ConnectionConfig) -> Result<PgPool, sqlx::Error> {
    tracing::info!(
        "Creating database connection pool with max_connections={}",
        config.max_connections
    );

    // Parse connection options from URL
    let mut connect_options = PgConnectOptions::from_str(&config.database_url)?;

    // Configure connection options
    connect_options = connect_options
        .log_statements(LevelFilter::Debug)
        .log_slow_statements(LevelFilter::Warn, Duration::from_secs(1))
        .clone();

    // Build the connection pool
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(config.connect_timeout)
        .idle_timeout(Some(config.idle_timeout))
        .max_lifetime(Some(config.max_lifetime))
        .test_before_acquire(true)
        .connect_with(connect_options)
        .await?;

    tracing::info!("Database connection pool created successfully");

    Ok(pool)
}

/// Verifies that the database connection pool is healthy
///
/// # Arguments
///
/// * `pool` - Reference to the connection pool
///
/// # Returns
///
/// Returns `Ok(())` if the connection is healthy, otherwise returns an error
///
/// # Errors
///
/// This function will return an error if the database query fails
///
/// # Examples
///
/// no_run
/// use db::connection::{create_pool, verify_connection, ConnectionConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = ConnectionConfig::new(
///         std::env::var("DATABASE_URL")?
///     );
///     let pool = create_pool(&config).await?;
///     verify_connection(&pool).await?;
///     Ok(())
/// }
/// 
pub async fn verify_connection(pool: &PgPool) -> Result<(), sqlx::Error> {
    tracing::debug!("Verifying database connection");

    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map(|_| {
            tracing::info!("Database connection verified successfully");
        })
}

/// Gracefully closes the database connection pool
///
/// # Arguments
///
/// * `pool` - The connection pool to close
///
/// # Examples
///
/// no_run
/// use db::connection::{create_pool, close_pool, ConnectionConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = ConnectionConfig::new(
///         std::env::var("DATABASE_URL")?
///     );
///     let pool = create_pool(&config).await?;
///     // ... use pool ...
///     close_pool(pool).await;
///     Ok(())
/// }
/// 
pub async fn close_pool(pool: PgPool) {
    tracing::info!("Closing database connection pool");
    pool.close().await;
    tracing::info!("Database connection pool closed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_config_new() {
        let url = "postgres://user:pass@localhost/test".to_string();
        let config = ConnectionConfig::new(url.clone());

        assert_eq!(config.database_url, url);
        assert_eq!(config.max_connections, DEFAULT_MAX_CONNECTIONS);
        assert_eq!(
            config.connect_timeout,
            Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS)
        );
    }

    #[test]
    fn test_connection_config_builder() {
        let url = "postgres://user:pass@localhost/test".to_string();
        let config = ConnectionConfig::new(url.clone())
            .with_max_connections(20)
            .with_connect_timeout(Duration::from_secs(10))
            .with_idle_timeout(Duration::from_secs(300))
            .with_max_lifetime(Duration::from_secs(900));

        assert_eq!(config.database_url, url);
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.connect_timeout, Duration::from_secs(10));
        assert_eq!(config.idle_timeout, Duration::from_secs(300));
        assert_eq!(config.max_lifetime, Duration::from_secs(900));
    }

    #[test]
    fn test_connection_config_defaults() {
        let config = ConnectionConfig::new("postgres://localhost/test".to_string());

        assert_eq!(config.max_connections, 10);
        assert_eq!(config.connect_timeout.as_secs(), 30);
        assert_eq!(config.idle_timeout.as_secs(), 600);
        assert_eq!(config.max_lifetime.as_secs(), 1800);
    }
}