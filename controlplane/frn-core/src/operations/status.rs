//! Operation status state machine
//!
//! Defines the lifecycle states for operations: PENDING → RUNNING → SUCCEEDED/FAILED/CANCELLED.
//! Operations start in PENDING, transition to RUNNING when picked up by the worker,
//! and end in a terminal state (SUCCEEDED, FAILED, or CANCELLED).

use std::str::FromStr;
use strum_macros::{Display, EnumString, IntoStaticStr};

/// Represents the lifecycle state of an operation.
///
/// State transitions:
/// - PENDING → RUNNING (worker picks up operation)
/// - RUNNING → SUCCEEDED (operation completed successfully)
/// - RUNNING → FAILED (operation failed after all retries)
/// - PENDING/RUNNING → CANCELLED (manually cancelled)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "UPPERCASE")]
pub enum OperationStatus {
    /// Initial state, waiting to be processed.
    #[default]
    Pending,
    /// Currently being executed by a worker.
    Running,
    /// Completed successfully.
    Succeeded,
    /// Failed after exhausting all retry attempts.
    Failed,
    /// Manually cancelled.
    Cancelled,
}

impl OperationStatus {
    /// Returns true if the operation is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            OperationStatus::Succeeded | OperationStatus::Failed | OperationStatus::Cancelled
        )
    }

    /// Returns true if the operation can be retried.
    pub fn is_retryable(&self) -> bool {
        matches!(self, OperationStatus::Pending | OperationStatus::Running)
    }
}

impl From<String> for OperationStatus {
    fn from(value: String) -> Self {
        OperationStatus::from_str(&value).expect("could not parse operation status")
    }
}

impl From<OperationStatus> for String {
    fn from(value: OperationStatus) -> Self {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_display() {
        assert_eq!(OperationStatus::Pending.to_string(), "PENDING");
        assert_eq!(OperationStatus::Running.to_string(), "RUNNING");
        assert_eq!(OperationStatus::Succeeded.to_string(), "SUCCEEDED");
        assert_eq!(OperationStatus::Failed.to_string(), "FAILED");
        assert_eq!(OperationStatus::Cancelled.to_string(), "CANCELLED");
    }

    #[test]
    fn test_status_from_string() {
        assert_eq!(
            OperationStatus::from_str("PENDING").unwrap(),
            OperationStatus::Pending
        );
        assert_eq!(
            OperationStatus::from_str("RUNNING").unwrap(),
            OperationStatus::Running
        );
    }

    #[test]
    fn test_is_terminal() {
        assert!(!OperationStatus::Pending.is_terminal());
        assert!(!OperationStatus::Running.is_terminal());
        assert!(OperationStatus::Succeeded.is_terminal());
        assert!(OperationStatus::Failed.is_terminal());
        assert!(OperationStatus::Cancelled.is_terminal());
    }
}
