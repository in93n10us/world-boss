rust
//! Route module aggregation and router setup.
//!
//! This module aggregates all route handlers and constructs the main application router
//! with proper middleware configuration. It serves as the central point for defining
//! the API structure and applying cross-cutting concerns like logging and CORS.

use axum::{
    routing::get,
    Router,
};
use tower_http::trace::TraceLayer;
use tower_http::cors::{CorsLayer, Any};

pub mod health;

/// Creates and configures the main application router.
///
/// This function assembles all route modules into a single router with appropriate
/// middleware layers applied. The router is configured with:
/// - Health check endpoints
/// - Request tracing for observability
/// - CORS configuration for cross-origin requests
///
/// # Arguments
///
/// * `state` - Application state containing shared resources like database pools
///
/// # Returns
///
/// A configured `Router` ready to be served by the Axum server
///
/// # Example
///
/// rust,no_run
/// use api::routes::create_router;
/// use api::AppState;
/// use std::sync::Arc;
///
/// # async fn example() {
/// let state = Arc::new(AppState::new());
/// let router = create_router(state);
/// # }
/// 
pub fn create_router<S>(state: S) -> Router
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        // Health check routes
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
        
        // Future route groups will be added here:
        // .nest("/api/v1/users", user_routes())
        // .nest("/api/v1/tasks", task_routes())
        // .nest("/api/v1/sessions", session_routes())
        
        // Apply middleware layers
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[derive(Clone)]
    struct TestState;

    #[tokio::test]
    async fn test_router_creation() {
        let state = TestState;
        let router = create_router(state);
        
        // Verify router can be created without panicking
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[tokio::test]
    async fn test_health_route_exists() {
        let state = TestState;
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_readiness_route_exists() {
        let state = TestState;
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_unknown_route_returns_404() {
        let state = TestState;
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/unknown")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}