use crate::common::{Api, IntoCi, seed_managed_service, seed_managed_service_version};
use frn_rpc::v1::managed::{ListVersionsRequest, RegisterVersionRequest};
use tonic::{Code, Request};

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_list_versions_returns_empty_when_no_versions(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;

    let response = api
        .managed
        .services
        .list_versions(Request::new(ListVersionsRequest {
            service_slug: "vaultwarden".to_owned(),
        }))
        .await;

    assert!(response.is_ok());
    let resp = response.unwrap().into_inner();
    assert!(resp.versions.is_empty());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_list_versions_returns_versions_for_service(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let service_id = seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;

    seed_managed_service_version(
        &pool,
        service_id,
        "1.0.0",
        Some("1.30.0"),
        "oci://registry.example.com/vaultwarden:1.0.0",
    )
    .await;

    seed_managed_service_version(
        &pool,
        service_id,
        "1.1.0",
        Some("1.31.0"),
        "oci://registry.example.com/vaultwarden:1.1.0",
    )
    .await;

    let response = api
        .managed
        .services
        .list_versions(Request::new(ListVersionsRequest {
            service_slug: "vaultwarden".to_owned(),
        }))
        .await;

    assert!(response.is_ok());
    let resp = response.unwrap().into_inner();
    assert_eq!(resp.versions.len(), 2);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_list_versions_returns_not_found_for_unknown_service(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .managed
        .services
        .list_versions(Request::new(ListVersionsRequest {
            service_slug: "nonexistent".to_owned(),
        }))
        .await;

    assert!(response.is_err());
    let status = response.unwrap_err();
    assert_eq!(status.code(), Code::NotFound);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_list_versions_exposes_ui_schema(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;

    let ui_schema_json = r#"{"ui:order":["domain"],"domain":{"ui:title":"Domaine"}}"#;

    api.managed
        .services
        .register_version(
            Request::new(RegisterVersionRequest {
                service_slug: "vaultwarden".to_owned(),
                chart_version: "1.0.0".to_owned(),
                app_version: None,
                oci_reference: "oci://registry.example.com/vaultwarden:1.0.0".to_owned(),
                configurable_values_schema: Some(r#"{"type":"object"}"#.to_owned()),
                ui_schema: Some(ui_schema_json.to_owned()),
            })
            .into_ci(),
        )
        .await
        .expect("register_version must succeed");

    let response = api
        .managed
        .services
        .list_versions(Request::new(ListVersionsRequest {
            service_slug: "vaultwarden".to_owned(),
        }))
        .await
        .expect("list_versions must succeed");

    let versions = response.into_inner().versions;
    assert_eq!(versions.len(), 1);
    let returned: serde_json::Value = serde_json::from_str(
        versions[0]
            .ui_schema
            .as_deref()
            .expect("ui_schema must be exposed in list_versions"),
    )?;
    let expected: serde_json::Value = serde_json::from_str(ui_schema_json)?;
    assert_eq!(returned, expected);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_list_versions_rejects_empty_slug(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .managed
        .services
        .list_versions(Request::new(ListVersionsRequest {
            service_slug: String::new(),
        }))
        .await;

    assert!(response.is_err());
    let status = response.unwrap_err();
    assert_eq!(status.code(), Code::InvalidArgument);

    Ok(())
}
