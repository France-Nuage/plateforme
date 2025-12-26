//! Operations Worker
//!
//! A background worker that polls for pending operations and executes them
//! against external systems (SpiceDB, Pangolin, Hoop, Kubernetes).
//!
//! The worker uses `FOR UPDATE SKIP LOCKED` for safe concurrent processing
//! and implements exponential backoff for retries.

use frn_core::operations::{
    CompositeExecutor, ExecutorError, Operation, OperationExecutor, OperationType, RetryPolicy,
};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::Mutex;

/// Default polling interval in milliseconds
const DEFAULT_POLL_INTERVAL_MS: u64 = 1000;

/// SpiceDB executor for authorization relationship operations.
///
/// Handles `SpiceDbWriteRelationship` and `SpiceDbDeleteRelationship` operations.
struct SpiceDbExecutor {
    client: Mutex<spicedb::SpiceDB>,
}

impl SpiceDbExecutor {
    async fn new(url: &str, token: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = spicedb::SpiceDB::connect(url, token).await?;
        Ok(Self {
            client: Mutex::new(client),
        })
    }
}

#[async_trait::async_trait]
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
                    .write_relationship(subject_type, subject_id, relation, object_type, object_id)
                    .await
                    .map_err(|e| ExecutorError::Internal(e.to_string()))?;

                Ok(serde_json::json!({"written": true}))
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
                    .delete_relationship(subject_type, subject_id, relation, object_type, object_id)
                    .await
                    .map_err(|e| ExecutorError::Internal(e.to_string()))?;

                Ok(serde_json::json!({"deleted": true}))
            }
            _ => Err(ExecutorError::NotHandled(operation.operation_type)),
        }
    }
}

/// Pangolin executor for Zero Trust VPN user management operations.
///
/// Handles `PangolinInviteUser`, `PangolinRemoveUser`, and `PangolinUpdateUser` operations.
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

/// Converts a Pangolin API error to an `ExecutorError`.
fn pangolin_error_to_executor_error(e: pangolin::api::Error) -> ExecutorError {
    match e {
        pangolin::api::Error::Connectivity(e) => ExecutorError::Connectivity(e.to_string()),
        pangolin::api::Error::Unauthorized => ExecutorError::Unauthorized("invalid API key".into()),
        pangolin::api::Error::NotFound(msg) => ExecutorError::NotFound(msg),
        pangolin::api::Error::BadRequest(msg) => ExecutorError::Rejected(msg),
        pangolin::api::Error::Conflict(msg) => ExecutorError::Rejected(msg),
        pangolin::api::Error::Internal(msg) => ExecutorError::TemporarilyUnavailable(msg),
        pangolin::api::Error::UnexpectedResponse(msg) => ExecutorError::Internal(msg),
    }
}

#[async_trait::async_trait]
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

                Ok(serde_json::json!({
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

                Ok(serde_json::json!({"removed": true}))
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

                Ok(serde_json::json!({"updated": true}))
            }
            _ => Err(ExecutorError::NotHandled(operation.operation_type)),
        }
    }
}

/// Process a single operation with the given executor.
async fn process_operation<E: OperationExecutor>(
    pool: &PgPool,
    operation: Operation,
    executor: &E,
    retry_policy: &RetryPolicy,
) {
    tracing::info!(
        operation_id = %operation.id,
        operation_type = %operation.operation_type,
        attempt = operation.attempt_count + 1,
        "processing operation"
    );

    // Mark as running
    if let Err(e) = operation.mark_running(pool).await {
        tracing::error!(
            operation_id = %operation.id,
            error = %e,
            "failed to mark operation as running"
        );
        return;
    }

    // Execute the operation
    match executor.execute(&operation).await {
        Ok(output) => {
            if let Err(e) = operation.mark_succeeded(pool, output).await {
                tracing::error!(
                    operation_id = %operation.id,
                    error = %e,
                    "failed to mark operation as succeeded"
                );
            } else {
                tracing::info!(
                    operation_id = %operation.id,
                    "operation succeeded"
                );
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            let should_retry = e.is_retryable() && operation.attempt_count < operation.max_attempts;

            if should_retry {
                let next_retry = retry_policy.next_retry_at(operation.attempt_count + 1);
                tracing::warn!(
                    operation_id = %operation.id,
                    error = %error_msg,
                    next_retry = %next_retry,
                    "operation failed, scheduling retry"
                );

                if let Err(e) = operation
                    .mark_failed_or_retry(pool, &error_msg, Some(next_retry))
                    .await
                {
                    tracing::error!(
                        operation_id = %operation.id,
                        error = %e,
                        "failed to schedule retry"
                    );
                }
            } else {
                tracing::error!(
                    operation_id = %operation.id,
                    error = %error_msg,
                    "operation failed permanently"
                );

                if let Err(e) = operation.mark_failed_or_retry(pool, &error_msg, None).await {
                    tracing::error!(
                        operation_id = %operation.id,
                        error = %e,
                        "failed to mark operation as failed"
                    );
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt().init();

    tracing::info!("starting operations-worker...");

    // Retrieve environment variables
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let spicedb_url = env::var("SPICEDB_URL").expect("SPICEDB_URL must be set");
    let spicedb_token =
        env::var("SPICEDB_GRPC_PRESHARED_KEY").expect("SPICEDB_GRPC_PRESHARED_KEY must be set");
    let pangolin_url = env::var("PANGOLIN_API_URL").expect("PANGOLIN_API_URL must be set");
    let pangolin_key = env::var("PANGOLIN_API_KEY").expect("PANGOLIN_API_KEY must be set");

    let poll_interval = Duration::from_millis(
        env::var("OPERATIONS_POLL_INTERVAL_MS")
            .unwrap_or_else(|_| DEFAULT_POLL_INTERVAL_MS.to_string())
            .parse()
            .expect("OPERATIONS_POLL_INTERVAL_MS must be a valid number"),
    );

    // Initialize database connection
    let pool = PgPool::connect(&database_url).await?;
    tracing::info!("connected to database");

    // Initialize SpiceDB executor
    let spicedb_executor = SpiceDbExecutor::new(&spicedb_url, &spicedb_token).await?;
    tracing::info!("connected to SpiceDB");

    // Initialize Pangolin executor
    let pangolin_executor = PangolinExecutor::new(&pangolin_url, &pangolin_key);
    tracing::info!("initialized Pangolin executor");

    // Build composite executor with all backends
    // TODO: Add HoopExecutor, K8sExecutor as they are implemented
    let executor = CompositeExecutor::new()
        .register(Box::new(spicedb_executor))
        .register(Box::new(pangolin_executor));

    // Initialize retry policy
    let retry_policy = RetryPolicy::default();

    tracing::info!(
        poll_interval_ms = poll_interval.as_millis(),
        "starting polling loop"
    );

    // Set up graceful shutdown signal handling
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl+c");
        tracing::info!("received shutdown signal, finishing current operations...");
        shutdown_clone.store(true, Ordering::SeqCst);
    });

    // Main polling loop
    while !shutdown.load(Ordering::SeqCst) {
        // Process all pending operations
        while let Some(operation) = Operation::fetch_next(&pool).await? {
            if shutdown.load(Ordering::SeqCst) {
                tracing::info!("shutdown requested, skipping remaining operations");
                break;
            }
            process_operation(&pool, operation, &executor, &retry_policy).await;
        }

        // Wait before next poll, but allow interruption for shutdown
        tokio::select! {
            _ = tokio::time::sleep(poll_interval) => {}
            _ = async {
                while !shutdown.load(Ordering::SeqCst) {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            } => {}
        }
    }

    tracing::info!("operations-worker shutdown complete");
    Ok(())
}
