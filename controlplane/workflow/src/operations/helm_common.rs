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
