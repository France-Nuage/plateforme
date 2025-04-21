//! Error handling for hypervisor operations.
//!
//! This module defines the error types for hypervisor-related operations and
//! provides conversions to and from other error types.

use thiserror::Error;
use tonic::Status;
use uuid::Uuid;

/// Represents errors that can occur during hypervisor operations.
///
/// This enum provides specific error types for hypervisor-related operations,
/// with appropriate error messages and source error information.
#[derive(Debug, Error)]
pub enum Problem {
    /// Error returned when a requested hypervisor cannot be found.
    #[error("hypervisor not found")]
    NotFound(Uuid),

    /// A general error that wraps any other error types not explicitly handled.
    #[error("other")]
    Other {
        /// The source error that caused this problem.
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Converts database errors into hypervisor-specific problems.
///
/// This implementation maps Sea ORM database errors to appropriate
/// hypervisor `Problem` variants.
impl From<sqlx::Error> for Problem {
    fn from(value: sqlx::Error) -> Self {
        Problem::Other {
            source: Box::new(value),
        }
    }
}

/// Converts hypervisor problems into gRPC status codes.
///
/// This implementation ensures that hypervisor-specific errors are
/// mapped to appropriate gRPC status codes for client responses.
impl From<Problem> for Status {
    fn from(value: Problem) -> Self {
        match value {
            Problem::NotFound(_) => Status::not_found(value.to_string()),
            _ => Status::from_error(Box::new(value)),
        }
    }
}
