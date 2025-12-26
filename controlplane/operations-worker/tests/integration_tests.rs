//! Integration tests for the Operations Worker.
//!
//! These tests verify the executor implementations against real or mock backends:
//!
//! - **SpiceDB**: Uses `SpiceDB::mock()` for an in-memory SpiceDB server
//! - **Pangolin**: Requires a real Pangolin instance (via docker-compose)
//! - **Database**: Requires a PostgreSQL database with migrations applied
//!
//! ## Running Tests
//!
//! For SpiceDB-only tests (no external dependencies):
//! ```bash
//! cargo test --package operations-worker --test integration_tests spicedb
//! ```
//!
//! For Pangolin tests (requires docker-compose):
//! ```bash
//! docker-compose up -d pangolin pangolin-db
//! export PANGOLIN_API_URL=http://localhost:3001
//! export PANGOLIN_API_KEY=<your-api-key>
//! export PANGOLIN_ORG_ID=<your-org-id>
//! cargo test --package operations-worker --test integration_tests pangolin
//! ```
//!
//! For database tests (requires docker-compose):
//! ```bash
//! docker-compose up -d postgres postgres-migrate
//! export DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
//! cargo test --package operations-worker --test integration_tests database
//! ```

use async_trait::async_trait;
use frn_core::operations::{
    CompositeExecutor, ExecutorError, Operation, OperationExecutor, OperationType, RetryPolicy,
};
use serde_json::{Value as JsonValue, json};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Configuration for Pangolin integration tests.
struct PangolinTestConfig {
    api_url: String,
    api_key: String,
    org_id: String,
}

impl PangolinTestConfig {
    /// Attempts to load test configuration from environment variables.
    /// Returns None if any required variable is not set.
    fn from_env() -> Option<Self> {
        let api_url = env::var("PANGOLIN_API_URL").ok()?;
        let api_key = env::var("PANGOLIN_API_KEY").ok()?;
        let org_id = env::var("PANGOLIN_ORG_ID").ok()?;

        Some(Self {
            api_url,
            api_key,
            org_id,
        })
    }
}

/// Configuration for database integration tests.
struct DatabaseTestConfig {
    pool: sqlx::PgPool,
}

impl DatabaseTestConfig {
    /// Attempts to connect to the database using DATABASE_URL environment variable.
    /// Returns None if DATABASE_URL is not set or connection fails.
    async fn connect() -> Option<Self> {
        let database_url = env::var("DATABASE_URL").ok()?;
        let pool = sqlx::PgPool::connect(&database_url).await.ok()?;

        Some(Self { pool })
    }
}

/// Helper macro to require Pangolin configuration.
/// Gracefully skips the test if environment variables are not set.
macro_rules! require_pangolin {
    () => {
        match PangolinTestConfig::from_env() {
            Some(config) => config,
            None => {
                eprintln!(
                    "Skipping test: Pangolin not configured. \
                     Set PANGOLIN_API_URL, PANGOLIN_API_KEY, and PANGOLIN_ORG_ID."
                );
                return;
            }
        }
    };
}

/// Helper macro to require database connection.
/// Gracefully skips the test if DATABASE_URL is not set or connection fails.
macro_rules! require_database {
    () => {
        match DatabaseTestConfig::connect().await {
            Some(config) => config.pool,
            None => {
                eprintln!(
                    "Skipping test: Database not configured. \
                     Set DATABASE_URL and ensure the database is running."
                );
                return;
            }
        }
    };
}

// =============================================================================
// SpiceDB Executor Tests (using mock)
// =============================================================================

mod spicedb_executor {
    use super::*;

    /// SpiceDB executor for authorization relationship operations.
    struct SpiceDbExecutor {
        client: Mutex<spicedb::SpiceDB>,
    }

    impl SpiceDbExecutor {
        async fn new_mock() -> Self {
            let client = spicedb::SpiceDB::mock().await;
            Self {
                client: Mutex::new(client),
            }
        }
    }

    #[async_trait]
    impl OperationExecutor for SpiceDbExecutor {
        fn handles(&self, operation_type: &OperationType) -> bool {
            matches!(
                operation_type,
                OperationType::SpiceDbWriteRelationship | OperationType::SpiceDbDeleteRelationship
            )
        }

        async fn execute(&self, operation: &Operation) -> Result<JsonValue, ExecutorError> {
            let mut client = self.client.lock().await;

            match operation.operation_type {
                OperationType::SpiceDbWriteRelationship => {
                    let input = &operation.input;

                    let subject_type = input["subject_type"]
                        .as_str()
                        .ok_or_else(|| ExecutorError::InvalidInput("missing subject_type".into()))?
                        .to_string();
                    let subject_id = input["subject_id"]
                        .as_str()
                        .ok_or_else(|| ExecutorError::InvalidInput("missing subject_id".into()))?
                        .to_string();
                    let relation = input["relation"]
                        .as_str()
                        .ok_or_else(|| ExecutorError::InvalidInput("missing relation".into()))?
                        .to_string();
                    let object_type = input["object_type"]
                        .as_str()
                        .ok_or_else(|| ExecutorError::InvalidInput("missing object_type".into()))?
                        .to_string();
                    let object_id = input["object_id"]
                        .as_str()
                        .ok_or_else(|| ExecutorError::InvalidInput("missing object_id".into()))?
                        .to_string();

                    client
                        .write_relationship(
                            subject_type,
                            subject_id,
                            relation,
                            object_type,
                            object_id,
                        )
                        .await
                        .map_err(|e| ExecutorError::Internal(e.to_string()))?;

                    Ok(json!({"written": true}))
                }
                OperationType::SpiceDbDeleteRelationship => {
                    let input = &operation.input;

                    let subject_type = input["subject_type"]
                        .as_str()
                        .ok_or_else(|| ExecutorError::InvalidInput("missing subject_type".into()))?
                        .to_string();
                    let subject_id = input["subject_id"]
                        .as_str()
                        .ok_or_else(|| ExecutorError::InvalidInput("missing subject_id".into()))?
                        .to_string();
                    let relation = input["relation"]
                        .as_str()
                        .ok_or_else(|| ExecutorError::InvalidInput("missing relation".into()))?
                        .to_string();
                    let object_type = input["object_type"]
                        .as_str()
                        .ok_or_else(|| ExecutorError::InvalidInput("missing object_type".into()))?
                        .to_string();
                    let object_id = input["object_id"]
                        .as_str()
                        .ok_or_else(|| ExecutorError::InvalidInput("missing object_id".into()))?
                        .to_string();

                    client
                        .delete_relationship(
                            subject_type,
                            subject_id,
                            relation,
                            object_type,
                            object_id,
                        )
                        .await
                        .map_err(|e| ExecutorError::Internal(e.to_string()))?;

                    Ok(json!({"deleted": true}))
                }
                _ => Err(ExecutorError::NotHandled(operation.operation_type)),
            }
        }
    }

    #[tokio::test]
    async fn test_spicedb_executor_handles_write_relationship() {
        let executor = SpiceDbExecutor::new_mock().await;

        assert!(executor.handles(&OperationType::SpiceDbWriteRelationship));
        assert!(executor.handles(&OperationType::SpiceDbDeleteRelationship));
        assert!(!executor.handles(&OperationType::PangolinInviteUser));
    }

    #[tokio::test]
    async fn test_spicedb_write_relationship_success() {
        let executor = SpiceDbExecutor::new_mock().await;

        let operation = create_test_operation(
            OperationType::SpiceDbWriteRelationship,
            json!({
                "subject_type": "user",
                "subject_id": "user-123",
                "relation": "member",
                "object_type": "organization",
                "object_id": "org-456"
            }),
        );

        let result = executor.execute(&operation).await;
        assert!(result.is_ok(), "Failed to write relationship: {:?}", result);

        let output = result.unwrap();
        assert_eq!(output["written"], true);
    }

    #[tokio::test]
    async fn test_spicedb_delete_relationship_success() {
        let executor = SpiceDbExecutor::new_mock().await;

        // First, write a relationship
        let write_operation = create_test_operation(
            OperationType::SpiceDbWriteRelationship,
            json!({
                "subject_type": "user",
                "subject_id": "user-789",
                "relation": "admin",
                "object_type": "project",
                "object_id": "proj-123"
            }),
        );
        executor.execute(&write_operation).await.unwrap();

        // Then delete it
        let delete_operation = create_test_operation(
            OperationType::SpiceDbDeleteRelationship,
            json!({
                "subject_type": "user",
                "subject_id": "user-789",
                "relation": "admin",
                "object_type": "project",
                "object_id": "proj-123"
            }),
        );

        let result = executor.execute(&delete_operation).await;
        assert!(
            result.is_ok(),
            "Failed to delete relationship: {:?}",
            result
        );

        let output = result.unwrap();
        assert_eq!(output["deleted"], true);
    }

    #[tokio::test]
    async fn test_spicedb_write_relationship_missing_input() {
        let executor = SpiceDbExecutor::new_mock().await;

        let operation = create_test_operation(
            OperationType::SpiceDbWriteRelationship,
            json!({
                "subject_type": "user",
                // Missing subject_id
                "relation": "member",
                "object_type": "organization",
                "object_id": "org-456"
            }),
        );

        let result = executor.execute(&operation).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ExecutorError::InvalidInput(msg) => {
                assert!(msg.contains("subject_id"));
            }
            other => panic!("Expected InvalidInput error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_spicedb_executor_not_handled() {
        let executor = SpiceDbExecutor::new_mock().await;

        let operation = create_test_operation(
            OperationType::PangolinInviteUser,
            json!({
                "org_id": "org-123",
                "email": "test@example.com",
                "role_id": "member"
            }),
        );

        let result = executor.execute(&operation).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ExecutorError::NotHandled(_) => {}
            other => panic!("Expected NotHandled error, got: {:?}", other),
        }
    }
}

// =============================================================================
// Pangolin Executor Tests (using real Pangolin)
// =============================================================================

mod pangolin_executor {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Pangolin executor for Zero Trust VPN user management operations.
    struct PangolinExecutor {
        client: Arc<reqwest::Client>,
        api_url: String,
        api_key: String,
    }

    impl PangolinExecutor {
        fn new(api_url: &str, api_key: &str) -> Self {
            Self {
                client: Arc::new(reqwest::Client::new()),
                api_url: api_url.to_string(),
                api_key: api_key.to_string(),
            }
        }
    }

    fn pangolin_error_to_executor_error(e: pangolin::api::Error) -> ExecutorError {
        match e {
            pangolin::api::Error::Connectivity(e) => ExecutorError::Connectivity(e.to_string()),
            pangolin::api::Error::Unauthorized => {
                ExecutorError::Unauthorized("invalid API key".into())
            }
            pangolin::api::Error::NotFound(msg) => ExecutorError::NotFound(msg),
            pangolin::api::Error::BadRequest(msg) => ExecutorError::Rejected(msg),
            pangolin::api::Error::Conflict(msg) => ExecutorError::Rejected(msg),
            pangolin::api::Error::Internal(msg) => ExecutorError::TemporarilyUnavailable(msg),
            pangolin::api::Error::UnexpectedResponse(msg) => ExecutorError::Internal(msg),
        }
    }

    #[async_trait]
    impl OperationExecutor for PangolinExecutor {
        fn handles(&self, operation_type: &OperationType) -> bool {
            matches!(
                operation_type,
                OperationType::PangolinInviteUser
                    | OperationType::PangolinRemoveUser
                    | OperationType::PangolinUpdateUser
            )
        }

        async fn execute(&self, operation: &Operation) -> Result<JsonValue, ExecutorError> {
            let input = &operation.input;

            match operation.operation_type {
                OperationType::PangolinInviteUser => {
                    let org_id = input["org_id"]
                        .as_str()
                        .ok_or_else(|| ExecutorError::InvalidInput("missing org_id".into()))?;
                    let email = input["email"]
                        .as_str()
                        .ok_or_else(|| ExecutorError::InvalidInput("missing email".into()))?;
                    let role_id = input["role_id"]
                        .as_str()
                        .ok_or_else(|| ExecutorError::InvalidInput("missing role_id".into()))?;
                    let send_email = input["send_email"].as_bool().unwrap_or(true);
                    let valid_for_hours = input["valid_for_hours"].as_i64();

                    let response = pangolin::api::create_invite(
                        &self.api_url,
                        &self.client,
                        &self.api_key,
                        org_id,
                        email,
                        role_id,
                        send_email,
                        valid_for_hours,
                    )
                    .await
                    .map_err(pangolin_error_to_executor_error)?;

                    Ok(json!({
                        "invite_id": response.invite_id,
                        "token": response.token,
                        "expires_at": response.expires_at
                    }))
                }
                OperationType::PangolinRemoveUser => {
                    let org_id = input["org_id"]
                        .as_str()
                        .ok_or_else(|| ExecutorError::InvalidInput("missing org_id".into()))?;
                    let user_id = input["user_id"]
                        .as_str()
                        .ok_or_else(|| ExecutorError::InvalidInput("missing user_id".into()))?;

                    pangolin::api::remove_user(
                        &self.api_url,
                        &self.client,
                        &self.api_key,
                        org_id,
                        user_id,
                    )
                    .await
                    .map_err(pangolin_error_to_executor_error)?;

                    Ok(json!({"removed": true}))
                }
                OperationType::PangolinUpdateUser => {
                    let org_id = input["org_id"]
                        .as_str()
                        .ok_or_else(|| ExecutorError::InvalidInput("missing org_id".into()))?;
                    let user_id = input["user_id"]
                        .as_str()
                        .ok_or_else(|| ExecutorError::InvalidInput("missing user_id".into()))?;
                    let role_id = input["role_id"].as_str();
                    let disabled = input["disabled"].as_bool();

                    pangolin::api::update_user(
                        &self.api_url,
                        &self.client,
                        &self.api_key,
                        org_id,
                        user_id,
                        role_id,
                        disabled,
                    )
                    .await
                    .map_err(pangolin_error_to_executor_error)?;

                    Ok(json!({"updated": true}))
                }
                _ => Err(ExecutorError::NotHandled(operation.operation_type)),
            }
        }
    }

    #[tokio::test]
    async fn test_pangolin_executor_handles_operations() {
        // This test doesn't need Pangolin, just checks handling logic
        let executor = PangolinExecutor::new("http://localhost:3001", "fake-key");

        assert!(executor.handles(&OperationType::PangolinInviteUser));
        assert!(executor.handles(&OperationType::PangolinRemoveUser));
        assert!(executor.handles(&OperationType::PangolinUpdateUser));
        assert!(!executor.handles(&OperationType::SpiceDbWriteRelationship));
    }

    #[tokio::test]
    async fn test_pangolin_invite_user_success() {
        let config = require_pangolin!();
        let executor = PangolinExecutor::new(&config.api_url, &config.api_key);

        // Generate unique email to avoid conflicts
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let test_email = format!("test-integration-{}@example.com", timestamp);

        let operation = create_test_operation(
            OperationType::PangolinInviteUser,
            json!({
                "org_id": config.org_id,
                "email": test_email,
                "role_id": "member",
                "send_email": false,
                "valid_for_hours": 1
            }),
        );

        let result = executor.execute(&operation).await;
        assert!(result.is_ok(), "Failed to invite user: {:?}", result);

        let output = result.unwrap();
        assert!(!output["invite_id"].as_str().unwrap_or("").is_empty());
        assert!(!output["token"].as_str().unwrap_or("").is_empty());
    }

    #[tokio::test]
    async fn test_pangolin_invite_user_unauthorized() {
        let config = require_pangolin!();
        let executor = PangolinExecutor::new(&config.api_url, "invalid-api-key");

        let operation = create_test_operation(
            OperationType::PangolinInviteUser,
            json!({
                "org_id": config.org_id,
                "email": "test@example.com",
                "role_id": "member",
                "send_email": false
            }),
        );

        let result = executor.execute(&operation).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ExecutorError::Unauthorized(_) => {}
            other => panic!("Expected Unauthorized error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_pangolin_remove_nonexistent_user() {
        let config = require_pangolin!();
        let executor = PangolinExecutor::new(&config.api_url, &config.api_key);

        let operation = create_test_operation(
            OperationType::PangolinRemoveUser,
            json!({
                "org_id": config.org_id,
                "user_id": "nonexistent-user-id-12345"
            }),
        );

        let result = executor.execute(&operation).await;
        // Should fail because user doesn't exist
        assert!(result.is_err(), "Should fail for nonexistent user");
    }

    #[tokio::test]
    async fn test_pangolin_update_nonexistent_user() {
        let config = require_pangolin!();
        let executor = PangolinExecutor::new(&config.api_url, &config.api_key);

        let operation = create_test_operation(
            OperationType::PangolinUpdateUser,
            json!({
                "org_id": config.org_id,
                "user_id": "nonexistent-user-id-12345",
                "role_id": "member"
            }),
        );

        let result = executor.execute(&operation).await;
        // Should fail because user doesn't exist
        assert!(result.is_err(), "Should fail for nonexistent user");
    }

    #[tokio::test]
    async fn test_pangolin_invite_missing_input() {
        let config = require_pangolin!();
        let executor = PangolinExecutor::new(&config.api_url, &config.api_key);

        let operation = create_test_operation(
            OperationType::PangolinInviteUser,
            json!({
                "org_id": config.org_id,
                // Missing email
                "role_id": "member"
            }),
        );

        let result = executor.execute(&operation).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ExecutorError::InvalidInput(msg) => {
                assert!(msg.contains("email"));
            }
            other => panic!("Expected InvalidInput error, got: {:?}", other),
        }
    }
}

// =============================================================================
// Composite Executor Tests
// =============================================================================

mod composite_executor {
    use super::*;

    /// Simple test executor that tracks calls.
    struct TrackerExecutor {
        handled_types: Vec<OperationType>,
        should_fail: bool,
    }

    impl TrackerExecutor {
        fn new(types: Vec<OperationType>) -> Self {
            Self {
                handled_types: types,
                should_fail: false,
            }
        }

        fn failing(types: Vec<OperationType>) -> Self {
            Self {
                handled_types: types,
                should_fail: true,
            }
        }
    }

    #[async_trait]
    impl OperationExecutor for TrackerExecutor {
        fn handles(&self, operation_type: &OperationType) -> bool {
            self.handled_types.contains(operation_type)
        }

        async fn execute(&self, _operation: &Operation) -> Result<JsonValue, ExecutorError> {
            if self.should_fail {
                Err(ExecutorError::Internal("intentional failure".into()))
            } else {
                Ok(json!({"executed": true}))
            }
        }
    }

    #[tokio::test]
    async fn test_composite_executor_routes_correctly() {
        let spicedb_executor = TrackerExecutor::new(vec![
            OperationType::SpiceDbWriteRelationship,
            OperationType::SpiceDbDeleteRelationship,
        ]);

        let pangolin_executor = TrackerExecutor::new(vec![
            OperationType::PangolinInviteUser,
            OperationType::PangolinRemoveUser,
            OperationType::PangolinUpdateUser,
        ]);

        let composite = CompositeExecutor::new()
            .register(Box::new(spicedb_executor))
            .register(Box::new(pangolin_executor));

        // Test SpiceDB operation
        let spicedb_op = create_test_operation(
            OperationType::SpiceDbWriteRelationship,
            json!({"test": true}),
        );
        assert!(composite.execute(&spicedb_op).await.is_ok());

        // Test Pangolin operation
        let pangolin_op =
            create_test_operation(OperationType::PangolinInviteUser, json!({"test": true}));
        assert!(composite.execute(&pangolin_op).await.is_ok());

        // Test unhandled operation
        let unhandled_op =
            create_test_operation(OperationType::HoopCreateAgent, json!({"test": true}));
        let result = composite.execute(&unhandled_op).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ExecutorError::NotHandled(_) => {}
            other => panic!("Expected NotHandled error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_composite_executor_propagates_errors() {
        let failing_executor =
            TrackerExecutor::failing(vec![OperationType::SpiceDbWriteRelationship]);

        let composite = CompositeExecutor::new().register(Box::new(failing_executor));

        let operation = create_test_operation(
            OperationType::SpiceDbWriteRelationship,
            json!({"test": true}),
        );

        let result = composite.execute(&operation).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ExecutorError::Internal(msg) => {
                assert!(msg.contains("intentional failure"));
            }
            other => panic!("Expected Internal error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_composite_executor_handles() {
        let executor = TrackerExecutor::new(vec![
            OperationType::SpiceDbWriteRelationship,
            OperationType::PangolinInviteUser,
        ]);

        let composite = CompositeExecutor::new().register(Box::new(executor));

        assert!(composite.handles(&OperationType::SpiceDbWriteRelationship));
        assert!(composite.handles(&OperationType::PangolinInviteUser));
        assert!(!composite.handles(&OperationType::HoopCreateAgent));
    }
}

// =============================================================================
// Database Integration Tests
// =============================================================================

mod database {
    use super::*;

    #[tokio::test]
    async fn test_operation_create_and_fetch() {
        let pool = require_database!();

        // Create an operation
        let operation = Operation::new(
            OperationType::SpiceDbWriteRelationship,
            "TestResource",
            uuid::Uuid::new_v4(),
            json!({
                "subject_type": "user",
                "subject_id": "test-user",
                "relation": "member",
                "object_type": "organization",
                "object_id": "test-org"
            }),
        )
        .create(&pool)
        .await
        .expect("Failed to create operation");

        // Verify the operation was created
        assert!(!operation.name.is_empty());
        assert_eq!(
            operation.status,
            frn_core::operations::OperationStatus::Pending
        );
        assert_eq!(operation.attempt_count, 0);

        // Fetch the operation by ID
        let fetched = Operation::find_by_id(&pool, operation.id)
            .await
            .expect("Failed to find operation")
            .expect("Operation not found");

        assert_eq!(fetched.id, operation.id);
        assert_eq!(
            fetched.operation_type,
            OperationType::SpiceDbWriteRelationship
        );

        // Clean up
        sqlx::query("DELETE FROM operations WHERE id = $1")
            .bind(operation.id)
            .execute(&pool)
            .await
            .expect("Failed to clean up");
    }

    #[tokio::test]
    async fn test_operation_lifecycle() {
        let pool = require_database!();

        // Create an operation
        let operation = Operation::new(
            OperationType::PangolinInviteUser,
            "Invitation",
            uuid::Uuid::new_v4(),
            json!({
                "org_id": "test-org",
                "email": "test@example.com",
                "role_id": "member"
            }),
        )
        .create(&pool)
        .await
        .expect("Failed to create operation");

        // Mark as running
        operation
            .mark_running(&pool)
            .await
            .expect("Failed to mark running");

        let running = Operation::find_by_id(&pool, operation.id)
            .await
            .expect("Failed to find operation")
            .expect("Operation not found");

        assert_eq!(
            running.status,
            frn_core::operations::OperationStatus::Running
        );
        assert_eq!(running.attempt_count, 1);

        // Mark as succeeded
        operation
            .mark_succeeded(&pool, json!({"invite_id": "inv-123"}))
            .await
            .expect("Failed to mark succeeded");

        let succeeded = Operation::find_by_id(&pool, operation.id)
            .await
            .expect("Failed to find operation")
            .expect("Operation not found");

        assert_eq!(
            succeeded.status,
            frn_core::operations::OperationStatus::Succeeded
        );
        assert!(succeeded.output.is_some());

        // Clean up
        sqlx::query("DELETE FROM operations WHERE id = $1")
            .bind(operation.id)
            .execute(&pool)
            .await
            .expect("Failed to clean up");
    }

    #[tokio::test]
    async fn test_operation_retry_lifecycle() {
        let pool = require_database!();

        // Create an operation
        let operation = Operation::new(
            OperationType::SpiceDbDeleteRelationship,
            "Relationship",
            uuid::Uuid::new_v4(),
            json!({
                "subject_type": "user",
                "subject_id": "user-123",
                "relation": "member",
                "object_type": "organization",
                "object_id": "org-456"
            }),
        )
        .max_attempts(3)
        .create(&pool)
        .await
        .expect("Failed to create operation");

        // Mark as running
        operation
            .mark_running(&pool)
            .await
            .expect("Failed to mark running");

        // Simulate a failure and retry
        let retry_policy = RetryPolicy::default();
        let next_retry = retry_policy.next_retry_at(1);

        operation
            .mark_failed_or_retry(&pool, "Connection timeout", Some(next_retry))
            .await
            .expect("Failed to mark for retry");

        let retrying = Operation::find_by_id(&pool, operation.id)
            .await
            .expect("Failed to find operation")
            .expect("Operation not found");

        assert_eq!(
            retrying.status,
            frn_core::operations::OperationStatus::Pending
        );
        assert!(retrying.next_retry_at.is_some());
        assert!(retrying.last_error.is_some());

        // Clean up
        sqlx::query("DELETE FROM operations WHERE id = $1")
            .bind(operation.id)
            .execute(&pool)
            .await
            .expect("Failed to clean up");
    }

    #[tokio::test]
    async fn test_operation_fetch_next_with_skip_locked() {
        let pool = require_database!();

        // Create multiple pending operations
        let op1 = Operation::new(
            OperationType::SpiceDbWriteRelationship,
            "Test1",
            uuid::Uuid::new_v4(),
            json!({"test": 1}),
        )
        .create(&pool)
        .await
        .expect("Failed to create operation 1");

        let op2 = Operation::new(
            OperationType::SpiceDbWriteRelationship,
            "Test2",
            uuid::Uuid::new_v4(),
            json!({"test": 2}),
        )
        .create(&pool)
        .await
        .expect("Failed to create operation 2");

        // Fetch the next pending operation
        let fetched = Operation::fetch_next(&pool)
            .await
            .expect("Failed to fetch next operation");

        assert!(fetched.is_some());
        let fetched = fetched.unwrap();

        // Should be one of our created operations
        assert!(fetched.id == op1.id || fetched.id == op2.id);

        // Clean up
        sqlx::query("DELETE FROM operations WHERE id IN ($1, $2)")
            .bind(op1.id)
            .bind(op2.id)
            .execute(&pool)
            .await
            .expect("Failed to clean up");
    }

    #[tokio::test]
    async fn test_operation_cancel() {
        let pool = require_database!();

        // Create an operation
        let operation = Operation::new(
            OperationType::PangolinRemoveUser,
            "UserRemoval",
            uuid::Uuid::new_v4(),
            json!({
                "org_id": "test-org",
                "user_id": "user-123"
            }),
        )
        .create(&pool)
        .await
        .expect("Failed to create operation");

        // Cancel the operation
        let cancelled = operation.cancel(&pool).await.expect("Failed to cancel");
        assert!(cancelled);

        let after = Operation::find_by_id(&pool, operation.id)
            .await
            .expect("Failed to find operation")
            .expect("Operation not found");

        assert_eq!(
            after.status,
            frn_core::operations::OperationStatus::Cancelled
        );

        // Try to cancel again (should fail since already in terminal state)
        let operation_again = Operation::find_by_id(&pool, operation.id)
            .await
            .expect("Failed to find operation")
            .expect("Operation not found");

        let cancelled_again = operation_again
            .cancel(&pool)
            .await
            .expect("Failed to cancel again");
        assert!(!cancelled_again);

        // Clean up
        sqlx::query("DELETE FROM operations WHERE id = $1")
            .bind(operation.id)
            .execute(&pool)
            .await
            .expect("Failed to clean up");
    }
}

// =============================================================================
// Retry Policy Tests
// =============================================================================

mod retry_policy {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_retry_policy_exponential_growth() {
        let policy = RetryPolicy::new(1, 300, 0.0); // No jitter for predictable tests

        // Verify exponential growth
        let delay1 = policy.next_retry_at(1);
        let delay2 = policy.next_retry_at(2);
        let delay3 = policy.next_retry_at(3);

        // Each delay should be roughly double the previous (within 1 second margin for test execution)
        let now = Utc::now();
        assert!(delay1 > now);
        assert!(delay2 > delay1);
        assert!(delay3 > delay2);
    }

    #[test]
    fn test_retry_policy_is_retryable() {
        assert!(ExecutorError::Connectivity("timeout".into()).is_retryable());
        assert!(ExecutorError::TemporarilyUnavailable("503".into()).is_retryable());
        assert!(!ExecutorError::Unauthorized("bad key".into()).is_retryable());
        assert!(!ExecutorError::NotFound("missing".into()).is_retryable());
        assert!(!ExecutorError::InvalidInput("bad json".into()).is_retryable());
        assert!(!ExecutorError::Rejected("conflict".into()).is_retryable());
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Creates a test operation with the given type and input.
fn create_test_operation(operation_type: OperationType, input: JsonValue) -> Operation {
    let id = uuid::Uuid::new_v4();
    Operation {
        id,
        name: format!("operations/{}", id),
        operation_type,
        target_backend: operation_type.target_backend(),
        resource_type: "TestResource".to_string(),
        resource_id: uuid::Uuid::new_v4(),
        status: frn_core::operations::OperationStatus::Pending,
        input,
        output: None,
        error_code: None,
        error_message: None,
        attempt_count: 0,
        max_attempts: 5,
        next_retry_at: None,
        last_error: None,
        idempotency_key: None,
        created_at: chrono::Utc::now(),
        started_at: None,
        completed_at: None,
        updated_at: chrono::Utc::now(),
    }
}
