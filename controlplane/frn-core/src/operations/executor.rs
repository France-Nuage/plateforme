//! Operation executor traits and composite executor
//!
//! Provides the `OperationExecutor` trait for implementing backend-specific
//! execution logic, and `CompositeExecutor` for dispatching operations to
//! the appropriate executor based on operation type.

use super::{Operation, OperationType};
use async_trait::async_trait;
use serde_json::Value as JsonValue;
use thiserror::Error;

/// Errors that can occur during operation execution.
#[derive(Debug, Error)]
pub enum ExecutorError {
    /// The executor does not handle this operation type.
    #[error("operation type not handled: {0}")]
    NotHandled(OperationType),

    /// Network or connectivity error with the external system.
    #[error("connectivity error: {0}")]
    Connectivity(String),

    /// Authentication/authorization failure with the external system.
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// The target resource was not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// The external system rejected the request (validation, conflict, etc).
    #[error("rejected: {0}")]
    Rejected(String),

    /// The external system is temporarily unavailable.
    #[error("temporarily unavailable: {0}")]
    TemporarilyUnavailable(String),

    /// Invalid input payload.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Generic internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

impl ExecutorError {
    /// Returns true if this error is retryable.
    ///
    /// Connectivity issues and temporary unavailability are typically retryable.
    /// Authorization and validation errors are not.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ExecutorError::Connectivity(_) | ExecutorError::TemporarilyUnavailable(_)
        )
    }
}

/// Trait for executing operations against external systems.
///
/// Each backend (SpiceDB, Pangolin, Hoop, Kubernetes) implements this trait
/// to handle its specific operation types.
///
/// ## Implementation Guidelines
///
/// 1. Implement `handles()` to return true only for operation types you support
/// 2. Implement `execute()` to perform the actual operation
/// 3. Return appropriate `ExecutorError` variants for error cases
/// 4. Use `serde_json` to parse `operation.input` into your domain types
#[async_trait]
pub trait OperationExecutor: Send + Sync {
    /// Execute the operation and return the result as JSON.
    ///
    /// # Arguments
    /// * `operation` - The operation to execute
    ///
    /// # Returns
    /// * `Ok(JsonValue)` - The output payload on success
    /// * `Err(ExecutorError)` - The error on failure
    async fn execute(&self, operation: &Operation) -> Result<JsonValue, ExecutorError>;

    /// Check if this executor handles the given operation type.
    fn handles(&self, operation_type: &OperationType) -> bool;
}

/// Composite executor that dispatches to the appropriate backend executor.
///
/// Register executors using `register()` and the composite will automatically
/// route operations to the correct executor based on `handles()`.
///
/// ## Example
///
/// ```rust,ignore
/// let executor = CompositeExecutor::new()
///     .register(Box::new(SpiceDbExecutor::new(&client)))
///     .register(Box::new(PangolinExecutor::new(&config)))
///     .register(Box::new(HoopExecutor::new(&config)));
///
/// // Execute will route to the correct executor
/// let result = executor.execute(&operation).await?;
/// ```
pub struct CompositeExecutor {
    executors: Vec<Box<dyn OperationExecutor>>,
}

impl CompositeExecutor {
    /// Creates a new empty composite executor.
    pub fn new() -> Self {
        Self { executors: vec![] }
    }

    /// Registers an executor with the composite.
    ///
    /// Operations will be dispatched to registered executors based on
    /// their `handles()` implementation.
    pub fn register(mut self, executor: Box<dyn OperationExecutor>) -> Self {
        self.executors.push(executor);
        self
    }

    /// Finds the executor that handles the given operation type.
    fn find_executor(&self, operation_type: &OperationType) -> Option<&dyn OperationExecutor> {
        self.executors
            .iter()
            .find(|e| e.handles(operation_type))
            .map(|e| e.as_ref())
    }
}

impl Default for CompositeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OperationExecutor for CompositeExecutor {
    async fn execute(&self, operation: &Operation) -> Result<JsonValue, ExecutorError> {
        let executor = self
            .find_executor(&operation.operation_type)
            .ok_or_else(|| ExecutorError::NotHandled(operation.operation_type))?;

        executor.execute(operation).await
    }

    fn handles(&self, operation_type: &OperationType) -> bool {
        self.find_executor(operation_type).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct TestExecutor {
        handled_types: Vec<OperationType>,
        call_count: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl OperationExecutor for TestExecutor {
        async fn execute(&self, _operation: &Operation) -> Result<JsonValue, ExecutorError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(json!({"executed": true}))
        }

        fn handles(&self, operation_type: &OperationType) -> bool {
            self.handled_types.contains(operation_type)
        }
    }

    #[test]
    fn test_executor_error_retryable() {
        assert!(ExecutorError::Connectivity("timeout".to_string()).is_retryable());
        assert!(ExecutorError::TemporarilyUnavailable("503".to_string()).is_retryable());
        assert!(!ExecutorError::Unauthorized("invalid token".to_string()).is_retryable());
        assert!(!ExecutorError::InvalidInput("bad json".to_string()).is_retryable());
    }

    #[test]
    fn test_composite_handles() {
        let call_count = Arc::new(AtomicUsize::new(0));

        let executor1 = TestExecutor {
            handled_types: vec![
                OperationType::SpiceDbWriteRelationship,
                OperationType::SpiceDbDeleteRelationship,
            ],
            call_count: call_count.clone(),
        };

        let executor2 = TestExecutor {
            handled_types: vec![
                OperationType::PangolinInviteUser,
                OperationType::PangolinRemoveUser,
            ],
            call_count: call_count.clone(),
        };

        let composite = CompositeExecutor::new()
            .register(Box::new(executor1))
            .register(Box::new(executor2));

        assert!(composite.handles(&OperationType::SpiceDbWriteRelationship));
        assert!(composite.handles(&OperationType::PangolinInviteUser));
        assert!(!composite.handles(&OperationType::HoopCreateAgent));
    }
}
