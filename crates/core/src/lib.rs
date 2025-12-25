rust
//! Core business logic and domain models for StudyFlow.
//!
//! This crate contains the domain entities, business rules, and port definitions
//! that are independent of infrastructure concerns. It follows hexagonal architecture
//! principles where the core domain is isolated from external dependencies.
//!
//! # Architecture
//!
//! - `domain`: Domain entities and value objects (User, Task, StudySession)
//! - `ports`: Port traits defining interfaces for repositories and services
//! - Business logic and validation rules are encapsulated within domain models
//!
//! # Example
//!
//! rust
//! use studyflow_core::domain::Task;
//! use chrono::Utc;
//!
//! // Domain models can be used independently of infrastructure
//! let task = Task::new(
//!     "Study Rust async programming".to_string(),
//!     Some("Complete chapters 1-3".to_string()),
//!     60,
//!     Utc::now().naive_utc(),
//! );
//! 

#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]
#![forbid(unsafe_code)]

/// Domain models and entities representing core business concepts.
///
/// Contains the fundamental building blocks of the application:
/// - User: Represents a student using the application
/// - Task: Represents a study task with time estimates and deadlines
/// - StudySession: Represents an actual study session with time tracking
/// - Tag: Represents categorization labels for tasks
///
/// All domain models include validation logic and business rules.
pub mod domain;

/// Port traits defining interfaces for external dependencies.
///
/// Following hexagonal architecture, ports define the contracts that
/// infrastructure adapters must implement. This allows the core domain
/// to remain independent of specific implementations.
///
/// Includes:
/// - Repository traits for data persistence
/// - Service traits for external integrations
/// - Event publisher traits for domain events
pub mod ports;

// Re-export commonly used types for convenience
pub use domain::{StudySession, Tag, Task, User};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_exports() {
        // Verify that main types are accessible from crate root
        let _user_type = std::any::type_name::<User>();
        let _task_type = std::any::type_name::<Task>();
        let _session_type = std::any::type_name::<StudySession>();
        let _tag_type = std::any::type_name::<Tag>();
    }
}