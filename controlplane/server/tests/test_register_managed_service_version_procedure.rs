use crate::common::{Api, IntoCi, seed_managed_service, seed_managed_service_version};
use frn_rpc::v1::managed::RegisterVersionRequest;
use tonic::{Code, Request};

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_register_version_creates_version(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;

    let request = Request::new(RegisterVersionRequest {
        service_slug: "vaultwarden".to_owned(),
        chart_version: "1.0.0".to_owned(),
        app_version: Some("1.30.0".to_owned()),
        oci_reference: "oci://registry.example.com/vaultwarden:1.0.0".to_owned(),
        configurable_values_schema: Some(r#"{"type": "object"}"#.to_owned()),
        ui_schema: None,
    })
    .into_ci();

    let response = api.managed.services.register_version(request).await;

    let version = response
        .expect("register_version must succeed")
        .into_inner()
        .version
        .unwrap();
    assert_eq!(version.chart_version, "1.0.0");
    assert_eq!(version.app_version, Some("1.30.0".to_owned()));
    assert_eq!(
        version.oci_reference,
        "oci://registry.example.com/vaultwarden:1.0.0"
    );
    assert!(version.configurable_values_schema.is_some());
    assert!(version.ui_schema.is_none());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_register_version_persists_ui_schema(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;

    let ui_schema_json = r#"{"ui:order":["domain","smtp"],"domain":{"ui:title":"Domaine"}}"#;

    let request = Request::new(RegisterVersionRequest {
        service_slug: "vaultwarden".to_owned(),
        chart_version: "1.0.0".to_owned(),
        app_version: None,
        oci_reference: "oci://registry.example.com/vaultwarden:1.0.0".to_owned(),
        configurable_values_schema: Some(r#"{"type":"object"}"#.to_owned()),
        ui_schema: Some(ui_schema_json.to_owned()),
    })
    .into_ci();

    let response = api.managed.services.register_version(request).await;

    assert!(response.is_ok());
    let version = response.unwrap().into_inner().version.unwrap();
    let returned: serde_json::Value = serde_json::from_str(
        version
            .ui_schema
            .as_deref()
            .expect("ui_schema must be persisted"),
    )?;
    let expected: serde_json::Value = serde_json::from_str(ui_schema_json)?;
    assert_eq!(returned, expected);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_register_version_rejects_invalid_ui_schema_json(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;

    let request = Request::new(RegisterVersionRequest {
        service_slug: "vaultwarden".to_owned(),
        chart_version: "1.0.0".to_owned(),
        app_version: None,
        oci_reference: "oci://registry.example.com/vaultwarden:1.0.0".to_owned(),
        configurable_values_schema: None,
        ui_schema: Some("not valid json".to_owned()),
    })
    .into_ci();

    let response = api.managed.services.register_version(request).await;

    assert!(response.is_err());
    let status = response.unwrap_err();
    assert_eq!(status.code(), Code::InvalidArgument);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_register_version_rejects_unauthenticated(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;

    let request = Request::new(RegisterVersionRequest {
        service_slug: "vaultwarden".to_owned(),
        chart_version: "1.0.0".to_owned(),
        app_version: None,
        oci_reference: "oci://registry.example.com/vaultwarden:1.0.0".to_owned(),
        configurable_values_schema: None,
        ui_schema: None,
    });

    let response = api.managed.services.register_version(request).await;

    assert!(response.is_err());
    let status = response.unwrap_err();
    assert_eq!(status.code(), Code::Unauthenticated);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_register_version_rejects_wrong_token(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;

    let mut request = Request::new(RegisterVersionRequest {
        service_slug: "vaultwarden".to_owned(),
        chart_version: "1.0.0".to_owned(),
        app_version: None,
        oci_reference: "oci://registry.example.com/vaultwarden:1.0.0".to_owned(),
        configurable_values_schema: None,
        ui_schema: None,
    });
    request
        .metadata_mut()
        .insert("authorization", "Bearer wrong-token".parse()?);

    let response = api.managed.services.register_version(request).await;

    assert!(response.is_err());
    let status = response.unwrap_err();
    assert_eq!(status.code(), Code::Unauthenticated);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_register_version_rejects_duplicate(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let service_id = seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;

    seed_managed_service_version(
        &pool,
        service_id,
        "1.0.0",
        None,
        "oci://registry.example.com/vaultwarden:1.0.0",
    )
    .await;

    let request = Request::new(RegisterVersionRequest {
        service_slug: "vaultwarden".to_owned(),
        chart_version: "1.0.0".to_owned(),
        app_version: None,
        oci_reference: "oci://registry.example.com/vaultwarden:1.0.0-new".to_owned(),
        configurable_values_schema: None,
        ui_schema: None,
    })
    .into_ci();

    let response = api.managed.services.register_version(request).await;

    assert!(response.is_err());
    let status = response.unwrap_err();
    assert_eq!(status.code(), Code::AlreadyExists);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_register_version_rejects_unknown_service(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let request = Request::new(RegisterVersionRequest {
        service_slug: "nonexistent".to_owned(),
        chart_version: "1.0.0".to_owned(),
        app_version: None,
        oci_reference: "oci://registry.example.com/nonexistent:1.0.0".to_owned(),
        configurable_values_schema: None,
        ui_schema: None,
    })
    .into_ci();

    let response = api.managed.services.register_version(request).await;

    assert!(response.is_err());
    let status = response.unwrap_err();
    assert_eq!(status.code(), Code::NotFound);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_register_version_rejects_invalid_schema_json(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;

    let request = Request::new(RegisterVersionRequest {
        service_slug: "vaultwarden".to_owned(),
        chart_version: "1.0.0".to_owned(),
        app_version: None,
        oci_reference: "oci://registry.example.com/vaultwarden:1.0.0".to_owned(),
        configurable_values_schema: Some("not valid json".to_owned()),
        ui_schema: None,
    })
    .into_ci();

    let response = api.managed.services.register_version(request).await;

    assert!(response.is_err());
    let status = response.unwrap_err();
    assert_eq!(status.code(), Code::InvalidArgument);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_register_version_rejects_invalid_oci_reference(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;

    let request = Request::new(RegisterVersionRequest {
        service_slug: "vaultwarden".to_owned(),
        chart_version: "1.0.0".to_owned(),
        app_version: None,
        oci_reference: "https://registry.example.com/vaultwarden:1.0.0".to_owned(),
        configurable_values_schema: None,
        ui_schema: None,
    })
    .into_ci();

    let response = api.managed.services.register_version(request).await;

    assert!(response.is_err());
    let status = response.unwrap_err();
    assert_eq!(status.code(), Code::InvalidArgument);

    Ok(())
}
