rust
//! Centralized error types with proper HTTP status mapping.
//!
//! This module provides a unified error handling approach for the API layer,
//! converting domain and infrastructure errors into appropriate HTTP responses.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;

/// API error type that can be converted into HTTP responses.
///
/// This enum represents all possible error conditions in the API layer,
/// with each variant mapping to an appropriate HTTP status code.
#[derive(Debug)]
pub enum ApiError {
    /// Bad request - client sent invalid data (400)
    BadRequest(String),
    
    /// Unauthorized - authentication required or failed (401)
    Unauthorized(String),
    
    /// Forbidden - authenticated but lacks permission (403)
    Forbidden(String),
    
    /// Not found - requested resource doesn't exist (404)
    NotFound(String),
    
    /// Conflict - request conflicts with current state (409)
    Conflict(String),
    
    /// Unprocessable entity - validation failed (422)
    UnprocessableEntity(String),
    
    /// Internal server error - unexpected error (500)
    InternalServerError(String),
    
    /// Service unavailable - temporary failure (503)
    ServiceUnavailable(String),
    
    /// Database error - wraps database-specific errors
    DatabaseError(String),
    
    /// Validation error - input validation failed
    ValidationError(Vec<ValidationErrorDetail>),
}

/// Detailed validation error information.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationErrorDetail {
    /// Field name that failed validation
    pub field: String,
    
    /// Human-readable error message
    pub message: String,
}

/// Standard error response body sent to clients.
#[derive(Serialize)]
struct ErrorResponse {
    /// Error type/code for programmatic handling
    error: String,
    
    /// Human-readable error message
    message: String,
    
    /// Optional additional details (e.g., validation errors)
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl ApiError {
    /// Creates a bad request error.
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    /// Creates an unauthorized error.
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }

    /// Creates a forbidden error.
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Forbidden(msg.into())
    }

    /// Creates a not found error.
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Creates a conflict error.
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    /// Creates an unprocessable entity error.
    pub fn unprocessable_entity(msg: impl Into<String>) -> Self {
        Self::UnprocessableEntity(msg.into())
    }

    /// Creates an internal server error.
    pub fn internal_server_error(msg: impl Into<String>) -> Self {
        Self::InternalServerError(msg.into())
    }

    /// Creates a service unavailable error.
    pub fn service_unavailable(msg: impl Into<String>) -> Self {
        Self::ServiceUnavailable(msg.into())
    }

    /// Creates a database error.
    pub fn database_error(msg: impl Into<String>) -> Self {
        Self::DatabaseError(msg.into())
    }

    /// Creates a validation error with multiple field errors.
    pub fn validation_error(errors: Vec<ValidationErrorDetail>) -> Self {
        Self::ValidationError(errors)
    }

    /// Returns the HTTP status code for this error.
    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::ValidationError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Returns the error type string for the response.
    fn error_type(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "bad_request",
            Self::Unauthorized(_) => "unauthorized",
            Self::Forbidden(_) => "forbidden",
            Self::NotFound(_) => "not_found",
            Self::Conflict(_) => "conflict",
            Self::UnprocessableEntity(_) => "unprocessable_entity",
            Self::ValidationError(_) => "validation_error",
            Self::InternalServerError(_) => "internal_server_error",
            Self::ServiceUnavailable(_) => "service_unavailable",
            Self::DatabaseError(_) => "database_error",
        }
    }

    /// Returns the error message.
    fn message(&self) -> String {
        match self {
            Self::BadRequest(msg)
            | Self::Unauthorized(msg)
            | Self::Forbidden(msg)
            | Self::NotFound(msg)
            | Self::Conflict(msg)
            | Self::UnprocessableEntity(msg)
            | Self::InternalServerError(msg)
            | Self::ServiceUnavailable(msg)
            | Self::DatabaseError(msg) => msg.clone(),
            Self::ValidationError(_) => "Validation failed".to_string(),
        }
    }

    /// Returns optional details for the error response.
    fn details(&self) -> Option<serde_json::Value> {
        match self {
            Self::ValidationError(errors) => {
                Some(serde_json::json!({ "validation_errors": errors }))
            }
            _ => None,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_type(), self.message())
    }
}

impl std::error::Error for ApiError {}

/// Convert ApiError into an Axum HTTP response.
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = ErrorResponse {
            error: self.error_type().to_string(),
            message: self.message(),
            details: self.details(),
        };

        // Log internal errors for debugging (don't expose details to client)
        if matches!(
            self,
            Self::InternalServerError(_) | Self::DatabaseError(_)
        ) {
            tracing::error!(
                error = ?self,
                status = %status,
                "Internal error occurred"
            );
        }

        (status, Json(body)).into_response()
    }
}

/// Convert sqlx errors into ApiError.
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => {
                Self::not_found("The requested resource was not found")
            }
            sqlx::Error::Database(db_err) => {
                // Check for common database constraint violations
                if let Some(code) = db_err.code() {
                    match code.as_ref() {
                        "23505" => {
                            // Unique violation
                            return Self::conflict("A resource with this value already exists");
                        }
                        "23503" => {
                            // Foreign key violation
                            return Self::bad_request("Referenced resource does not exist");
                        }
                        "23502" => {
                            // Not null violation
                            return Self::bad_request("Required field is missing");
                        }
                        _ => {}
                    }
                }
                
                tracing::error!(error = ?db_err, "Database error occurred");
                Self::database_error("A database error occurred")
            }
            _ => {
                tracing::error!(error = ?err, "Unexpected database error");
                Self::database_error("An unexpected database error occurred")
            }
        }
    }
}

/// Convert anyhow errors into ApiError.
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!(error = ?err, "Anyhow error occurred");
        Self::internal_server_error("An unexpected error occurred")
    }
}

/// Type alias for Results that use ApiError.
pub type ApiResult<T> = Result<T, ApiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            ApiError::bad_request("test").status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ApiError::unauthorized("test").status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ApiError::forbidden("test").status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            ApiError::not_found("test").status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ApiError::conflict("test").status_code(),
            StatusCode::CONFLICT
        );
        assert_eq!(
            ApiError::internal_server_error("test").status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_error_types() {
        assert_eq!(ApiError::bad_request("test").error_type(), "bad_request");
        assert_eq!(
            ApiError::unauthorized("test").error_type(),
            "unauthorized"
        );
        assert_eq!(ApiError::not_found("test").error_type(), "not_found");
    }

    #[test]
    fn test_validation_error_details() {
        let errors = vec![
            ValidationErrorDetail {
                field: "email".to_string(),
                message: "Invalid email format".to_string(),
            },
            ValidationErrorDetail {
                field: "password".to_string(),
                message: "Password too short".to_string(),
            },
        ];

        let api_error = ApiError::validation_error(errors.clone());
        let details = api_error.details();

        assert!(details.is_some());
        assert_eq!(api_error.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn test_error_display() {
        let error = ApiError::not_found("User not found");
        assert_eq!(error.to_string(), "not_found: User not found");
    }
}