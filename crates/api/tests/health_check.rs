rust
//! Integration tests for the health check endpoint.
//!
//! These tests verify that the health check endpoint returns the correct
//! status code and response body format. The health check is critical for
//! load balancers, monitoring systems, and orchestration platforms.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt; // for `oneshot` and `ready`

/// Helper function to create a test application instance.
///
/// This creates a fresh router with all routes configured, suitable for
/// testing without requiring a running server or database connection.
fn create_test_app() -> axum::Router {
    api::routes::create_router()
}

/// Helper function to parse response body as JSON.
///
/// # Arguments
/// * `body` - The response body bytes
///
/// # Returns
/// Parsed JSON value or panics if parsing fails
async fn body_to_json(body: axum::body::Body) -> Value {
    let bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .expect("Failed to read response body");
    serde_json::from_slice(&bytes).expect("Failed to parse JSON response")
}

#[tokio::test]
async fn test_health_check_returns_200_ok() {
    // Arrange
    let app = create_test_app();
    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .expect("Failed to build request");

    // Act
    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Health check should return 200 OK"
    );
}

#[tokio::test]
async fn test_health_check_response_body_structure() {
    // Arrange
    let app = create_test_app();
    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .expect("Failed to build request");

    // Act
    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");
    let body = body_to_json(response.into_body()).await;

    // Assert
    assert!(
        body.is_object(),
        "Response body should be a JSON object"
    );
    assert!(
        body.get("status").is_some(),
        "Response should contain 'status' field"
    );
    assert!(
        body.get("service").is_some(),
        "Response should contain 'service' field"
    );
    assert!(
        body.get("version").is_some(),
        "Response should contain 'version' field"
    );
}

#[tokio::test]
async fn test_health_check_status_field_value() {
    // Arrange
    let app = create_test_app();
    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .expect("Failed to build request");

    // Act
    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");
    let body = body_to_json(response.into_body()).await;

    // Assert
    let status = body
        .get("status")
        .and_then(|v| v.as_str())
        .expect("Status field should be a string");
    assert_eq!(
        status, "healthy",
        "Status field should have value 'healthy'"
    );
}

#[tokio::test]
async fn test_health_check_service_field_value() {
    // Arrange
    let app = create_test_app();
    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .expect("Failed to build request");

    // Act
    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");
    let body = body_to_json(response.into_body()).await;

    // Assert
    let service = body
        .get("service")
        .and_then(|v| v.as_str())
        .expect("Service field should be a string");
    assert_eq!(
        service, "studyflow-api",
        "Service field should have value 'studyflow-api'"
    );
}

#[tokio::test]
async fn test_health_check_version_field_format() {
    // Arrange
    let app = create_test_app();
    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .expect("Failed to build request");

    // Act
    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");
    let body = body_to_json(response.into_body()).await;

    // Assert
    let version = body
        .get("version")
        .and_then(|v| v.as_str())
        .expect("Version field should be a string");
    assert!(
        !version.is_empty(),
        "Version field should not be empty"
    );
    // Version should follow semver format (basic check)
    assert!(
        version.chars().any(|c| c.is_numeric()),
        "Version should contain numeric characters"
    );
}

#[tokio::test]
async fn test_health_check_content_type_is_json() {
    // Arrange
    let app = create_test_app();
    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .expect("Failed to build request");

    // Act
    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");

    // Assert
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .expect("Content-Type header should be present");
    assert!(
        content_type.contains("application/json"),
        "Content-Type should be application/json, got: {}",
        content_type
    );
}

#[tokio::test]
async fn test_health_check_multiple_requests() {
    // Arrange & Act - Test that multiple requests work correctly
    for _ in 0..5 {
        let app = create_test_app();
        let request = Request::builder()
            .uri("/health")
            .method("GET")
            .body(Body::empty())
            .expect("Failed to build request");

        let response = app
            .oneshot(request)
            .await
            .expect("Failed to execute request");

        // Assert
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Each health check request should return 200 OK"
        );
    }
}

#[tokio::test]
async fn test_health_check_with_query_parameters() {
    // Arrange - Health check should ignore query parameters
    let app = create_test_app();
    let request = Request::builder()
        .uri("/health?foo=bar&baz=qux")
        .method("GET")
        .body(Body::empty())
        .expect("Failed to build request");

    // Act
    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Health check should work with query parameters"
    );
}

#[tokio::test]
async fn test_health_check_post_method_not_allowed() {
    // Arrange - Health check should only accept GET requests
    let app = create_test_app();
    let request = Request::builder()
        .uri("/health")
        .method("POST")
        .body(Body::empty())
        .expect("Failed to build request");

    // Act
    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(
        response.status(),
        StatusCode::METHOD_NOT_ALLOWED,
        "POST requests to health check should return 405 Method Not Allowed"
    );
}

#[tokio::test]
async fn test_health_check_put_method_not_allowed() {
    // Arrange
    let app = create_test_app();
    let request = Request::builder()
        .uri("/health")
        .method("PUT")
        .body(Body::empty())
        .expect("Failed to build request");

    // Act
    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(
        response.status(),
        StatusCode::METHOD_NOT_ALLOWED,
        "PUT requests to health check should return 405 Method Not Allowed"
    );
}

#[tokio::test]
async fn test_health_check_delete_method_not_allowed() {
    // Arrange
    let app = create_test_app();
    let request = Request::builder()
        .uri("/health")
        .method("DELETE")
        .body(Body::empty())
        .expect("Failed to build request");

    // Act
    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(
        response.status(),
        StatusCode::METHOD_NOT_ALLOWED,
        "DELETE requests to health check should return 405 Method Not Allowed"
    );
}

#[tokio::test]
async fn test_health_check_response_matches_expected_schema() {
    // Arrange
    let app = create_test_app();
    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .expect("Failed to build request");

    // Act
    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");
    let body = body_to_json(response.into_body()).await;

    // Assert - Verify complete expected schema
    let expected_keys = vec!["status", "service", "version"];
    for key in expected_keys {
        assert!(
            body.get(key).is_some(),
            "Response should contain '{}' field",
            key
        );
    }

    // Verify no unexpected fields (strict schema validation)
    let actual_keys: Vec<&str> = body
        .as_object()
        .expect("Body should be an object")
        .keys()
        .map(|s| s.as_str())
        .collect();
    assert_eq!(
        actual_keys.len(),
        3,
        "Response should contain exactly 3 fields"
    );
}

#[tokio::test]
async fn test_health_check_head_request() {
    // Arrange - HEAD requests should work like GET but without body
    let app = create_test_app();
    let request = Request::builder()
        .uri("/health")
        .method("HEAD")
        .body(Body::empty())
        .expect("Failed to build request");

    // Act
    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");

    // Assert - HEAD should return 200 or 405 depending on implementation
    // Most implementations return 405 for HEAD if not explicitly handled
    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::METHOD_NOT_ALLOWED,
        "HEAD request should return either 200 OK or 405 Method Not Allowed"
    );
}

#[tokio::test]
async fn test_health_check_with_accept_header() {
    // Arrange - Test with explicit Accept header
    let app = create_test_app();
    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .header("Accept", "application/json")
        .body(Body::empty())
        .expect("Failed to build request");

    // Act
    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Health check should accept application/json"
    );
}

#[tokio::test]
async fn test_health_check_response_is_valid_json() {
    // Arrange
    let app = create_test_app();
    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .expect("Failed to build request");

    // Act
    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");

    // Assert - Should parse without error
    let parse_result = serde_json::from_slice::<Value>(&bytes);
    assert!(
        parse_result.is_ok(),
        "Response body should be valid JSON: {:?}",
        parse_result.err()
    );
}