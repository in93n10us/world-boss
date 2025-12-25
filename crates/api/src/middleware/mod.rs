rust
//! Middleware module for the API server.
//!
//! This module provides middleware components for cross-cutting concerns:
//! - Request/response logging
//! - Error handling
//! - Future authentication and authorization
//!
//! Middleware is applied to routes using Tower's layer system with Axum.

pub mod logging;

pub use logging::LoggingMiddleware;

// Future middleware modules will be added here:
// pub mod auth;
// pub mod rate_limit;
// pub mod cors;