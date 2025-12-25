rust
//! Health check endpoint for monitoring and readiness probes.
//!
//! This module provides a simple health check endpoint that can be used by:
//! - Load balancers to determine if the service is ready to receive traffic
//! - Monitoring systems to track service availability
//! - Kubernetes liveness and readiness probes
//!
//! The endpoint returns basic service information and can be extended to include
//! database connectivity checks and other dependency health status.

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

/// Response structure for the health check endpoint.
///
/// Contains basic service status information that can be extended
/// with additional health metrics as needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall service status
    pub status: String,
    /// Service name identifier
    pub service: String,
    /// API version
    pub version: String,
    /// Optional timestamp of the health check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl Default for HealthResponse {
    fn default() -> Self {
        Self {
            status: "healthy".to_string(),
            service: "studyflow-api".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        }
    }
}

/// Basic health check handler.
///
/// Returns a 200 OK response with service status information.
/// This endpoint should always return successfully unless the service
/// is completely unable to process requests.
///
/// # Example Response
///
/// json
/// {
///   "status": "healthy",
///   "service": "studyflow-api",
///   "version": "0.1.0",
///   "timestamp": "2024-01-15T10:30:00Z"
/// }
/// 
///
/// # Returns
///
/// * `200 OK` - Service is healthy and ready to accept requests
#[tracing::instrument(name = "health_check")]
pub async fn health_check() -> impl IntoResponse {
    tracing::debug!("Health check endpoint called");
    
    let response = HealthResponse::default();
    
    (StatusCode::OK, Json(response))
}

/// Readiness check handler.
///
/// Similar to health check but can be extended to verify that all
/// dependencies (database, external services) are available.
/// Currently returns the same response as health_check but can be
/// enhanced to include dependency checks.
///
/// # Returns
///
/// * `200 OK` - Service is ready to accept traffic
/// * `503 Service Unavailable` - Service is not ready (future enhancement)
#[tracing::instrument(name = "readiness_check")]
pub async fn readiness_check() -> impl IntoResponse {
    tracing::debug!("Readiness check endpoint called");
    
    let response = HealthResponse::default();
    
    (StatusCode::OK, Json(response))
}

/// Liveness check handler.
///
/// Minimal check to verify the service process is running.
/// This should only fail if the service is completely unresponsive.
///
/// # Returns
///
/// * `200 OK` - Service process is alive
#[tracing::instrument(name = "liveness_check")]
pub async fn liveness_check() -> impl IntoResponse {
    tracing::trace!("Liveness check endpoint called");
    
    // Minimal response for liveness - just confirm we can respond
    (StatusCode::OK, Json(serde_json::json!({
        "status": "alive"
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check_returns_ok() {
        let response = health_check().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_response_default() {
        let response = HealthResponse::default();
        assert_eq!(response.status, "healthy");
        assert_eq!(response.service, "studyflow-api");
        assert!(!response.version.is_empty());
        assert!(response.timestamp.is_some());
    }

    #[tokio::test]
    async fn test_readiness_check_returns_ok() {
        let response = readiness_check().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_liveness_check_returns_ok() {
        let response = liveness_check().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            service: "test-service".to_string(),
            version: "1.0.0".to_string(),
            timestamp: Some("2024-01-15T10:30:00Z".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("test-service"));
        assert!(json.contains("1.0.0"));
    }
}