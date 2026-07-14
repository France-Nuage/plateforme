//! Shared process wiring for the helm-based operations.
//!
//! Centralizes how `helm` is invoked (binary, target kubeconfig, optional piped
//! stdin) so each operation only owns its arguments and the interpretation of
//! the result. Keeps a single place to evolve cross-cutting concerns such as
//! global flags or timeouts.

use std::io::Error as IoError;
use std::process::{Output, Stdio};

use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::WorkerContext;

/// Runs `helm <args>` against the worker's target kubeconfig and returns the
/// raw process output.
pub(crate) async fn helm_run(ctx: &WorkerContext, args: &[&str]) -> Result<Output, IoError> {
    let mut command = Command::new("helm");
    command.args(args);
    ctx.apply_kubeconfig(&mut command);
    command.output().await
}

/// Like [`helm_run`] but pipes `stdin_values` to helm's standard input, for
/// commands invoked with `--values -`.
pub(crate) async fn helm_run_with_stdin(
    ctx: &WorkerContext,
    args: &[&str],
    stdin_values: &[u8],
) -> Result<Output, IoError> {
    let mut command = Command::new("helm");
    command.args(args);
    ctx.apply_kubeconfig(&mut command);
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().expect("stdin was configured as piped");
    stdin.write_all(stdin_values).await?;
    drop(stdin);

    child.wait_with_output().await
}

/// How a finished helm invocation was interpreted.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum HelmOutcome {
    /// Helm applied the change (process exited zero).
    Applied,
    /// Helm exited non-zero, but its stderr matched a known idempotent marker
    /// (e.g. a release already installed or already gone), so a retry that
    /// races an already-reconciled state must be treated as a success rather
    /// than consuming a hard retry.
    AlreadyReconciled,
}

/// Interprets a finished helm process from whether it succeeded and its
/// captured stderr.
///
/// A zero exit is [`HelmOutcome::Applied`]. A non-zero exit whose stderr
/// contains any of `idempotent_markers` is [`HelmOutcome::AlreadyReconciled`].
/// Any other non-zero exit returns the (lossily decoded) stderr as the failure
/// message, which the caller wraps in its own `Failed` error variant.
pub(crate) fn classify_helm_result(
    success: bool,
    stderr: &[u8],
    idempotent_markers: &[&str],
) -> Result<HelmOutcome, String> {
    if success {
        return Ok(HelmOutcome::Applied);
    }

    let stderr = String::from_utf8_lossy(stderr);
    if idempotent_markers
        .iter()
        .any(|marker| stderr.contains(marker))
    {
        Ok(HelmOutcome::AlreadyReconciled)
    } else {
        Err(stderr.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::{HelmOutcome, classify_helm_result};

    const NAME_IN_USE: &str = "cannot re-use a name that is still in use";

    #[test]
    fn success_is_applied_regardless_of_stderr() {
        let outcome = classify_helm_result(true, b"noise on stderr", &[NAME_IN_USE]);

        assert_eq!(outcome, Ok(HelmOutcome::Applied));
    }

    #[test]
    fn failure_matching_a_marker_is_already_reconciled() {
        let stderr = b"Error: INSTALLATION FAILED: cannot re-use a name that is still in use";

        let outcome = classify_helm_result(false, stderr, &[NAME_IN_USE]);

        assert_eq!(outcome, Ok(HelmOutcome::AlreadyReconciled));
    }

    #[test]
    fn failure_matching_any_of_several_markers_is_already_reconciled() {
        let stderr =
            b"Error: UPGRADE FAILED: another operation is in progress: has no deployed releases";

        let outcome = classify_helm_result(
            false,
            stderr,
            &["no changes since last release", "has no deployed releases"],
        );

        assert_eq!(outcome, Ok(HelmOutcome::AlreadyReconciled));
    }

    #[test]
    fn failure_without_a_marker_returns_the_stderr() {
        let outcome = classify_helm_result(false, b"Error: connection refused", &[NAME_IN_USE]);

        assert_eq!(outcome, Err("Error: connection refused".to_owned()));
    }

    #[test]
    fn failure_with_no_markers_always_returns_the_stderr() {
        let outcome = classify_helm_result(false, b"Error: rollback failed", &[]);

        assert_eq!(outcome, Err("Error: rollback failed".to_owned()));
    }

    #[test]
    fn non_utf8_stderr_is_decoded_lossily_without_panicking() {
        let outcome = classify_helm_result(false, &[0xff, 0xfe, 0x00], &[NAME_IN_USE]);

        assert!(outcome.is_err());
    }
}
