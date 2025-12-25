//! Operation entity and database operations
//!
//! The Operation entity represents an async operation to be executed against an external system.
//! Operations follow a state machine pattern and are processed by workers using optimistic
//! locking with `FOR UPDATE SKIP LOCKED`.

use super::{OperationStatus, OperationType, TargetBackend};
use chrono::{DateTime, Utc};
use fabrique::{Factory, Persistable};
use serde_json::Value as JsonValue;
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

/// Maximum number of retry attempts before an operation is marked as failed.
pub const DEFAULT_MAX_ATTEMPTS: i32 = 5;

/// Represents an asynchronous operation to be executed against an external system.
///
/// Operations are created by services when they need to synchronize state with external
/// backends. The operations-worker polls for pending operations and executes them using
/// the appropriate executor.
///
/// ## Lifecycle
/// 1. Service creates Operation with status=PENDING
/// 2. Worker picks up operation (FOR UPDATE SKIP LOCKED), sets status=RUNNING
/// 3. Worker executes operation via executor
/// 4. On success: status=SUCCEEDED, output populated
/// 5. On failure: if retries remain, status=PENDING with next_retry_at; else status=FAILED
#[derive(Debug, Factory, Persistable)]
pub struct Operation {
    /// Unique identifier for the operation.
    #[fabrique(primary_key)]
    pub id: Uuid,

    /// Human-readable name in format "operations/{uuid}".
    pub name: String,

    /// The type of operation to perform.
    #[fabrique(as = "String")]
    pub operation_type: OperationType,

    /// The external backend this operation targets.
    #[fabrique(as = "String")]
    pub target_backend: TargetBackend,

    /// The type of resource this operation affects (e.g., "User", "Organization").
    pub resource_type: String,

    /// The ID of the resource this operation affects.
    pub resource_id: Uuid,

    /// Current status of the operation.
    #[fabrique(as = "String")]
    pub status: OperationStatus,

    /// Input payload for the operation (JSON).
    pub input: JsonValue,

    /// Output from successful execution (JSON).
    pub output: Option<JsonValue>,

    /// Error code on failure.
    pub error_code: Option<String>,

    /// Error message on failure.
    pub error_message: Option<String>,

    /// Number of execution attempts.
    pub attempt_count: i32,

    /// Maximum allowed attempts before permanent failure.
    pub max_attempts: i32,

    /// Next retry time (for exponential backoff).
    pub next_retry_at: Option<DateTime<Utc>>,

    /// Last error encountered (for debugging).
    pub last_error: Option<String>,

    /// Idempotency key to prevent duplicate operations.
    pub idempotency_key: Option<String>,

    /// When the operation was created.
    pub created_at: DateTime<Utc>,

    /// When execution started.
    pub started_at: Option<DateTime<Utc>>,

    /// When execution completed (success or failure).
    pub completed_at: Option<DateTime<Utc>>,

    /// Last update time.
    pub updated_at: DateTime<Utc>,
}

/// Helper to construct Operation from a Row
impl Operation {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            operation_type: row.try_get::<String, _>("operation_type")?.into(),
            target_backend: row.try_get::<String, _>("target_backend")?.into(),
            resource_type: row.try_get("resource_type")?,
            resource_id: row.try_get("resource_id")?,
            status: row.try_get::<String, _>("status")?.into(),
            input: row.try_get("input")?,
            output: row.try_get("output")?,
            error_code: row.try_get("error_code")?,
            error_message: row.try_get("error_message")?,
            attempt_count: row.try_get("attempt_count")?,
            max_attempts: row.try_get("max_attempts")?,
            next_retry_at: row.try_get("next_retry_at")?,
            last_error: row.try_get("last_error")?,
            idempotency_key: row.try_get("idempotency_key")?,
            created_at: row.try_get("created_at")?,
            started_at: row.try_get("started_at")?,
            completed_at: row.try_get("completed_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl Operation {
    /// Creates a new operation builder with required fields.
    pub fn new(
        operation_type: OperationType,
        resource_type: impl Into<String>,
        resource_id: Uuid,
        input: JsonValue,
    ) -> OperationBuilder {
        let id = Uuid::new_v4();
        OperationBuilder {
            id,
            name: format!("operations/{}", id),
            operation_type,
            target_backend: operation_type.target_backend(),
            resource_type: resource_type.into(),
            resource_id,
            status: OperationStatus::Pending,
            input,
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            idempotency_key: None,
        }
    }

    /// Fetches the next pending operation using `FOR UPDATE SKIP LOCKED` for concurrent safety.
    ///
    /// This query picks up:
    /// - Operations in PENDING status with no next_retry_at or past retry time
    /// - Operations stuck in RUNNING for more than 5 minutes (considered stale)
    pub async fn fetch_next(pool: &Pool<Postgres>) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                id, name, operation_type, target_backend, resource_type, resource_id,
                status, input, output, error_code, error_message, attempt_count,
                max_attempts, next_retry_at, last_error, idempotency_key,
                created_at, started_at, completed_at, updated_at
            FROM operations
            WHERE (status = 'PENDING' AND (next_retry_at IS NULL OR next_retry_at <= now()))
               OR (status = 'RUNNING' AND started_at < now() - INTERVAL '5 minutes')
            ORDER BY created_at
            LIMIT 1
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .fetch_optional(pool)
        .await?;

        row.map(|r| Self::from_row(&r)).transpose()
    }

    /// Marks the operation as running and increments the attempt count.
    pub async fn mark_running(&self, pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE operations
            SET status = 'RUNNING',
                started_at = now(),
                attempt_count = attempt_count + 1,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(self.id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Marks the operation as succeeded with the given output.
    pub async fn mark_succeeded(
        &self,
        pool: &Pool<Postgres>,
        output: JsonValue,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE operations
            SET status = 'SUCCEEDED',
                output = $2,
                completed_at = now(),
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(self.id)
        .bind(output)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Marks the operation for retry with exponential backoff, or as failed if retries exhausted.
    pub async fn mark_failed_or_retry(
        &self,
        pool: &Pool<Postgres>,
        error: &str,
        next_retry_at: Option<DateTime<Utc>>,
    ) -> Result<(), sqlx::Error> {
        if self.attempt_count < self.max_attempts {
            // Schedule retry
            sqlx::query(
                r#"
                UPDATE operations
                SET status = 'PENDING',
                    last_error = $2,
                    next_retry_at = $3,
                    updated_at = now()
                WHERE id = $1
                "#,
            )
            .bind(self.id)
            .bind(error)
            .bind(next_retry_at)
            .execute(pool)
            .await?;
        } else {
            // Max retries exhausted
            sqlx::query(
                r#"
                UPDATE operations
                SET status = 'FAILED',
                    error_code = 'EXHAUSTED_RETRIES',
                    error_message = $2,
                    completed_at = now(),
                    updated_at = now()
                WHERE id = $1
                "#,
            )
            .bind(self.id)
            .bind(error)
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    /// Cancels the operation if it's not already in a terminal state.
    pub async fn cancel(&self, pool: &Pool<Postgres>) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE operations
            SET status = 'CANCELLED',
                completed_at = now(),
                updated_at = now()
            WHERE id = $1 AND status NOT IN ('SUCCEEDED', 'FAILED', 'CANCELLED')
            "#,
        )
        .bind(self.id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Finds an operation by its ID.
    pub async fn find_by_id(pool: &Pool<Postgres>, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                id, name, operation_type, target_backend, resource_type, resource_id,
                status, input, output, error_code, error_message, attempt_count,
                max_attempts, next_retry_at, last_error, idempotency_key,
                created_at, started_at, completed_at, updated_at
            FROM operations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        row.map(|r| Self::from_row(&r)).transpose()
    }

    /// Finds an operation by its name.
    pub async fn find_by_name(
        pool: &Pool<Postgres>,
        name: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                id, name, operation_type, target_backend, resource_type, resource_id,
                status, input, output, error_code, error_message, attempt_count,
                max_attempts, next_retry_at, last_error, idempotency_key,
                created_at, started_at, completed_at, updated_at
            FROM operations
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;

        row.map(|r| Self::from_row(&r)).transpose()
    }

    /// Lists operations for a specific resource.
    pub async fn list_by_resource(
        pool: &Pool<Postgres>,
        resource_type: &str,
        resource_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, name, operation_type, target_backend, resource_type, resource_id,
                status, input, output, error_code, error_message, attempt_count,
                max_attempts, next_retry_at, last_error, idempotency_key,
                created_at, started_at, completed_at, updated_at
            FROM operations
            WHERE resource_type = $1 AND resource_id = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(resource_type)
        .bind(resource_id)
        .fetch_all(pool)
        .await?;

        rows.iter().map(|r| Self::from_row(r)).collect()
    }
}

/// Builder for creating new Operation instances.
pub struct OperationBuilder {
    id: Uuid,
    name: String,
    operation_type: OperationType,
    target_backend: TargetBackend,
    resource_type: String,
    resource_id: Uuid,
    status: OperationStatus,
    input: JsonValue,
    max_attempts: i32,
    idempotency_key: Option<String>,
}

impl OperationBuilder {
    /// Sets a custom idempotency key to prevent duplicate operations.
    pub fn idempotency_key(mut self, key: impl Into<String>) -> Self {
        self.idempotency_key = Some(key.into());
        self
    }

    /// Sets a custom maximum number of retry attempts.
    pub fn max_attempts(mut self, max: i32) -> Self {
        self.max_attempts = max;
        self
    }

    /// Creates the operation in the database.
    pub async fn create(self, pool: &Pool<Postgres>) -> Result<Operation, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO operations (
                id, name, operation_type, target_backend, resource_type, resource_id,
                status, input, max_attempts, idempotency_key
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING
                id, name, operation_type, target_backend, resource_type, resource_id,
                status, input, output, error_code, error_message, attempt_count,
                max_attempts, next_retry_at, last_error, idempotency_key,
                created_at, started_at, completed_at, updated_at
            "#,
        )
        .bind(self.id)
        .bind(&self.name)
        .bind(String::from(self.operation_type))
        .bind(String::from(self.target_backend))
        .bind(&self.resource_type)
        .bind(self.resource_id)
        .bind(String::from(self.status))
        .bind(&self.input)
        .bind(self.max_attempts)
        .bind(&self.idempotency_key)
        .fetch_one(pool)
        .await?;

        let operation = Operation::from_row(&row)?;

        tracing::info!(
            operation_id = %operation.id,
            operation_type = %operation.operation_type,
            target_backend = %operation.target_backend,
            "operation created"
        );

        Ok(operation)
    }
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} [{}] {} -> {} ({})",
            self.name, self.status, self.operation_type, self.target_backend, self.resource_type
        )
    }
}
