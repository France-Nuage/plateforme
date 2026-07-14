//! Shared interpretation of Kubernetes API results for the k8s operations.
//!
//! The operations are idempotent saga steps: creating a resource that already
//! exists (HTTP 409) or deleting one that is already gone (HTTP 404) is a benign
//! outcome for a retried step, not a failure. Centralizing that mapping keeps
//! each operation's control flow to a single `match` and keeps the decision
//! unit-testable in isolation from a live API server.

use kube::Error as KubeError;

/// Outcome of a create-style Kubernetes call.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CreateOutcome {
    /// The resource was created.
    Created,
    /// The resource already existed (HTTP 409); benign for a retried step.
    AlreadyExists,
}

/// Outcome of a delete-style Kubernetes call.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DeleteOutcome {
    /// The resource was deleted.
    Deleted,
    /// The resource was already gone (HTTP 404); benign for a retried step.
    AlreadyGone,
}

/// Maps a create result, turning an HTTP 409 conflict into
/// [`CreateOutcome::AlreadyExists`]. Any other API or transport error is
/// propagated unchanged for the caller to wrap in its own error variant.
pub(crate) fn classify_create<T>(result: Result<T, KubeError>) -> Result<CreateOutcome, KubeError> {
    match result {
        Ok(_) => Ok(CreateOutcome::Created),
        Err(KubeError::Api(err)) if err.code == 409 => Ok(CreateOutcome::AlreadyExists),
        Err(err) => Err(err),
    }
}

/// Maps a delete result, turning an HTTP 404 not-found into
/// [`DeleteOutcome::AlreadyGone`]. Any other error is propagated unchanged.
pub(crate) fn classify_delete<T>(result: Result<T, KubeError>) -> Result<DeleteOutcome, KubeError> {
    match result {
        Ok(_) => Ok(DeleteOutcome::Deleted),
        Err(KubeError::Api(err)) if err.code == 404 => Ok(DeleteOutcome::AlreadyGone),
        Err(err) => Err(err),
    }
}

/// Maps a get result, turning an HTTP 404 not-found into `Ok(None)` so callers
/// can treat a missing resource as an empty read rather than an error.
pub(crate) fn classify_get<T>(result: Result<T, KubeError>) -> Result<Option<T>, KubeError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(KubeError::Api(err)) if err.code == 404 => Ok(None),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::{CreateOutcome, DeleteOutcome, classify_create, classify_delete, classify_get};
    use kube::Error as KubeError;
    use kube::core::ErrorResponse;

    fn api_error(code: u16) -> KubeError {
        KubeError::Api(ErrorResponse {
            status: "Failure".to_owned(),
            message: "boom".to_owned(),
            reason: "Reason".to_owned(),
            code,
        })
    }

    #[test]
    fn create_success_is_created() {
        assert_eq!(
            classify_create::<()>(Ok(())).unwrap(),
            CreateOutcome::Created
        );
    }

    #[test]
    fn create_conflict_is_already_exists() {
        assert_eq!(
            classify_create::<()>(Err(api_error(409))).unwrap(),
            CreateOutcome::AlreadyExists
        );
    }

    #[test]
    fn create_other_api_error_is_propagated() {
        assert!(classify_create::<()>(Err(api_error(500))).is_err());
    }

    #[test]
    fn delete_success_is_deleted() {
        assert_eq!(
            classify_delete::<()>(Ok(())).unwrap(),
            DeleteOutcome::Deleted
        );
    }

    #[test]
    fn delete_not_found_is_already_gone() {
        assert_eq!(
            classify_delete::<()>(Err(api_error(404))).unwrap(),
            DeleteOutcome::AlreadyGone
        );
    }

    #[test]
    fn delete_other_api_error_is_propagated() {
        assert!(classify_delete::<()>(Err(api_error(500))).is_err());
    }

    #[test]
    fn get_success_yields_the_value() {
        assert_eq!(classify_get(Ok(42)).unwrap(), Some(42));
    }

    #[test]
    fn get_not_found_yields_none() {
        assert_eq!(classify_get::<i32>(Err(api_error(404))).unwrap(), None);
    }

    #[test]
    fn get_other_api_error_is_propagated() {
        assert!(classify_get::<i32>(Err(api_error(500))).is_err());
    }
}
