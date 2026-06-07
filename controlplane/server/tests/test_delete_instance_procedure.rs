mod common;

use common::{
    Api, OnBehalfOf, seed_managed_service, seed_managed_service_instance,
    seed_managed_service_version,
};
use frn_rpc::v1::managed::DeleteInstanceRequest;
use tonic::{Code, Request};
use uuid::Uuid;
use workflow::fsm::FsmRepository;

#[sqlx::test(migrations = "../migrations")]
async fn test_delete_instance_schedules_workflow(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let service_id = seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;
    let version_id = seed_managed_service_version(
        &pool,
        service_id,
        "1.0.0",
        None,
        "oci://registry.example.com/charts/vaultwarden",
    )
    .await;

    let instance = seed_managed_service_instance(&pool, service_id, version_id, "vaultwarden").await;

    // A freshly created instance is in 'provisioning'; the FSM only allows
    // deletion from 'running' or 'failed', so advance it to 'running' first.
    let mut conn = pool.acquire().await?;
    FsmRepository::state_machine_transition(&mut conn, &instance.status, "running".to_owned())
        .await
        .expect("could not transition instance to running");
    drop(conn);

    let response = api
        .managed
        .services
        .delete_instance(
            Request::new(DeleteInstanceRequest {
                instance_id: instance.id.to_string(),
            })
            .on_behalf_of(&api.service_account),
        )
        .await;

    assert!(response.is_ok());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_delete_instance_returns_not_found(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .managed
        .services
        .delete_instance(
            Request::new(DeleteInstanceRequest {
                instance_id: Uuid::new_v4().to_string(),
            })
            .on_behalf_of(&api.service_account),
        )
        .await;

    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), Code::NotFound);

    Ok(())
}
