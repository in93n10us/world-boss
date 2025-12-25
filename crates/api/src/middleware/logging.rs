rust
//! Request/response logging middleware using tracing.
//!
//! This middleware provides structured logging for all HTTP requests and responses,
//! including timing information, status codes, and request metadata.

use axum::{
    body::Body,
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{info, warn};
use uuid::Uuid;

/// Middleware that logs HTTP requests and responses with structured tracing.
///
/// Captures:
/// - Request method, path, and matched route
/// - Request ID for correlation
/// - Response status code
/// - Request duration in milliseconds
/// - Client IP address (if available)
///
/// # Example
///
/// rust,no_run
/// use axum::{Router, middleware};
/// use crate::middleware::logging::logging_middleware;
///
/// let app = Router::new()
///     .layer(middleware::from_fn(logging_middleware));
/// 
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let start = Instant::now();
    
    // Generate a unique request ID for tracing
    let request_id = Uuid::new_v4();
    
    // Extract request metadata
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();
    
    // Get the matched route pattern if available (e.g., "/users/:id")
    let matched_path = request
        .extensions()
        .get::<MatchedPath>()
        .map(|mp| mp.as_str().to_string())
        .unwrap_or_else(|| path.clone());
    
    // Extract client IP from headers or connection info
    let client_ip = extract_client_ip(&request);
    
    // Log the incoming request
    info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        matched_path = %matched_path,
        client_ip = ?client_ip,
        "incoming request"
    );
    
    // Process the request
    let response = next.run(request).await;
    
    // Calculate request duration
    let duration = start.elapsed();
    let duration_ms = duration.as_millis();
    
    // Extract response status
    let status = response.status();
    let status_code = status.as_u16();
    
    // Log based on status code severity
    if status.is_server_error() {
        warn!(
            request_id = %request_id,
            method = %method,
            path = %path,
            matched_path = %matched_path,
            status = status_code,
            duration_ms = duration_ms,
            client_ip = ?client_ip,
            "request completed with server error"
        );
    } else if status.is_client_error() {
        warn!(
            request_id = %request_id,
            method = %method,
            path = %path,
            matched_path = %matched_path,
            status = status_code,
            duration_ms = duration_ms,
            client_ip = ?client_ip,
            "request completed with client error"
        );
    } else {
        info!(
            request_id = %request_id,
            method = %method,
            path = %path,
            matched_path = %matched_path,
            status = status_code,
            duration_ms = duration_ms,
            client_ip = ?client_ip,
            "request completed successfully"
        );
    }
    
    response
}

/// Extracts the client IP address from request headers or connection info.
///
/// Checks the following headers in order:
/// 1. X-Forwarded-For (proxy/load balancer)
/// 2. X-Real-IP (nginx proxy)
/// 3. Forwarded (RFC 7239)
///
/// Returns None if no IP address can be determined.
fn extract_client_ip(request: &Request<Body>) -> Option<String> {
    // Check X-Forwarded-For header (most common)
    if let Some(forwarded_for) = request.headers().get("x-forwarded-for") {
        if let Ok(value) = forwarded_for.to_str() {
            // X-Forwarded-For can contain multiple IPs, take the first one
            return value.split(',').next().map(|s| s.trim().to_string());
        }
    }
    
    // Check X-Real-IP header (nginx)
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(value) = real_ip.to_str() {
            return Some(value.to_string());
        }
    }
    
    // Check Forwarded header (RFC 7239)
    if let Some(forwarded) = request.headers().get("forwarded") {
        if let Ok(value) = forwarded.to_str() {
            // Parse "for=192.0.2.60;proto=http;by=203.0.113.43"
            for part in value.split(';') {
                if let Some(for_value) = part.trim().strip_prefix("for=") {
                    return Some(for_value.trim_matches('"').to_string());
                }
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        response::IntoResponse,
        routing::get,
        Router,
    };
    use tower::ServiceExt;
    
    async fn test_handler() -> impl IntoResponse {
        (StatusCode::OK, "test response")
    }
    
    async fn error_handler() -> impl IntoResponse {
        (StatusCode::INTERNAL_SERVER_ERROR, "error response")
    }
    
    #[tokio::test]
    async fn test_logging_middleware_success() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(logging_middleware));
        
        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    #[tokio::test]
    async fn test_logging_middleware_error() {
        let app = Router::new()
            .route("/error", get(error_handler))
            .layer(middleware::from_fn(logging_middleware));
        
        let request = Request::builder()
            .uri("/error")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    #[test]
    fn test_extract_client_ip_from_x_forwarded_for() {
        let request = Request::builder()
            .header("x-forwarded-for", "203.0.113.195, 70.41.3.18")
            .body(Body::empty())
            .unwrap();
        
        let ip = extract_client_ip(&request);
        assert_eq!(ip, Some("203.0.113.195".to_string()));
    }
    
    #[test]
    fn test_extract_client_ip_from_x_real_ip() {
        let request = Request::builder()
            .header("x-real-ip", "203.0.113.195")
            .body(Body::empty())
            .unwrap();
        
        let ip = extract_client_ip(&request);
        assert_eq!(ip, Some("203.0.113.195".to_string()));
    }
    
    #[test]
    fn test_extract_client_ip_from_forwarded() {
        let request = Request::builder()
            .header("forwarded", "for=192.0.2.60;proto=http;by=203.0.113.43")
            .body(Body::empty())
            .unwrap();
        
        let ip = extract_client_ip(&request);
        assert_eq!(ip, Some("192.0.2.60".to_string()));
    }
    
    #[test]
    fn test_extract_client_ip_no_headers() {
        let request = Request::builder()
            .body(Body::empty())
            .unwrap();
        
        let ip = extract_client_ip(&request);
        assert_eq!(ip, None);
    }
    
    #[test]
    fn test_extract_client_ip_priority() {
        // X-Forwarded-For should take priority
        let request = Request::builder()
            .header("x-forwarded-for", "203.0.113.195")
            .header("x-real-ip", "192.0.2.60")
            .body(Body::empty())
            .unwrap();
        
        let ip = extract_client_ip(&request);
        assert_eq!(ip, Some("203.0.113.195".to_string()));
    }
}