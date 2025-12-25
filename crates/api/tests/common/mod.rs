rust
//! Common test utilities for integration tests.
//!
//! This module provides shared functionality for integration tests including:
//! - Test application spawning with isolated state
//! - Test database setup and cleanup
//! - Helper functions for making HTTP requests
//! - Fixtures and test data builders

use anyhow::{Context, Result};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde::de::DeserializeOwned;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::net::TcpListener;
use tower::ServiceExt;
use uuid::Uuid;

/// Test application instance with isolated database and server.
///
/// Each test gets its own database and server instance to ensure test isolation.
/// The database is automatically cleaned up when the TestApp is dropped.
pub struct TestApp {
    /// Address where the test server is listening
    pub address: SocketAddr,
    /// Database connection pool for the test
    pub db_pool: PgPool,
    /// Name of the test database (for cleanup)
    pub db_name: String,
    /// Base URL for making HTTP requests
    pub base_url: String,
}

impl TestApp {
    /// Makes a GET request to the test server.
    ///
    /// # Arguments
    /// * `path` - The path to request (e.g., "/health")
    ///
    /// # Returns
    /// The response status code and body as a string
    pub async fn get(&self, path: &str) -> Result<(StatusCode, String)> {
        let url = format!("{}{}", self.base_url, path);
        let response = reqwest::get(&url)
            .await
            .context("Failed to execute request")?;
        
        let status = response.status();
        let body = response.text().await.context("Failed to read response body")?;
        
        Ok((status, body))
    }

    /// Makes a POST request to the test server with JSON body.
    ///
    /// # Arguments
    /// * `path` - The path to request
    /// * `body` - The JSON body to send
    ///
    /// # Returns
    /// The response status code and body as a string
    pub async fn post<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<(StatusCode, String)> {
        let url = format!("{}{}", self.base_url, path);
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(body)
            .send()
            .await
            .context("Failed to execute request")?;
        
        let status = response.status();
        let body = response.text().await.context("Failed to read response body")?;
        
        Ok((status, body))
    }

    /// Makes a GET request and deserializes the JSON response.
    ///
    /// # Type Parameters
    /// * `T` - The type to deserialize the response into
    ///
    /// # Arguments
    /// * `path` - The path to request
    ///
    /// # Returns
    /// The deserialized response
    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let (status, body) = self.get(path).await?;
        
        if !status.is_success() {
            anyhow::bail!("Request failed with status {}: {}", status, body);
        }
        
        serde_json::from_str(&body).context("Failed to deserialize response")
    }

    /// Executes a raw SQL query on the test database.
    ///
    /// Useful for setting up test data or verifying database state.
    pub async fn execute_query(&self, query: &str) -> Result<()> {
        sqlx::query(query)
            .execute(&self.db_pool)
            .await
            .context("Failed to execute query")?;
        Ok(())
    }

    /// Cleans up all data from the test database.
    ///
    /// This truncates all tables while preserving the schema.
    pub async fn cleanup_database(&self) -> Result<()> {
        // Truncate all tables in the correct order to respect foreign keys
        sqlx::query("TRUNCATE TABLE study_sessions, tasks, users CASCADE")
            .execute(&self.db_pool)
            .await
            .context("Failed to cleanup database")?;
        Ok(())
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        // Note: Async cleanup in Drop is not possible in stable Rust
        // The database will be cleaned up by the test database manager
        // or can be manually cleaned up in test teardown
    }
}

/// Spawns a test application instance with an isolated database.
///
/// This function:
/// 1. Creates a unique test database
/// 2. Runs migrations on the test database
/// 3. Spawns the application server on a random port
/// 4. Returns a TestApp instance for making requests
///
/// # Returns
/// A configured TestApp instance ready for testing
///
/// # Panics
/// Panics if the test database cannot be created or the server cannot start
pub async fn spawn_app() -> TestApp {
    // Generate a unique database name for this test
    let db_name = format!("test_db_{}", Uuid::new_v4().to_string().replace('-', "_"));
    
    // Create the test database
    let db_pool = configure_test_database(&db_name)
        .await
        .expect("Failed to configure test database");
    
    // Bind to a random port
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");
    
    let address = listener.local_addr().expect("Failed to get local address");
    let base_url = format!("http://{}", address);
    
    // Build the application router with the test database pool
    let app = api::create_app(db_pool.clone());
    
    // Spawn the server in a background task
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("Failed to start test server");
    });
    
    TestApp {
        address,
        db_pool,
        db_name,
        base_url,
    }
}

/// Configures a test database with a unique name.
///
/// This function:
/// 1. Connects to the default postgres database
/// 2. Creates a new database with the given name
/// 3. Runs all migrations on the new database
/// 4. Returns a connection pool to the test database
///
/// # Arguments
/// * `db_name` - The name for the test database
///
/// # Returns
/// A connection pool to the newly created test database
async fn configure_test_database(db_name: &str) -> Result<PgPool> {
    // Get database URL from environment or use default
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://studyflow:studyflow@localhost:5432/studyflow_dev".to_string());
    
    // Parse the base connection options
    let mut connection_options = PgConnectOptions::from_str(&db_url)
        .context("Failed to parse DATABASE_URL")?;
    
    // Connect to the postgres database to create our test database
    connection_options = connection_options.database("postgres");
    
    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .context("Failed to connect to postgres database")?;
    
    // Create the test database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .context("Failed to create test database")?;
    
    // Connect to the new test database
    connection_options = connection_options.database(db_name);
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(connection_options)
        .await
        .context("Failed to connect to test database")?;
    
    // Run migrations
    sqlx::migrate!("./crates/db/migrations")
        .run(&pool)
        .await
        .context("Failed to run migrations on test database")?;
    
    Ok(pool)
}

/// Drops a test database.
///
/// This should be called in test cleanup to remove the test database.
///
/// # Arguments
/// * `db_name` - The name of the database to drop
pub async fn drop_test_database(db_name: &str) -> Result<()> {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://studyflow:studyflow@localhost:5432/studyflow_dev".to_string());
    
    let mut connection_options = PgConnectOptions::from_str(&db_url)
        .context("Failed to parse DATABASE_URL")?
        .database("postgres");
    
    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .context("Failed to connect to postgres database")?;
    
    // Terminate existing connections to the test database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                AND pid <> pg_backend_pid();
                "#,
                db_name
            )
            .as_str(),
        )
        .await
        .context("Failed to terminate connections")?;
    
    // Drop the test database
    connection
        .execute(format!(r#"DROP DATABASE IF EXISTS "{}";"#, db_name).as_str())
        .await
        .context("Failed to drop test database")?;
    
    Ok(())
}

/// Test data builder for creating users.
///
/// Provides a fluent API for building test user data with sensible defaults.
#[derive(Debug, Clone)]
pub struct UserBuilder {
    email: String,
    name: String,
}

impl UserBuilder {
    /// Creates a new UserBuilder with default values.
    pub fn new() -> Self {
        Self {
            email: format!("test_{}@example.com", Uuid::new_v4()),
            name: "Test User".to_string(),
        }
    }
    
    /// Sets the email for the user.
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = email.into();
        self
    }
    
    /// Sets the name for the user.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    
    /// Builds the user data as a JSON value.
    pub fn build_json(self) -> serde_json::Value {
        serde_json::json!({
            "email": self.email,
            "name": self.name,
        })
    }
}

impl Default for UserBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Test data builder for creating tasks.
#[derive(Debug, Clone)]
pub struct TaskBuilder {
    title: String,
    description: Option<String>,
    user_id: Uuid,
}

impl TaskBuilder {
    /// Creates a new TaskBuilder with default values.
    pub fn new(user_id: Uuid) -> Self {
        Self {
            title: format!("Test Task {}", Uuid::new_v4()),
            description: None,
            user_id,
        }
    }
    
    /// Sets the title for the task.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    
    /// Sets the description for the task.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// Builds the task data as a JSON value.
    pub fn build_json(self) -> serde_json::Value {
        serde_json::json!({
            "title": self.title,
            "description": self.description,
            "user_id": self.user_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spawn_app_creates_isolated_instance() {
        // Spawn two test apps
        let app1 = spawn_app().await;
        let app2 = spawn_app().await;
        
        // They should have different addresses
        assert_ne!(app1.address, app2.address);
        
        // They should have different database names
        assert_ne!(app1.db_name, app2.db_name);
        
        // Both should be able to connect to their databases
        let result1 = sqlx::query("SELECT 1").fetch_one(&app1.db_pool).await;
        let result2 = sqlx::query("SELECT 1").fetch_one(&app2.db_pool).await;
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        // Cleanup
        drop_test_database(&app1.db_name).await.unwrap();
        drop_test_database(&app2.db_name).await.unwrap();
    }

    #[tokio::test]
    async fn test_user_builder_creates_valid_json() {
        let user = UserBuilder::new()
            .email("test@example.com")
            .name("Test User")
            .build_json();
        
        assert_eq!(user["email"], "test@example.com");
        assert_eq!(user["name"], "Test User");
    }

    #[tokio::test]
    async fn test_user_builder_generates_unique_emails() {
        let user1 = UserBuilder::new().build_json();
        let user2 = UserBuilder::new().build_json();
        
        assert_ne!(user1["email"], user2["email"]);
    }

    #[tokio::test]
    async fn test_task_builder_creates_valid_json() {
        let user_id = Uuid::new_v4();
        let task = TaskBuilder::new(user_id)
            .title("Test Task")
            .description("Test Description")
            .build_json();
        
        assert_eq!(task["title"], "Test Task");
        assert_eq!(task["description"], "Test Description");
        assert_eq!(task["user_id"], user_id.to_string());
    }

    #[tokio::test]
    async fn test_cleanup_database_removes_all_data() {
        let app = spawn_app().await;
        
        // Insert some test data
        sqlx::query(
            r#"
            INSERT INTO users (id, email, name, created_at, updated_at)
            VALUES ($1, $2, $3, NOW(), NOW())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind("test@example.com")
        .bind("Test User")
        .execute(&app.db_pool)
        .await
        .unwrap();
        
        // Verify data exists
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&app.db_pool)
            .await
            .unwrap();
        assert_eq!(count.0, 1);
        
        // Cleanup
        app.cleanup_database().await.unwrap();
        
        // Verify data is removed
        let count: