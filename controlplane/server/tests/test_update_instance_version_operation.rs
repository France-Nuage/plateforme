use common::{
    seed_managed_service, seed_managed_service_instance, seed_managed_service_version,
    worker_context,
};
use uuid::Uuid;
use workflow::execution::WorkflowExecutionId;
use workflow::operations::Operation;
use workflow::operations::update_instance_version::UpdateInstanceVersionOp;

mod common;

/// Seeds a service with two versions and an instance pinned to the first,
/// returning the instance id and both version ids.
async fn seed_two_versions(pool: &sqlx::PgPool) -> (Uuid, Uuid, Uuid) {
    let service_id = seed_managed_service(pool, "vaultwarden", "Vaultwarden", "security").await;
    let v1 = seed_managed_service_version(
        pool,
        service_id,
        "1.0.0",
        None,
        "oci://registry.example.com/charts/vaultwarden",
    )
    .await;
    let v2 = seed_managed_service_version(
        pool,
        service_id,
        "2.0.0",
        None,
        "oci://registry.example.com/charts/vaultwarden",
    )
    .await;
    let instance = seed_managed_service_instance(pool, service_id, v1, "vaultwarden").await;
    (instance.id, v1, v2)
}

async fn instance_version(pool: &sqlx::PgPool, instance_id: Uuid) -> Uuid {
    let (version_id,): (Uuid,) =
        sqlx::query_as("SELECT version_id FROM managed.service_instance WHERE id = $1")
            .bind(instance_id)
            .fetch_one(pool)
            .await
            .expect("could not read instance version");
    version_id
}

fn update_to(instance_id: Uuid, version_id: Uuid) -> UpdateInstanceVersionOp {
    UpdateInstanceVersionOp {
        instance_id,
        version_id,
        previous_version_id: None,
    }
}

#[sqlx::test(migrations = "../migrations")]
async fn test_execute_updates_the_instance_version(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (instance_id, _v1, v2) = seed_two_versions(&pool).await;
    let ctx = worker_context(&pool).await;

    update_to(instance_id, v2)
        .execute(ctx, WorkflowExecutionId::new())
        .await
        .expect("version update must succeed");

    assert_eq!(instance_version(&pool, instance_id).await, v2);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_rollback_restores_the_previous_version(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (instance_id, v1, v2) = seed_two_versions(&pool).await;
    let ctx = worker_context(&pool).await;

    let updated = update_to(instance_id, v2)
        .execute(ctx.clone(), WorkflowExecutionId::new())
        .await
        .expect("version update must succeed");

    updated
        .rollback(ctx, WorkflowExecutionId::new())
        .await
        .expect("rollback must succeed");

    assert_eq!(instance_version(&pool, instance_id).await, v1);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_execute_fails_for_an_unknown_instance(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_instance_id, _v1, v2) = seed_two_versions(&pool).await;
    let ctx = worker_context(&pool).await;

    let result = update_to(Uuid::new_v4(), v2)
        .execute(ctx, WorkflowExecutionId::new())
        .await;

    assert!(result.is_err());

    Ok(())
}
