rust
//! Port traits for repository and service abstractions.
//!
//! This module defines the port interfaces following hexagonal architecture principles.
//! Ports define the boundaries between the core business logic and external adapters
//! (database, external services, etc.).
//!
//! # Architecture
//!
//! - **Repositories**: Data persistence abstractions
//! - **Services**: External service abstractions (notifications, email, etc.)
//!
//! These traits are implemented by adapters in other crates (e.g., `db` crate).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{StudySession, Task, User};

/// Result type alias for repository operations.
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// Errors that can occur during repository operations.
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    /// Entity not found in the repository.
    #[error("Entity not found: {entity_type} with id {id}")]
    NotFound {
        entity_type: String,
        id: String,
    },

    /// Duplicate entity (e.g., unique constraint violation).
    #[error("Duplicate entity: {0}")]
    Duplicate(String),

    /// Database connection error.
    #[error("Database connection error: {0}")]
    ConnectionError(String),

    /// General database error.
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Validation error.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Repository port for User entity operations.
///
/// Defines the contract for user data persistence without coupling
/// to a specific database implementation.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Create a new user.
    ///
    /// # Arguments
    ///
    /// * `user` - The user to create
    ///
    /// # Returns
    ///
    /// The created user with generated ID and timestamps.
    ///
    /// # Errors
    ///
    /// Returns `RepositoryError::Duplicate` if email already exists.
    async fn create(&self, user: User) -> RepositoryResult<User>;

    /// Find a user by their unique ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The user's UUID
    ///
    /// # Returns
    ///
    /// The user if found, or `RepositoryError::NotFound`.
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<User>;

    /// Find a user by their email address.
    ///
    /// # Arguments
    ///
    /// * `email` - The user's email address
    ///
    /// # Returns
    ///
    /// The user if found, or `RepositoryError::NotFound`.
    async fn find_by_email(&self, email: &str) -> RepositoryResult<User>;

    /// Update an existing user.
    ///
    /// # Arguments
    ///
    /// * `user` - The user with updated fields
    ///
    /// # Returns
    ///
    /// The updated user.
    ///
    /// # Errors
    ///
    /// Returns `RepositoryError::NotFound` if user doesn't exist.
    async fn update(&self, user: User) -> RepositoryResult<User>;

    /// Delete a user by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The user's UUID
    ///
    /// # Errors
    ///
    /// Returns `RepositoryError::NotFound` if user doesn't exist.
    async fn delete(&self, id: Uuid) -> RepositoryResult<()>;

    /// List all users with optional pagination.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of users to return
    /// * `offset` - Number of users to skip
    ///
    /// # Returns
    ///
    /// A vector of users.
    async fn list(&self, limit: i64, offset: i64) -> RepositoryResult<Vec<User>>;
}

/// Repository port for Task entity operations.
#[async_trait]
pub trait TaskRepository: Send + Sync {
    /// Create a new task.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to create
    ///
    /// # Returns
    ///
    /// The created task with generated ID and timestamps.
    async fn create(&self, task: Task) -> RepositoryResult<Task>;

    /// Find a task by its unique ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The task's UUID
    ///
    /// # Returns
    ///
    /// The task if found, or `RepositoryError::NotFound`.
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Task>;

    /// Find all tasks for a specific user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's UUID
    /// * `limit` - Maximum number of tasks to return
    /// * `offset` - Number of tasks to skip
    ///
    /// # Returns
    ///
    /// A vector of tasks belonging to the user.
    async fn find_by_user_id(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> RepositoryResult<Vec<Task>>;

    /// Find tasks by status for a specific user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's UUID
    /// * `status` - The task status to filter by
    ///
    /// # Returns
    ///
    /// A vector of tasks matching the status.
    async fn find_by_user_and_status(
        &self,
        user_id: Uuid,
        status: &str,
    ) -> RepositoryResult<Vec<Task>>;

    /// Find tasks due within a date range.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's UUID
    /// * `start` - Start of date range
    /// * `end` - End of date range
    ///
    /// # Returns
    ///
    /// A vector of tasks due within the range.
    async fn find_by_due_date_range(
        &self,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<Vec<Task>>;

    /// Update an existing task.
    ///
    /// # Arguments
    ///
    /// * `task` - The task with updated fields
    ///
    /// # Returns
    ///
    /// The updated task.
    ///
    /// # Errors
    ///
    /// Returns `RepositoryError::NotFound` if task doesn't exist.
    async fn update(&self, task: Task) -> RepositoryResult<Task>;

    /// Delete a task by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The task's UUID
    ///
    /// # Errors
    ///
    /// Returns `RepositoryError::NotFound` if task doesn't exist.
    async fn delete(&self, id: Uuid) -> RepositoryResult<()>;

    /// Mark a task as completed.
    ///
    /// # Arguments
    ///
    /// * `id` - The task's UUID
    /// * `completed_at` - The completion timestamp
    ///
    /// # Returns
    ///
    /// The updated task.
    async fn mark_completed(&self, id: Uuid, completed_at: DateTime<Utc>)
        -> RepositoryResult<Task>;
}

/// Repository port for StudySession entity operations.
#[async_trait]
pub trait StudySessionRepository: Send + Sync {
    /// Create a new study session.
    ///
    /// # Arguments
    ///
    /// * `session` - The study session to create
    ///
    /// # Returns
    ///
    /// The created session with generated ID and timestamps.
    async fn create(&self, session: StudySession) -> RepositoryResult<StudySession>;

    /// Find a study session by its unique ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The session's UUID
    ///
    /// # Returns
    ///
    /// The session if found, or `RepositoryError::NotFound`.
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<StudySession>;

    /// Find all study sessions for a specific user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's UUID
    /// * `limit` - Maximum number of sessions to return
    /// * `offset` - Number of sessions to skip
    ///
    /// # Returns
    ///
    /// A vector of study sessions belonging to the user.
    async fn find_by_user_id(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> RepositoryResult<Vec<StudySession>>;

    /// Find study sessions for a specific task.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The task's UUID
    ///
    /// # Returns
    ///
    /// A vector of study sessions for the task.
    async fn find_by_task_id(&self, task_id: Uuid) -> RepositoryResult<Vec<StudySession>>;

    /// Find study sessions within a date range.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's UUID
    /// * `start` - Start of date range
    /// * `end` - End of date range
    ///
    /// # Returns
    ///
    /// A vector of study sessions within the range.
    async fn find_by_date_range(
        &self,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<Vec<StudySession>>;

    /// Update an existing study session.
    ///
    /// # Arguments
    ///
    /// * `session` - The session with updated fields
    ///
    /// # Returns
    ///
    /// The updated session.
    ///
    /// # Errors
    ///
    /// Returns `RepositoryError::NotFound` if session doesn't exist.
    async fn update(&self, session: StudySession) -> RepositoryResult<StudySession>;

    /// Delete a study session by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The session's UUID
    ///
    /// # Errors
    ///
    /// Returns `RepositoryError::NotFound` if session doesn't exist.
    async fn delete(&self, id: Uuid) -> RepositoryResult<()>;

    /// Calculate total study time for a user within a date range.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's UUID
    /// * `start` - Start of date range
    /// * `end` - End of date range
    ///
    /// # Returns
    ///
    /// Total duration in minutes.
    async fn calculate_total_duration(
        &self,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<i32>;
}

/// Service port for notification operations.
///
/// Abstracts notification delivery mechanisms (email, push, SMS, etc.).
#[async_trait]
pub trait NotificationService: Send + Sync {
    /// Send a task reminder notification.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user to notify
    /// * `task_id` - The task to remind about
    /// * `message` - The notification message
    async fn send_task_reminder(
        &self,
        user_id: Uuid,
        task_id: Uuid,
        message: &str,
    ) -> Result<(), NotificationError>;

    /// Send a study session reminder.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user to notify
    /// * `session_id` - The session to remind about
    /// * `message` - The notification message
    async fn send_session_reminder(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        message: &str,
    ) -> Result<(), NotificationError>;

    /// Send a progress update notification.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user to notify
    /// * `message` - The progress message
    async fn send_progress_update(
        &self,
        user_id: Uuid,
        message: &str,
    ) -> Result<(), NotificationError>;
}

/// Errors that can occur during notification operations.
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    /// Failed to send notification.
    #[error("Failed to send notification: {0}")]
    SendFailed(String),

    /// Invalid recipient.
    #[error("Invalid recipient: {0}")]
    InvalidRecipient(String),

    /// Service unavailable.
    #[error("Notification service unavailable: {0}")]
    ServiceUnavailable(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_error_display() {
        let error = RepositoryError::NotFound {
            entity_type: "User".to_string(),
            id: "123".to_string(),
        };
        assert_eq!(error.to_string(), "Entity not found: User with id 123");

        let error = RepositoryError::Duplicate("email already exists".to_string());
        assert_eq!(error.to_string(), "Duplicate entity: email already exists");
    }

    #[test]
    fn test_notification_error_display() {
        let error = NotificationError::SendFailed("network timeout".to_string());
        assert_eq!(
            error.to_string(),
            "Failed to send notification: network timeout"
        );
    }
}