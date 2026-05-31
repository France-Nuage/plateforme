use std::env;
use std::error::Error as StdError;
use std::io::Write;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use chrono::{DateTime, Utc};
use frn_core::kubernetes::KubernetesClusters;
use frn_crypto::Kek;
use frn_rpc::v1::workflow::WorkflowExecution as ProtoExecution;
use frn_rpc::v1::workflow::{
    GetStatusRequest, GetStatusResponse, Initiator, NextRequest, ScheduleRequest, ScheduleResponse,
    UnlockRequest, from_timestamp, status_from_proto, to_timestamp,
    workflow_engine_client::WorkflowEngineClient,
};
use futures::FutureExt;
use kube::Client as KubeClient;
use spicedb::SpiceDB;
use sqlx::PgPool;
use tempfile::NamedTempFile;
use tokio::signal;
use tokio::time::sleep;
use tonic::Status;
use tracing::{debug, error, info};
use uuid::Uuid;

use workflow::WorkerContext;
use workflow::execution::{
    WorkflowExecution, WorkflowExecutionId, WorkflowExecutionStatus, WorkflowInitiator,
};
use workflow::operations::{Operation, OperationError};
use workflow::workflows::{WorkflowDefinition, WorkflowDefinitions};

enum ProcessOutcome {
    Completed,
    WaitingForDependencies {
        dependencies: Vec<WorkflowExecutionId>,
        next_retry_at: DateTime<Utc>,
    },
    ScheduledSubWorkflows {
        dependencies: Vec<WorkflowExecutionId>,
    },
    Failed {
        status: WorkflowExecutionStatus,
    },
}

const MAX_OPERATION_ROUNDS: u32 = 100;
const SUB_WORKFLOW_MAX_RETRY: i32 = 3;
const PROCESS_TIMEOUT: Duration = Duration::from_secs(4 * 60);
const MAX_BACKOFF: Duration = Duration::from_secs(60 * 60);

type Client = WorkflowEngineClient<tonic::transport::Channel>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    tracing_subscriber::fmt().init();
    info!("starting workflow worker...");

    let server_url = env::var("WORKFLOW_SERVER_URL").expect("WORKFLOW_SERVER_URL must be set");
    let worker_token = env::var("WORKER_TOKEN").expect("WORKER_TOKEN must be set");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let spicedb_url = env::var("SPICEDB_URL").expect("SPICEDB_URL must be set");
    let spicedb_token =
        env::var("SPICEDB_GRPC_PRESHARED_KEY").expect("SPICEDB_GRPC_PRESHARED_KEY must be set");
    let poll_interval_ms: u64 = env::var("POLL_INTERVAL_MS")
        .unwrap_or_else(|_| "2000".to_owned())
        .parse()
        .expect("POLL_INTERVAL_MS must be a number");

    let default_storage_class = env::var("MANAGED_DEFAULT_STORAGE_CLASS").ok();
    let kek = Arc::new(
        Kek::from_base64(
            &env::var("KUBECONFIG_ENCRYPTION_KEY").expect("KUBECONFIG_ENCRYPTION_KEY must be set"),
        )
        .expect("KUBECONFIG_ENCRYPTION_KEY must be base64-encoded 32 bytes"),
    );

    let pool = PgPool::connect(&database_url).await?;
    let spicedb = SpiceDB::connect(&spicedb_url, &spicedb_token).await?;
    let kube = KubeClient::try_default().await?;

    let ctx = WorkerContext {
        pool,
        spicedb,
        kube,
        platform_config: workflow::PlatformConfig {
            default_storage_class,
        },
        kek,
        kubeconfig_path: None,
    };

    let shutdown = Arc::new(AtomicBool::new(false));
    tokio::spawn(shutdown_signal(shutdown.clone()));

    run(server_url, worker_token, ctx, poll_interval_ms, shutdown).await
}

async fn run(
    server_url: String,
    worker_token: String,
    ctx: WorkerContext,
    poll_interval_ms: u64,
    shutdown: Arc<AtomicBool>,
) -> Result<(), Box<dyn StdError>> {
    let poll_interval = Duration::from_millis(poll_interval_ms);

    loop {
        let result = AssertUnwindSafe(worker_loop(
            &server_url,
            &worker_token,
            &ctx,
            poll_interval,
            &shutdown,
        ))
        .catch_unwind()
        .await;

        match result {
            Ok(Ok(true)) => break,
            Ok(Ok(false)) => continue,
            Ok(Err(e)) => {
                error!("worker loop error, restarting: {e}");
                sleep(Duration::from_secs(1)).await;
            }
            Err(panic_info) => {
                error!("worker panic caught, restarting loop: {:?}", panic_info);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

    Ok(())
}

async fn worker_loop(
    server_url: &str,
    worker_token: &str,
    ctx: &WorkerContext,
    poll_interval: Duration,
    shutdown: &AtomicBool,
) -> Result<bool, Box<dyn StdError>> {
    let mut client = WorkflowEngineClient::connect(server_url.to_owned()).await?;
    let mut consecutive_errors: u32 = 0;

    loop {
        if shutdown.load(Ordering::Relaxed) {
            info!("shutting down worker");
            return Ok(true);
        }

        let mut request = tonic::Request::new(NextRequest {});
        inject_token(&mut request, worker_token)?;

        match client.next(request).await {
            Ok(response) => {
                consecutive_errors = 0;

                let Some(proto_exec) = response.into_inner().execution else {
                    debug!(
                        "no execution found, retrying in {} ms",
                        poll_interval.as_millis()
                    );
                    sleep(poll_interval).await;
                    continue;
                };

                let mut execution: WorkflowExecution = proto_exec
                    .try_into()
                    .map_err(|s: Status| -> Box<dyn StdError> { Box::new(s) })?;
                info!(execution_id = %execution.execution_id, "processing workflow");

                let process_result = tokio::time::timeout(
                    PROCESS_TIMEOUT,
                    process_execution(&mut client, worker_token, ctx.clone(), &mut execution),
                )
                .await;

                match process_result {
                    Ok(Ok(outcome)) => {
                        apply_outcome(&mut execution, outcome);
                    }
                    Ok(Err(err)) => {
                        error!(execution_id = %execution.execution_id, error = %err, "execution failed");
                        handle_failure(&mut execution, &err);
                    }
                    Err(_) => {
                        error!(execution_id = %execution.execution_id, "execution timed out after {:?}", PROCESS_TIMEOUT);
                        handle_failure(&mut execution, &ProcessError::Timeout);
                    }
                }

                if let Err(e) = send_unlock(&mut client, worker_token, &execution).await {
                    error!(execution_id = %execution.execution_id, error = %e, "failed to unlock, will retry after lock expiry");
                }
            }
            Err(status) => {
                consecutive_errors = consecutive_errors.saturating_add(1);
                let backoff = poll_interval * 2u32.saturating_pow(consecutive_errors.min(5));
                error!(
                    "failed to poll: {status}, retrying in {} ms",
                    backoff.as_millis()
                );
                sleep(backoff).await;
            }
        }
    }
}

async fn process_execution(
    client: &mut Client,
    worker_token: &str,
    mut ctx: WorkerContext,
    execution: &mut WorkflowExecution,
) -> Result<ProcessOutcome, ProcessError> {
    if execution.hard_try_count >= execution.max_try_count {
        return Err(ProcessError::MaxRetriesExceeded);
    }

    let _kubeconfig_guard = match execution.definition.target_cluster_id() {
        Some(cluster_id) => Some(
            resolve_cluster_kubeconfig(&mut ctx, cluster_id)
                .await
                .map_err(ProcessError::WorkflowError)?,
        ),
        None => None,
    };

    // Phase 1: Check dependencies
    for dependency in &execution.dependencies {
        let resp = get_status(client, worker_token, *dependency).await?;
        let dep_status =
            status_from_proto(resp.status).map_err(|e| ProcessError::WorkflowError(Box::new(e)))?;
        if dep_status == WorkflowExecutionStatus::Failed {
            return Err(ProcessError::DependencyFailed(*dependency));
        }
        if dep_status != WorkflowExecutionStatus::Completed {
            info!(dependency = %dependency, "waiting for dependency");
            let next_retry_at = resp
                .next_retry_at
                .as_ref()
                .map(from_timestamp)
                .transpose()?
                .unwrap_or_else(Utc::now)
                + chrono::Duration::seconds(1);
            return Ok(ProcessOutcome::WaitingForDependencies {
                dependencies: execution.dependencies.clone(),
                next_retry_at,
            });
        }
    }

    // Phase 2: Schedule prerequisites
    let dependencies = execution
        .definition
        .needed_workflows(ctx.clone())
        .await
        .map_err(ProcessError::WorkflowError)?;

    if !dependencies.is_empty() {
        let mut scheduled_deps = Vec::new();
        for dependency in dependencies {
            let resp = send_schedule(
                client,
                worker_token,
                &dependency,
                SUB_WORKFLOW_MAX_RETRY,
                WorkflowInitiator::Workflow(execution.execution_id),
                None,
            )
            .await?;

            let dep_id: WorkflowExecutionId = resp
                .execution
                .ok_or(ProcessError::MissingResponse)?
                .execution_id
                .parse()
                .map_err(|_| ProcessError::MissingResponse)?;

            scheduled_deps.push(dep_id);
        }

        return Ok(ProcessOutcome::ScheduledSubWorkflows {
            dependencies: scheduled_deps,
        });
    }

    // Phase 3: Execute operations
    let mut rollbacks: Vec<workflow::operations::Operations> = Vec::new();
    let mut rounds: u32 = 0;

    loop {
        if rounds >= MAX_OPERATION_ROUNDS {
            error!(execution_id = %execution.execution_id, "max operation rounds exceeded, aborting");
            for op in rollbacks.into_iter().rev() {
                if let Err(e) = op.rollback(ctx.clone(), execution.execution_id).await {
                    error!("rollback failed: {e}");
                    break;
                }
            }
            return Err(ProcessError::MaxOperationRoundsExceeded);
        }
        rounds += 1;

        let ops = execution
            .definition
            .next_operations(ctx.clone())
            .await
            .map_err(ProcessError::WorkflowError)?;

        if ops.is_empty() {
            break;
        }

        let results = futures::future::join_all(
            ops.into_iter()
                .map(|op| op.execute(ctx.clone(), execution.execution_id)),
        )
        .await;

        let (errors, successes): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_err);
        let errors: Vec<_> = errors.into_iter().filter_map(Result::err).collect();
        let successes: Vec<_> = successes.into_iter().filter_map(Result::ok).collect();
        rollbacks.extend(successes);

        if !errors.is_empty() {
            for op in rollbacks.into_iter().rev() {
                if let Err(e) = op.rollback(ctx.clone(), execution.execution_id).await {
                    error!("rollback failed: {e}");
                    break;
                }
            }
            return Err(ProcessError::OperationFailed(errors));
        }
    }

    // Phase 4: Commit
    let mut commit_errors: Vec<String> = Vec::new();
    for op in rollbacks {
        if let Err(e) = op.commit(ctx.clone(), execution.execution_id).await {
            error!(execution_id = %execution.execution_id, "commit failed: {e}");
            commit_errors.push(e.to_string());
        }
    }

    if !commit_errors.is_empty() {
        error!(
            execution_id = %execution.execution_id,
            errors = ?commit_errors,
            "partial commit failure, manual investigation required"
        );
        return Ok(ProcessOutcome::Failed {
            status: WorkflowExecutionStatus::Failed,
        });
    }

    // Phase 5: Schedule follow-ups
    let next_workflows = execution
        .definition
        .next_workflows(ctx)
        .await
        .map_err(ProcessError::WorkflowError)?;

    for scheduled in next_workflows {
        send_schedule(
            client,
            worker_token,
            &scheduled.workflow,
            SUB_WORKFLOW_MAX_RETRY,
            WorkflowInitiator::Workflow(execution.execution_id),
            scheduled.schedule_at,
        )
        .await?;
    }

    info!(execution_id = %execution.execution_id, "workflow completed");
    Ok(ProcessOutcome::Completed)
}

async fn resolve_cluster_kubeconfig(
    ctx: &mut WorkerContext,
    cluster_id: Uuid,
) -> Result<NamedTempFile, Box<dyn StdError>> {
    let clusters = KubernetesClusters::new(ctx.pool.clone(), ctx.kek.clone());
    let kubeconfig_yaml = clusters.decrypt_kubeconfig(cluster_id).await?;
    let kube = KubernetesClusters::client_from_kubeconfig(&kubeconfig_yaml).await?;
    let mut file = NamedTempFile::new()?;
    // The kubeconfig holds live cluster credentials in plaintext on disk; lock
    // the file to the worker user before writing so no other local process can
    // read it during its short lifetime.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.as_file()
            .set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    file.write_all(kubeconfig_yaml.as_bytes())?;
    ctx.kube = kube;
    ctx.kubeconfig_path = Some(file.path().to_path_buf());
    Ok(file)
}

fn apply_outcome(execution: &mut WorkflowExecution, outcome: ProcessOutcome) {
    match outcome {
        ProcessOutcome::Completed => {
            execution.status = WorkflowExecutionStatus::Completed;
        }
        ProcessOutcome::WaitingForDependencies {
            dependencies,
            next_retry_at,
        } => {
            execution.status = WorkflowExecutionStatus::WillRetry;
            execution.dependencies = dependencies;
            execution.next_retry_at = next_retry_at;
        }
        ProcessOutcome::ScheduledSubWorkflows { dependencies } => {
            execution.status = WorkflowExecutionStatus::WillRetry;
            execution.dependencies = dependencies;
        }
        ProcessOutcome::Failed { status } => {
            execution.status = status;
        }
    }
}

fn handle_failure(execution: &mut WorkflowExecution, err: &ProcessError) {
    execution.status = WorkflowExecutionStatus::WillRetry;
    execution.soft_try_count += 1;

    match err {
        ProcessError::OperationFailed(errors) => {
            if errors.iter().any(|e| e.is_violated_invariant()) {
                execution.status = WorkflowExecutionStatus::Failed;
            }
            if errors.iter().any(|e| e.consume_retry()) {
                execution.hard_try_count += 1;
                if execution.hard_try_count >= execution.max_try_count {
                    execution.status = WorkflowExecutionStatus::Failed;
                }
            }
        }
        ProcessError::DependencyFailed(_) => {
            execution.hard_try_count += 1;
            execution.status = WorkflowExecutionStatus::Failed;
        }
        _ => {
            execution.hard_try_count += 1;
            if execution.hard_try_count >= execution.max_try_count {
                execution.status = WorkflowExecutionStatus::Failed;
            }
        }
    }

    let backoff =
        Duration::from_secs(2u64.saturating_pow(execution.hard_try_count as u32)).min(MAX_BACKOFF);
    execution.next_retry_at =
        Utc::now() + chrono::Duration::from_std(backoff).unwrap_or(chrono::Duration::hours(1));
}

fn inject_token<T>(request: &mut tonic::Request<T>, token: &str) -> Result<(), Box<dyn StdError>> {
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse()?);
    Ok(())
}

async fn send_unlock(
    client: &mut Client,
    worker_token: &str,
    execution: &WorkflowExecution,
) -> Result<(), Box<dyn StdError>> {
    let proto = ProtoExecution::try_from(execution)?;
    let mut request = tonic::Request::new(UnlockRequest {
        execution: Some(proto),
    });
    inject_token(&mut request, worker_token)?;
    client.unlock(request).await?;
    Ok(())
}

async fn send_schedule(
    client: &mut Client,
    worker_token: &str,
    definition: &WorkflowDefinitions,
    max_retry: i32,
    initiated_by: WorkflowInitiator,
    schedule_at: Option<DateTime<Utc>>,
) -> Result<ScheduleResponse, Box<dyn StdError>> {
    let mut request = tonic::Request::new(ScheduleRequest {
        definition: serde_json::to_string(definition)?,
        max_retry,
        initiated_by: Some(Initiator::from(&initiated_by)),
        schedule_at: schedule_at.map(to_timestamp),
    });
    inject_token(&mut request, worker_token)?;
    Ok(client.schedule(request).await?.into_inner())
}

async fn get_status(
    client: &mut Client,
    worker_token: &str,
    execution_id: WorkflowExecutionId,
) -> Result<GetStatusResponse, Box<dyn StdError>> {
    let mut request = tonic::Request::new(GetStatusRequest {
        execution_id: execution_id.to_string(),
    });
    inject_token(&mut request, worker_token)?;
    Ok(client.get_status(request).await?.into_inner())
}

#[derive(Debug, thiserror::Error)]
enum ProcessError {
    #[error("operation failed")]
    OperationFailed(Vec<Arc<dyn OperationError + Send + Sync>>),
    #[error("max retries exceeded")]
    MaxRetriesExceeded,
    #[error("max operation rounds exceeded")]
    MaxOperationRoundsExceeded,
    #[error("workflow error: {0}")]
    WorkflowError(Box<dyn StdError>),
    #[error("dependency {0} failed")]
    DependencyFailed(WorkflowExecutionId),
    #[error("missing response from server")]
    MissingResponse,
    #[error("execution timed out")]
    Timeout,
}

impl From<tonic::Status> for ProcessError {
    fn from(status: tonic::Status) -> Self {
        ProcessError::WorkflowError(Box::new(status))
    }
}

impl From<Box<dyn StdError>> for ProcessError {
    fn from(err: Box<dyn StdError>) -> Self {
        ProcessError::WorkflowError(err)
    }
}

async fn shutdown_signal(shutdown: Arc<AtomicBool>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install ctrl+c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }

    shutdown.store(true, Ordering::Relaxed);
}
