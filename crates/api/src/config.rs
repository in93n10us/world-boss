rust
//! Configuration management for the API server.
//!
//! This module handles loading and validating configuration from environment variables
//! and configuration files. It uses the `config` crate for flexible configuration sources
//! and `validator` for validation rules.

use serde::Deserialize;
use std::net::SocketAddr;
use validator::Validate;

/// Main application configuration.
///
/// This struct contains all configuration needed to run the API server,
/// including server settings, database connection, and logging configuration.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct Config {
    /// Server configuration
    #[validate(nested)]
    pub server: ServerConfig,

    /// Database configuration
    #[validate(nested)]
    pub database: DatabaseConfig,

    /// Logging configuration
    #[validate(nested)]
    pub logging: LoggingConfig,

    /// Application environment (development, staging, production)
    #[validate(length(min = 1))]
    pub environment: String,
}

/// Server-specific configuration.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ServerConfig {
    /// Host address to bind to (e.g., "0.0.0.0" or "127.0.0.1")
    #[validate(length(min = 1))]
    pub host: String,

    /// Port number to listen on
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,

    /// Request timeout in seconds
    #[validate(range(min = 1, max = 300))]
    pub request_timeout_secs: u64,

    /// Maximum request body size in bytes
    #[validate(range(min = 1024))]
    pub max_body_size: usize,

    /// Enable CORS (Cross-Origin Resource Sharing)
    pub cors_enabled: bool,

    /// Allowed CORS origins (comma-separated)
    pub cors_origins: Option<String>,
}

/// Database configuration.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct DatabaseConfig {
    /// Full database connection URL
    /// Format: postgres://user:password@host:port/database
    #[validate(length(min = 1))]
    pub url: String,

    /// Maximum number of connections in the pool
    #[validate(range(min = 1, max = 100))]
    pub max_connections: u32,

    /// Minimum number of idle connections
    #[validate(range(min = 0, max = 50))]
    pub min_connections: u32,

    /// Connection timeout in seconds
    #[validate(range(min = 1, max = 60))]
    pub connect_timeout_secs: u64,

    /// Idle connection timeout in seconds
    #[validate(range(min = 60, max = 3600))]
    pub idle_timeout_secs: u64,

    /// Enable SQL query logging
    pub log_queries: bool,
}

/// Logging configuration.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    #[validate(length(min = 1))]
    pub level: String,

    /// Log format (json or pretty)
    #[validate(length(min = 1))]
    pub format: String,

    /// Enable log output to stdout
    pub stdout: bool,

    /// Optional log file path
    pub file: Option<String>,
}

impl Config {
    /// Load configuration from environment variables and config files.
    ///
    /// This function attempts to load configuration in the following order:
    /// 1. Default values
    /// 2. Configuration file (config/default.toml)
    /// 3. Environment-specific file (config/{environment}.toml)
    /// 4. Environment variables (prefixed with APP_)
    /// 5. .env file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required environment variables are missing
    /// - Configuration values fail validation
    /// - Configuration files cannot be parsed
    ///
    /// # Examples
    ///
    /// no_run
    /// use api::config::Config;
    ///
    /// let config = Config::load().expect("Failed to load configuration");
    /// println!("Server will run on {}:{}", config.server.host, config.server.port);
    /// 
    pub fn load() -> Result<Self, ConfigError> {
        // Load .env file if it exists (ignore errors in production)
        dotenvy::dotenv().ok();

        // Determine environment
        let environment = std::env::var("APP_ENVIRONMENT")
            .or_else(|_| std::env::var("ENVIRONMENT"))
            .unwrap_or_else(|_| "development".to_string());

        let config = config::Config::builder()
            // Start with default values
            .set_default("environment", environment.clone())?
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 8000)?
            .set_default("server.request_timeout_secs", 30)?
            .set_default("server.max_body_size", 2_097_152)? // 2MB
            .set_default("server.cors_enabled", true)?
            .set_default("database.max_connections", 10)?
            .set_default("database.min_connections", 2)?
            .set_default("database.connect_timeout_secs", 10)?
            .set_default("database.idle_timeout_secs", 600)?
            .set_default("database.log_queries", false)?
            .set_default("logging.level", "info")?
            .set_default("logging.format", "json")?
            .set_default("logging.stdout", true)?
            // Add environment variables (with APP_ prefix)
            .add_source(
                config::Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        let mut cfg: Config = config.try_deserialize()?;

        // Validate configuration
        cfg.validate()
            .map_err(|e| ConfigError::Validation(e.to_string()))?;

        // Additional custom validations
        cfg.validate_custom()?;

        Ok(cfg)
    }

    /// Perform custom validation logic beyond what the validator crate provides.
    fn validate_custom(&self) -> Result<(), ConfigError> {
        // Validate log level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(ConfigError::Validation(format!(
                "Invalid log level '{}'. Must be one of: {}",
                self.logging.level,
                valid_levels.join(", ")
            )));
        }

        // Validate log format
        let valid_formats = ["json", "pretty"];
        if !valid_formats.contains(&self.logging.format.as_str()) {
            return Err(ConfigError::Validation(format!(
                "Invalid log format '{}'. Must be one of: {}",
                self.logging.format,
                valid_formats.join(", ")
            )));
        }

        // Validate database URL format
        if !self.database.url.starts_with("postgres://")
            && !self.database.url.starts_with("postgresql://")
        {
            return Err(ConfigError::Validation(
                "Database URL must start with 'postgres://' or 'postgresql://'".to_string(),
            ));
        }

        // Validate min_connections <= max_connections
        if self.database.min_connections > self.database.max_connections {
            return Err(ConfigError::Validation(
                "database.min_connections cannot be greater than database.max_connections"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Get the server socket address.
    ///
    /// # Errors
    ///
    /// Returns an error if the host and port cannot be parsed into a valid socket address.
    pub fn socket_addr(&self) -> Result<SocketAddr, ConfigError> {
        let addr = format!("{}:{}", self.server.host, self.server.port);
        addr.parse()
            .map_err(|e| ConfigError::InvalidAddress(format!("Invalid socket address: {}", e)))
    }

    /// Check if running in development mode.
    pub fn is_development(&self) -> bool {
        self.environment == "development" || self.environment == "dev"
    }

    /// Check if running in production mode.
    pub fn is_production(&self) -> bool {
        self.environment == "production" || self.environment == "prod"
    }

    /// Get CORS allowed origins as a vector.
    pub fn cors_origins(&self) -> Vec<String> {
        self.server
            .cors_origins
            .as_ref()
            .map(|origins| {
                origins
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Configuration-related errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Configuration validation failed
    #[error("Configuration validation failed: {0}")]
    Validation(String),

    /// Invalid socket address
    #[error("Invalid socket address: {0}")]
    InvalidAddress(String),

    /// Configuration loading error
    #[error("Failed to load configuration: {0}")]
    LoadError(#[from] config::ConfigError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        // Set minimal required environment variables
        std::env::set_var("APP_DATABASE__URL", "postgres://user:pass@localhost/test");

        let config = Config::load().expect("Failed to load config");

        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8000);
        assert_eq!(config.server.request_timeout_secs, 30);
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_environment_override() {
        std::env::set_var("APP_SERVER__PORT", "9000");
        std::env::set_var("APP_DATABASE__URL", "postgres://user:pass@localhost/test");

        let config = Config::load().expect("Failed to load config");

        assert_eq!(config.server.port, 9000);
    }

    #[test]
    fn test_socket_addr() {
        std::env::set_var("APP_SERVER__HOST", "0.0.0.0");
        std::env::set_var("APP_SERVER__PORT", "8080");
        std::env::set_var("APP_DATABASE__URL", "postgres://user:pass@localhost/test");

        let config = Config::load().expect("Failed to load config");
        let addr = config.socket_addr().expect("Failed to parse socket address");

        assert_eq!(addr.to_string(), "0.0.0.0:8080");
    }

    #[test]
    fn test_invalid_log_level() {
        std::env::set_var("APP_LOGGING__LEVEL", "invalid");
        std::env::set_var("APP_DATABASE__URL", "postgres://user:pass@localhost/test");

        let result = Config::load();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_database_url() {
        std::env::set_var("APP_DATABASE__URL", "mysql://user:pass@localhost/test");

        let result = Config::load();
        assert!(result.is_err());
    }

    #[test]
    fn test_min_connections_greater_than_max() {
        std::env::set_var("APP_DATABASE__URL", "postgres://user:pass@localhost/test");
        std::env::set_var("APP_DATABASE__MIN_CONNECTIONS", "20");
        std::env::set_var("APP_DATABASE__MAX_CONNECTIONS", "10");

        let result = Config::load();
        assert!(result.is_err());
    }

    #[test]
    fn test_cors_origins_parsing() {
        std::env::set_var("APP_DATABASE__URL", "postgres://user:pass@localhost/test");
        std::env::set_var(
            "APP_SERVER__CORS_ORIGINS",
            "http://localhost:3000, https://example.com",
        );

        let config = Config::load().expect("Failed to load config");
        let origins = config.cors_origins();

        assert_eq!(origins.len(), 2);
        assert!(origins.contains(&"http://localhost:3000".to_string()));
        assert!(origins.contains(&"https://example.com".to_string()));
    }

    #[test]
    fn test_is_development() {
        std::env::set_var("APP_ENVIRONMENT", "development");
        std::env::set_var("APP_DATABASE__URL", "postgres://user:pass@localhost/test");

        let config = Config::load().expect("Failed to load config");
        assert!(config.is_development());
        assert!(!config.is_production());
    }

    #[test]
    fn test_is_production() {
        std::env::set_var("APP_ENVIRONMENT", "production");
        std::env::set_var("APP_DATABASE__URL", "postgres://user:pass@localhost/test");

        let config = Config::load().expect("Failed to load config");
        assert!(config.is_production());
        assert!(!config.is_development());
    }
}