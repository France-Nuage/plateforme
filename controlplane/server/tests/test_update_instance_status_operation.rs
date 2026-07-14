use common::{
    seed_managed_service, seed_managed_service_instance, seed_managed_service_version,
    worker_context,
};
use frn_core::managed::ManagedServiceInstance;
use workflow::execution::WorkflowExecutionId;
use workflow::fsm::FsmRepository;
use workflow::operations::Operation;
use workflow::operations::update_instance_status::{
    UpdateInstanceStatusError, UpdateInstanceStatusOp,
};

mod common;

async fn seed_instance(pool: &sqlx::PgPool) -> ManagedServiceInstance {
    let service_id = seed_managed_service(pool, "vaultwarden", "Vaultwarden", "security").await;
    let version_id = seed_managed_service_version(
        pool,
        service_id,
        "1.0.0",
        None,
        "oci://registry.example.com/charts/vaultwarden",
    )
    .await;
    seed_managed_service_instance(pool, service_id, version_id, "vaultwarden").await
}

/// Resolves the current FSM state name for an instance's status machine.
async fn status_name(pool: &sqlx::PgPool, status_id: uuid::Uuid) -> String {
    let mut conn = pool.acquire().await.expect("could not acquire connection");
    FsmRepository::current_state_name(&mut conn, &status_id)
        .await
        .expect("could not read status name")
}

fn transition_to(instance_id: uuid::Uuid, new_status: &str) -> UpdateInstanceStatusOp {
    UpdateInstanceStatusOp {
        instance_id,
        new_status: new_status.to_owned(),
        previous_status: None,
    }
}

#[sqlx::test(migrations = "../migrations")]
async fn test_execute_transitions_instance_status(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let instance = seed_instance(&pool).await;
    let ctx = worker_context(&pool).await;

    transition_to(instance.id, "running")
        .execute(ctx, WorkflowExecutionId::new())
        .await
        .expect("transition provisioning -> running must succeed");

    assert_eq!(status_name(&pool, instance.status).await, "running");

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_execute_rejects_an_unreachable_transition(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let instance = seed_instance(&pool).await;
    let ctx = worker_context(&pool).await;

    // `deleted` is not reachable from the initial `provisioning` state.
    let result = transition_to(instance.id, "deleted")
        .execute(ctx, WorkflowExecutionId::new())
        .await;

    assert!(matches!(
        result,
        Err(UpdateInstanceStatusError::InvalidTransition(_))
    ));

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_rollback_restores_the_previous_status(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let instance = seed_instance(&pool).await;
    let ctx = worker_context(&pool).await;

    transition_to(instance.id, "running")
        .execute(ctx.clone(), WorkflowExecutionId::new())
        .await
        .expect("provisioning -> running must succeed");

    let upgrading = transition_to(instance.id, "upgrading")
        .execute(ctx.clone(), WorkflowExecutionId::new())
        .await
        .expect("running -> upgrading must succeed");

    upgrading
        .rollback(ctx, WorkflowExecutionId::new())
        .await
        .expect("rollback must succeed");

    assert_eq!(status_name(&pool, instance.status).await, "running");

    Ok(())
}
