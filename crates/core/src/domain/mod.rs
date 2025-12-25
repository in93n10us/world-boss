rust
//! Unit tests for domain models validation and business rules.
//!
//! These tests verify that domain entities enforce their invariants correctly,
//! validate input data, and maintain business rules integrity.

#[cfg(test)]
mod tests {
    use super::super::*;
    use chrono::{Duration, Utc};
    use validator::Validate;

    // ============================================================================
    // User Tests
    // ============================================================================

    mod user_tests {
        use super::*;

        #[test]
        fn test_user_creation_with_valid_data() {
            let user = User {
                id: Uuid::new_v4(),
                email: "test@example.com".to_string(),
                name: "Test User".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            assert!(user.validate().is_ok());
        }

        #[test]
        fn test_user_validation_with_invalid_email() {
            let user = User {
                id: Uuid::new_v4(),
                email: "invalid-email".to_string(),
                name: "Test User".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let validation_result = user.validate();
            assert!(validation_result.is_err());
            
            if let Err(errors) = validation_result {
                assert!(errors.field_errors().contains_key("email"));
            }
        }

        #[test]
        fn test_user_validation_with_empty_email() {
            let user = User {
                id: Uuid::new_v4(),
                email: "".to_string(),
                name: "Test User".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let validation_result = user.validate();
            assert!(validation_result.is_err());
        }

        #[test]
        fn test_user_validation_with_empty_name() {
            let user = User {
                id: Uuid::new_v4(),
                email: "test@example.com".to_string(),
                name: "".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let validation_result = user.validate();
            assert!(validation_result.is_err());
            
            if let Err(errors) = validation_result {
                assert!(errors.field_errors().contains_key("name"));
            }
        }

        #[test]
        fn test_user_validation_with_name_too_long() {
            let long_name = "a".repeat(256);
            let user = User {
                id: Uuid::new_v4(),
                email: "test@example.com".to_string(),
                name: long_name,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let validation_result = user.validate();
            assert!(validation_result.is_err());
        }

        #[test]
        fn test_user_validation_with_maximum_valid_name_length() {
            let max_name = "a".repeat(255);
            let user = User {
                id: Uuid::new_v4(),
                email: "test@example.com".to_string(),
                name: max_name,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            assert!(user.validate().is_ok());
        }

        #[test]
        fn test_user_serialization() {
            let user = User {
                id: Uuid::new_v4(),
                email: "test@example.com".to_string(),
                name: "Test User".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let json = serde_json::to_string(&user);
            assert!(json.is_ok());
        }

        #[test]
        fn test_user_deserialization() {
            let user_id = Uuid::new_v4();
            let now = Utc::now();
            let json = format!(
                r#"{{"id":"{}","email":"test@example.com","name":"Test User","created_at":"{}","updated_at":"{}"}}"#,
                user_id,
                now.to_rfc3339(),
                now.to_rfc3339()
            );

            let user: Result<User, _> = serde_json::from_str(&json);
            assert!(user.is_ok());
            
            let user = user.unwrap();
            assert_eq!(user.id, user_id);
            assert_eq!(user.email, "test@example.com");
            assert_eq!(user.name, "Test User");
        }

        #[test]
        fn test_user_clone() {
            let user = User {
                id: Uuid::new_v4(),
                email: "test@example.com".to_string(),
                name: "Test User".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let cloned = user.clone();
            assert_eq!(user.id, cloned.id);
            assert_eq!(user.email, cloned.email);
            assert_eq!(user.name, cloned.name);
        }
    }

    // ============================================================================
    // Task Tests
    // ============================================================================

    mod task_tests {
        use super::*;

        fn create_valid_task() -> Task {
            Task {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                title: "Study Rust".to_string(),
                description: Some("Learn about ownership and borrowing".to_string()),
                estimated_duration_minutes: Some(120),
                priority: TaskPriority::Medium,
                status: TaskStatus::Pending,
                due_date: Some(Utc::now() + Duration::days(7)),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        }

        #[test]
        fn test_task_creation_with_valid_data() {
            let task = create_valid_task();
            assert!(task.validate().is_ok());
        }

        #[test]
        fn test_task_validation_with_empty_title() {
            let mut task = create_valid_task();
            task.title = "".to_string();

            let validation_result = task.validate();
            assert!(validation_result.is_err());
            
            if let Err(errors) = validation_result {
                assert!(errors.field_errors().contains_key("title"));
            }
        }

        #[test]
        fn test_task_validation_with_title_too_long() {
            let mut task = create_valid_task();
            task.title = "a".repeat(256);

            let validation_result = task.validate();
            assert!(validation_result.is_err());
        }

        #[test]
        fn test_task_validation_with_maximum_valid_title_length() {
            let mut task = create_valid_task();
            task.title = "a".repeat(255);

            assert!(task.validate().is_ok());
        }

        #[test]
        fn test_task_validation_with_description_too_long() {
            let mut task = create_valid_task();
            task.description = Some("a".repeat(2001));

            let validation_result = task.validate();
            assert!(validation_result.is_err());
        }

        #[test]
        fn test_task_validation_with_maximum_valid_description_length() {
            let mut task = create_valid_task();
            task.description = Some("a".repeat(2000));

            assert!(task.validate().is_ok());
        }

        #[test]
        fn test_task_validation_with_none_description() {
            let mut task = create_valid_task();
            task.description = None;

            assert!(task.validate().is_ok());
        }

        #[test]
        fn test_task_validation_with_negative_duration() {
            let mut task = create_valid_task();
            task.estimated_duration_minutes = Some(-10);

            let validation_result = task.validate();
            assert!(validation_result.is_err());
        }

        #[test]
        fn test_task_validation_with_zero_duration() {
            let mut task = create_valid_task();
            task.estimated_duration_minutes = Some(0);

            let validation_result = task.validate();
            assert!(validation_result.is_err());
        }

        #[test]
        fn test_task_validation_with_excessive_duration() {
            let mut task = create_valid_task();
            task.estimated_duration_minutes = Some(1441); // More than 24 hours

            let validation_result = task.validate();
            assert!(validation_result.is_err());
        }

        #[test]
        fn test_task_validation_with_maximum_valid_duration() {
            let mut task = create_valid_task();
            task.estimated_duration_minutes = Some(1440); // Exactly 24 hours

            assert!(task.validate().is_ok());
        }

        #[test]
        fn test_task_validation_with_none_duration() {
            let mut task = create_valid_task();
            task.estimated_duration_minutes = None;

            assert!(task.validate().is_ok());
        }

        #[test]
        fn test_task_priority_variants() {
            let priorities = vec![
                TaskPriority::Low,
                TaskPriority::Medium,
                TaskPriority::High,
                TaskPriority::Urgent,
            ];

            for priority in priorities {
                let mut task = create_valid_task();
                task.priority = priority;
                assert!(task.validate().is_ok());
            }
        }

        #[test]
        fn test_task_status_variants() {
            let statuses = vec![
                TaskStatus::Pending,
                TaskStatus::InProgress,
                TaskStatus::Completed,
                TaskStatus::Cancelled,
            ];

            for status in statuses {
                let mut task = create_valid_task();
                task.status = status;
                assert!(task.validate().is_ok());
            }
        }

        #[test]
        fn test_task_serialization() {
            let task = create_valid_task();
            let json = serde_json::to_string(&task);
            assert!(json.is_ok());
        }

        #[test]
        fn test_task_deserialization() {
            let task = create_valid_task();
            let json = serde_json::to_string(&task).unwrap();
            let deserialized: Result<Task, _> = serde_json::from_str(&json);
            
            assert!(deserialized.is_ok());
            let deserialized = deserialized.unwrap();
            assert_eq!(task.id, deserialized.id);
            assert_eq!(task.title, deserialized.title);
        }

        #[test]
        fn test_task_priority_ordering() {
            assert!(TaskPriority::Low < TaskPriority::Medium);
            assert!(TaskPriority::Medium < TaskPriority::High);
            assert!(TaskPriority::High < TaskPriority::Urgent);
        }
    }

    // ============================================================================
    // StudySession Tests
    // ============================================================================

    mod study_session_tests {
        use super::*;

        fn create_valid_study_session() -> StudySession {
            let start = Utc::now();
            StudySession {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                task_id: Some(Uuid::new_v4()),
                start_time: start,
                end_time: Some(start + Duration::hours(2)),
                notes: Some("Productive session".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        }

        #[test]
        fn test_study_session_creation_with_valid_data() {
            let session = create_valid_study_session();
            assert!(session.validate().is_ok());
        }

        #[test]
        fn test_study_session_validation_with_end_before_start() {
            let start = Utc::now();
            let session = StudySession {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                task_id: Some(Uuid::new_v4()),
                start_time: start,
                end_time: Some(start - Duration::hours(1)),
                notes: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let validation_result = session.validate();
            assert!(validation_result.is_err());
        }

        #[test]
        fn test_study_session_validation_with_same_start_and_end() {
            let time = Utc::now();
            let session = StudySession {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                task_id: Some(Uuid::new_v4()),
                start_time: time,
                end_time: Some(time),
                notes: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let validation_result = session.validate();
            assert!(validation_result.is_err());
        }

        #[test]
        fn test_study_session_validation_with_none_end_time() {
            let mut session = create_valid_study_session();
            session.end_time = None;

            // Ongoing session should be valid
            assert!(session.validate().is_ok());
        }

        #[test]
        fn test_study_session_validation_with_none_task_id() {
            let mut session = create_valid_study_session();
            session.task_id = None;

            // Session without task should be valid (general study time)
            assert!(session.validate().is_ok());
        }

        #[test]
        fn test_study_session_validation_with_notes_too_long() {
            let mut session = create_valid_study_session();
            session.notes = Some("a".repeat(5001));

            let validation_result = session.validate();
            assert!(validation_result.is_err());
        }

        #[test]
        fn test_study_session_validation_with_maximum_valid_notes_length() {
            let mut session = create_valid_study_session();
            session.notes = Some("a".repeat(5000));

            assert!(session.validate().is_ok());
        }

        #[test]
        fn test_study_session_validation_with_empty_notes() {
            let mut session = create_valid_study_session();
            session.notes = Some("".to_string());

            // Empty notes should be valid
            assert!(session.validate().is_ok());
        }

        #[test]
        fn test_study_session_duration_calculation() {
            let start = Utc::now();
            let end = start + Duration::hours(2);
            let session = StudySession {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                task_id: None,
                start_time: start,
                end_time: Some(end),
                notes: None,
                created_at: Utc::now(),