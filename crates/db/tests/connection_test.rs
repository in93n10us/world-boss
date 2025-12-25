rust
//! Integration tests for database connection pool creation and basic query execution.
//!
//! These tests verify that:
//! - Connection pools can be created with valid configuration
//! - Basic queries can be executed against the database
//! - Connection pool handles errors appropriately
//! - Pool configuration parameters are respected
//!
//! # Test Database Setup
//!
//! These tests require a running PostgreSQL instance. Use docker-compose to start:
//! bash
//! docker-compose up -d postgres
//! 

use anyhow::{Context, Result};
use sqlx::{PgPool, Row};
use std::time::Duration;

/// Test database URL for integration tests.
/// This should match the docker-compose configuration.
const TEST_DATABASE_URL: &str = "postgres://studyflow:studyflow@localhost:5432/studyflow_dev";

/// Helper function to create a test connection pool with default settings.
async fn create_test_pool() -> Result<PgPool> {
    db::connection::create_pool(TEST_DATABASE_URL, None, None, None)
        .await
        .context("Failed to create test connection pool")
}

/// Helper function to create a test connection pool with custom settings.
async fn create_test_pool_with_config(
    max_connections: Option<u32>,
    min_connections: Option<u32>,
    acquire_timeout: Option<Duration>,
) -> Result<PgPool> {
    db::connection::create_pool(
        TEST_DATABASE_URL,
        max_connections,
        min_connections,
        acquire_timeout,
    )
    .await
    .context("Failed to create test connection pool with custom config")
}

#[tokio::test]
async fn test_create_pool_with_valid_url() {
    // Arrange & Act
    let result = create_test_pool().await;

    // Assert
    assert!(
        result.is_ok(),
        "Should successfully create connection pool with valid database URL"
    );

    let pool = result.unwrap();
    assert!(
        !pool.is_closed(),
        "Connection pool should not be closed after creation"
    );
}

#[tokio::test]
async fn test_create_pool_with_invalid_url() {
    // Arrange
    let invalid_url = "postgres://invalid:invalid@localhost:9999/nonexistent";

    // Act
    let result = db::connection::create_pool(invalid_url, None, None, None).await;

    // Assert
    assert!(
        result.is_err(),
        "Should fail to create connection pool with invalid database URL"
    );
}

#[tokio::test]
async fn test_create_pool_with_malformed_url() {
    // Arrange
    let malformed_url = "not-a-valid-url";

    // Act
    let result = db::connection::create_pool(malformed_url, None, None, None).await;

    // Assert
    assert!(
        result.is_err(),
        "Should fail to create connection pool with malformed URL"
    );
}

#[tokio::test]
async fn test_basic_query_execution() {
    // Arrange
    let pool = create_test_pool()
        .await
        .expect("Failed to create connection pool");

    // Act - Execute a simple query
    let result = sqlx::query("SELECT 1 as value")
        .fetch_one(&pool)
        .await;

    // Assert
    assert!(result.is_ok(), "Should successfully execute basic query");

    let row = result.unwrap();
    let value: i32 = row.get("value");
    assert_eq!(value, 1, "Query should return expected value");
}

#[tokio::test]
async fn test_query_current_database() {
    // Arrange
    let pool = create_test_pool()
        .await
        .expect("Failed to create connection pool");

    // Act - Query the current database name
    let result = sqlx::query("SELECT current_database() as db_name")
        .fetch_one(&pool)
        .await;

    // Assert
    assert!(
        result.is_ok(),
        "Should successfully query current database name"
    );

    let row = result.unwrap();
    let db_name: String = row.get("db_name");
    assert_eq!(
        db_name, "studyflow_dev",
        "Should be connected to the correct database"
    );
}

#[tokio::test]
async fn test_query_database_version() {
    // Arrange
    let pool = create_test_pool()
        .await
        .expect("Failed to create connection pool");

    // Act - Query PostgreSQL version
    let result = sqlx::query("SELECT version() as version")
        .fetch_one(&pool)
        .await;

    // Assert
    assert!(
        result.is_ok(),
        "Should successfully query database version"
    );

    let row = result.unwrap();
    let version: String = row.get("version");
    assert!(
        version.contains("PostgreSQL"),
        "Version string should contain 'PostgreSQL'"
    );
}

#[tokio::test]
async fn test_multiple_queries_on_same_pool() {
    // Arrange
    let pool = create_test_pool()
        .await
        .expect("Failed to create connection pool");

    // Act - Execute multiple queries
    let result1 = sqlx::query("SELECT 1 as value").fetch_one(&pool).await;
    let result2 = sqlx::query("SELECT 2 as value").fetch_one(&pool).await;
    let result3 = sqlx::query("SELECT 3 as value").fetch_one(&pool).await;

    // Assert
    assert!(result1.is_ok(), "First query should succeed");
    assert!(result2.is_ok(), "Second query should succeed");
    assert!(result3.is_ok(), "Third query should succeed");

    let value1: i32 = result1.unwrap().get("value");
    let value2: i32 = result2.unwrap().get("value");
    let value3: i32 = result3.unwrap().get("value");

    assert_eq!(value1, 1, "First query should return 1");
    assert_eq!(value2, 2, "Second query should return 2");
    assert_eq!(value3, 3, "Third query should return 3");
}

#[tokio::test]
async fn test_invalid_query_execution() {
    // Arrange
    let pool = create_test_pool()
        .await
        .expect("Failed to create connection pool");

    // Act - Execute an invalid SQL query
    let result = sqlx::query("SELECT * FROM nonexistent_table")
        .fetch_one(&pool)
        .await;

    // Assert
    assert!(
        result.is_err(),
        "Should fail when executing query on non-existent table"
    );
}

#[tokio::test]
async fn test_query_with_syntax_error() {
    // Arrange
    let pool = create_test_pool()
        .await
        .expect("Failed to create connection pool");

    // Act - Execute a query with syntax error
    let result = sqlx::query("INVALID SQL SYNTAX").fetch_one(&pool).await;

    // Assert
    assert!(
        result.is_err(),
        "Should fail when executing query with syntax error"
    );
}

#[tokio::test]
async fn test_pool_with_custom_max_connections() {
    // Arrange & Act
    let result = create_test_pool_with_config(Some(5), None, None).await;

    // Assert
    assert!(
        result.is_ok(),
        "Should successfully create pool with custom max connections"
    );

    let pool = result.unwrap();
    
    // Verify pool works by executing a query
    let query_result = sqlx::query("SELECT 1").fetch_one(&pool).await;
    assert!(
        query_result.is_ok(),
        "Should be able to execute queries with custom pool configuration"
    );
}

#[tokio::test]
async fn test_pool_with_custom_min_connections() {
    // Arrange & Act
    let result = create_test_pool_with_config(None, Some(2), None).await;

    // Assert
    assert!(
        result.is_ok(),
        "Should successfully create pool with custom min connections"
    );

    let pool = result.unwrap();
    
    // Verify pool works
    let query_result = sqlx::query("SELECT 1").fetch_one(&pool).await;
    assert!(
        query_result.is_ok(),
        "Should be able to execute queries with custom min connections"
    );
}

#[tokio::test]
async fn test_pool_with_custom_acquire_timeout() {
    // Arrange & Act
    let result = create_test_pool_with_config(None, None, Some(Duration::from_secs(10))).await;

    // Assert
    assert!(
        result.is_ok(),
        "Should successfully create pool with custom acquire timeout"
    );

    let pool = result.unwrap();
    
    // Verify pool works
    let query_result = sqlx::query("SELECT 1").fetch_one(&pool).await;
    assert!(
        query_result.is_ok(),
        "Should be able to execute queries with custom acquire timeout"
    );
}

#[tokio::test]
async fn test_pool_with_all_custom_settings() {
    // Arrange & Act
    let result = create_test_pool_with_config(
        Some(10),
        Some(2),
        Some(Duration::from_secs(15)),
    )
    .await;

    // Assert
    assert!(
        result.is_ok(),
        "Should successfully create pool with all custom settings"
    );

    let pool = result.unwrap();
    
    // Verify pool works
    let query_result = sqlx::query("SELECT 1").fetch_one(&pool).await;
    assert!(
        query_result.is_ok(),
        "Should be able to execute queries with all custom settings"
    );
}

#[tokio::test]
async fn test_concurrent_queries() {
    // Arrange
    let pool = create_test_pool()
        .await
        .expect("Failed to create connection pool");

    // Act - Execute multiple queries concurrently
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let pool_clone = pool.clone();
            tokio::spawn(async move {
                sqlx::query("SELECT $1 as value")
                    .bind(i)
                    .fetch_one(&pool_clone)
                    .await
            })
        })
        .collect();

    // Wait for all queries to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // Assert
    for (i, result) in results.iter().enumerate() {
        assert!(
            result.is_ok(),
            "Concurrent query {} should succeed",
            i
        );
        
        let query_result = result.as_ref().unwrap();
        assert!(
            query_result.is_ok(),
            "Query result {} should be ok",
            i
        );
    }
}

#[tokio::test]
async fn test_pool_connection_health_check() {
    // Arrange
    let pool = create_test_pool()
        .await
        .expect("Failed to create connection pool");

    // Act - Acquire a connection and verify it's healthy
    let conn_result = pool.acquire().await;

    // Assert
    assert!(
        conn_result.is_ok(),
        "Should be able to acquire connection from pool"
    );

    let mut conn = conn_result.unwrap();
    
    // Verify connection is usable
    let ping_result = sqlx::query("SELECT 1").fetch_one(&mut *conn).await;
    assert!(
        ping_result.is_ok(),
        "Should be able to execute query on acquired connection"
    );
}

#[tokio::test]
async fn test_pool_closes_properly() {
    // Arrange
    let pool = create_test_pool()
        .await
        .expect("Failed to create connection pool");

    // Act
    pool.close().await;

    // Assert
    assert!(
        pool.is_closed(),
        "Pool should be closed after calling close()"
    );

    // Verify that queries fail on closed pool
    let result = sqlx::query("SELECT 1").fetch_one(&pool).await;
    assert!(
        result.is_err(),
        "Queries should fail on closed connection pool"
    );
}

#[tokio::test]
async fn test_transaction_support() {
    // Arrange
    let pool = create_test_pool()
        .await
        .expect("Failed to create connection pool");

    // Act - Begin a transaction
    let tx_result = pool.begin().await;

    // Assert
    assert!(
        tx_result.is_ok(),
        "Should be able to begin transaction on pool"
    );

    let mut tx = tx_result.unwrap();
    
    // Execute query within transaction
    let query_result = sqlx::query("SELECT 1 as value")
        .fetch_one(&mut *tx)
        .await;
    
    assert!(
        query_result.is_ok(),
        "Should be able to execute queries within transaction"
    );

    // Commit transaction
    let commit_result = tx.commit().await;
    assert!(
        commit_result.is_ok(),
        "Should be able to commit transaction"
    );
}

#[tokio::test]
async fn test_prepared_statement_execution() {
    // Arrange
    let pool = create_test_pool()
        .await
        .expect("Failed to create connection pool");

    // Act - Execute a prepared statement with parameters
    let result = sqlx::query("SELECT $1::int as value")
        .bind(42)
        .fetch_one(&pool)
        .await;

    // Assert
    assert!(
        result.is_ok(),
        "Should successfully execute prepared statement"
    );

    let row = result.unwrap();
    let value: i32 = row.get("value");
    assert_eq!(value, 42, "Prepared statement should return bound value");
}

#[tokio::test]
async fn test_multiple_parameter_binding() {
    // Arrange
    let pool = create_test_pool()
        .await
        .expect("Failed to create connection pool");

    // Act - Execute query with multiple parameters
    let result = sqlx::query("SELECT $1::int + $2::int as sum")
        .bind(10)
        .bind(20)
        .fetch_one(&pool)
        .await;

    // Assert
    assert!(
        result.is_ok(),
        "Should successfully execute query with multiple parameters"
    );

    let row = result.unwrap();
    let sum: i32 = row.get("sum");
    assert_eq!(sum, 30, "Query should correctly compute sum of parameters");
}

#[tokio::test]
async fn test_fetch_all_results() {
    // Arrange
    let pool = create_test_pool()
        .await
        .expect("Failed to create connection pool");

    // Act - Fetch multiple rows
    let result = sqlx::query("SELECT generate_series(1, 5) as num")
        .fetch_all(&pool)
        .await;

    // Assert
    assert!(
        result.is_ok(),
        "Should successfully fetch all results"
    );

    let rows = result.unwrap();
    assert_eq!(rows.len(), 5, "Should return 5 rows");

    for (i, row) in rows.iter().enumerate() {
        let num: i32 = row.get("num");
        assert_eq!(num, (i + 1) as i32, "Row {} should have correct value", i);
    }
}

#[tokio::test]
async fn test_fetch_