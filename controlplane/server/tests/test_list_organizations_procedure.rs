use std::str::FromStr;

use auth::{
    OpenID,
    mock::{WithJwks, WithWellKnown},
    model::User,
};
use mock_server::MockServer;
use resources::{
    organizations::Organization,
    v1::{ListOrganizationsRequest, resources_client::ResourcesClient},
};
use server::Config;
use sqlx::{Pool, Postgres, types::Uuid};
use tonic::{Code, Request, metadata::MetadataValue};

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_organizations_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new().await.with_well_known().with_jwks();
    Organization::factory().create(&pool).await?;
    let user = User::factory()
        .email("wile.coyote@acme.org".to_owned())
        .create(&pool)
        .await
        .unwrap();
    let token = OpenID::token(&user.email);
    let config = Config::test(&pool, &mock).await?;
    let server_url = format!("http://{}", config.addr);
    let shutdown_tx = server::serve(config).await?;
    let mut client = ResourcesClient::connect(server_url).await?;

    // Act the request to the test_the_status_procedure_works
    let mut request = Request::new(ListOrganizationsRequest::default());
    request.metadata_mut().insert(
        "authorization",
        MetadataValue::from_str(&format!("Bearer {}", &token)).unwrap(),
    );
    let response = client.list_organizations(request).await;

    // Assert the result
    println!("response: {:#?}", &response);
    assert!(response.is_ok());
    assert_eq!(response.unwrap().into_inner().organizations.len(), 1);

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_organizations_procedure_rejects_unauthenticated_users(pool: Pool<Postgres>) {
    // Arrange the grpc server and a client
    let mock = MockServer::new().await.with_well_known().with_jwks();
    let config = Config::test(&pool, &mock).await.unwrap();
    let server_url = format!("http://{}", config.addr);
    let shutdown_tx = server::serve(config).await.unwrap();
    let mut client = ResourcesClient::connect(server_url).await.unwrap();

    // Act the request to the test_the_status_procedure_works
    let request = Request::new(ListOrganizationsRequest::default());
    let response = client.list_organizations(request).await;

    // Assert the result
    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), Code::Unauthenticated);

    // Shutdown the server
    shutdown_tx.send(()).ok();
}

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_organizations_procedure_rejects_calls_with_an_invalid_token(
    pool: Pool<Postgres>,
) {
    // Arrange the grpc server and a client
    let mock = MockServer::new().await.with_well_known().with_jwks();
    let config = Config::test(&pool, &mock).await.unwrap();
    let server_url = format!("http://{}", config.addr);
    let shutdown_tx = server::serve(config).await.unwrap();
    let mut client = ResourcesClient::connect(server_url).await.unwrap();

    // Act the request to the test_the_status_procedure_works
    let mut request = Request::new(ListOrganizationsRequest::default());
    request.metadata_mut().insert(
        "authorization",
        MetadataValue::from_str("Bearer foobar").unwrap(),
    );
    let response = client.list_organizations(request).await;

    // Assert the result
    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), Code::Unauthenticated);

    // Shutdown the server
    shutdown_tx.send(()).ok();
}

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_organizations_procedure_only_returns_the_user_organization(
    pool: Pool<Postgres>,
) {
    // Arrange the grpc server and a client
    let mock = MockServer::new().await.with_well_known().with_jwks();
    let organization = Organization::factory()
        .id(Uuid::new_v4())
        .create(&pool)
        .await
        .unwrap();
    let _other_organization = Organization::factory()
        .id(Uuid::new_v4())
        .create(&pool)
        .await
        .unwrap();
    let user = User::factory()
        .email("wile.coyote@acme.org".to_owned())
        .organization_id(organization.id)
        .create(&pool)
        .await
        .unwrap();
    let token = OpenID::token(&user.email);
    let config = Config::test(&pool, &mock).await.unwrap();
    let server_url = format!("http://{}", config.addr);
    let shutdown_tx = server::serve(config).await.unwrap();
    let mut client = ResourcesClient::connect(server_url).await.unwrap();

    // Act the request to the test_the_status_procedure_works
    let mut request = Request::new(ListOrganizationsRequest::default());
    request.metadata_mut().insert(
        "authorization",
        MetadataValue::from_str(&format!("Bearer {}", &token)).unwrap(),
    );
    let response = client.list_organizations(request).await;

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(response.unwrap().into_inner().organizations.len(), 1);

    // Shutdown the server
    shutdown_tx.send(()).ok();
}

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_organizations_procedure_returns_all_organizations_for_an_admin(
    pool: Pool<Postgres>,
) {
    // Arrange the grpc server and a client
    let mock = MockServer::new().await.with_well_known().with_jwks();
    let organization = Organization::factory()
        .id(Uuid::new_v4())
        .create(&pool)
        .await
        .unwrap();
    let _other_organization = Organization::factory()
        .id(Uuid::new_v4())
        .create(&pool)
        .await
        .unwrap();
    let user = User::factory()
        .email("wile.coyote@acme.org".to_owned())
        .organization_id(organization.id)
        .is_admin(true)
        .create(&pool)
        .await
        .unwrap();
    let token = OpenID::token(&user.email);
    let config = Config::test(&pool, &mock).await.unwrap();
    let server_url = format!("http://{}", config.addr);
    let shutdown_tx = server::serve(config).await.unwrap();
    let mut client = ResourcesClient::connect(server_url).await.unwrap();

    // Act the request to the test_the_status_procedure_works
    let mut request = Request::new(ListOrganizationsRequest::default());
    request.metadata_mut().insert(
        "authorization",
        MetadataValue::from_str(&format!("Bearer {}", &token)).unwrap(),
    );
    let response = client.list_organizations(request).await;

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(response.unwrap().into_inner().organizations.len(), 2);

    // Shutdown the server
    shutdown_tx.send(()).ok();
}
