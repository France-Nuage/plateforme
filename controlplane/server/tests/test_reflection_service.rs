use auth::mock::WithWellKnown;
use mock_server::MockServer;
use server::Config;
use tonic::transport::Channel;
use tonic_reflection::pb::v1::ServerReflectionRequest;
use tonic_reflection::pb::v1::server_reflection_client::ServerReflectionClient;
use tonic_reflection::pb::v1::server_reflection_request::MessageRequest;
use tonic_reflection::pb::v1alpha::ServerReflectionRequest as ServerReflectionRequestV1Alpha;
use tonic_reflection::pb::v1alpha::server_reflection_client::ServerReflectionClient as ServerReflectionClientV1Alpha;
use tonic_reflection::pb::v1alpha::server_reflection_request::MessageRequest as MessageRequestV1Alpha;

/// Tests that the gRPC reflection service is properly registered and responds
/// to reflection queries.
///
/// This test verifies that clients can discover available services and their
/// schemas at runtime without requiring access to .proto files. This is essential
/// for tools like grpcurl and grpcui to work properly.
#[sqlx::test(migrations = "../migrations")]
async fn test_reflection_service_lists_services(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Start the gRPC server with reflection enabled
    let mock = MockServer::new().await.with_well_known();
    let config = Config::test(&pool, &mock).await?;
    let server_url = format!("http://{}", config.addr);
    let shutdown_tx = server::serve(config).await?;

    // Create a reflection client
    let channel = Channel::from_shared(server_url)?.connect().await?;
    let mut client = ServerReflectionClient::new(channel);

    // Act: Request the list of all services
    let request = tonic::Request::new(tokio_stream::iter(vec![ServerReflectionRequest {
        host: String::new(),
        message_request: Some(MessageRequest::ListServices(String::new())),
    }]));

    let mut response_stream = client.server_reflection_info(request).await?.into_inner();

    // Assert: Verify we get a response with service listings
    let response = response_stream.message().await?;

    assert!(response.is_some(), "Expected a reflection response");

    let response = response.unwrap();

    // Extract service names from the response
    if let Some(
        tonic_reflection::pb::v1::server_reflection_response::MessageResponse::ListServicesResponse(
            services,
        ),
    ) = response.message_response
    {
        let service_names: Vec<String> = services.service.iter().map(|s| s.name.clone()).collect();

        // Verify that our services are listed
        assert!(
            service_names
                .iter()
                .any(|name| name.contains("Hypervisors")),
            "Hypervisors service should be discoverable via reflection. Found: {:?}",
            service_names
        );

        assert!(
            service_names
                .iter()
                .any(|name| name.contains("Organizations")),
            "Organizations service should be discoverable via reflection. Found: {:?}",
            service_names
        );

        assert!(
            service_names.iter().any(|name| name.contains("Zones")),
            "Zones service should be discoverable via reflection. Found: {:?}",
            service_names
        );

        assert!(
            service_names.iter().any(|name| name.contains("Projects")),
            "Projects service should be discoverable via reflection. Found: {:?}",
            service_names
        );

        assert!(
            service_names.iter().any(|name| name.contains("Instances")),
            "Instances service should be discoverable via reflection. Found: {:?}",
            service_names
        );
    } else {
        panic!("Expected ListServicesResponse, got: {:?}", response);
    }

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}

/// Tests that the gRPC reflection service can provide file descriptors for
/// registered services.
///
/// This verifies that reflection clients can retrieve complete schema information
/// including message types, field definitions, and service method signatures.
#[sqlx::test(migrations = "../migrations")]
async fn test_reflection_service_provides_file_descriptors(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Start the gRPC server with reflection enabled
    let mock = MockServer::new().await.with_well_known();
    let config = Config::test(&pool, &mock).await?;
    let server_url = format!("http://{}", config.addr);
    let shutdown_tx = server::serve(config).await?;

    // Create a reflection client
    let channel = Channel::from_shared(server_url)?.connect().await?;
    let mut client = ServerReflectionClient::new(channel);

    // Act: Request file descriptor for a known service
    let request = tonic::Request::new(tokio_stream::iter(vec![ServerReflectionRequest {
        host: String::new(),
        message_request: Some(MessageRequest::FileContainingSymbol(
            "francenuage.fr.v1.compute.Hypervisors".to_string(),
        )),
    }]));

    let mut response_stream = client.server_reflection_info(request).await?.into_inner();

    // Assert: Verify we get a file descriptor response
    let response = response_stream.message().await?;

    assert!(response.is_some(), "Expected a reflection response");

    let response = response.unwrap();

    // Verify the response contains a file descriptor
    if let Some(tonic_reflection::pb::v1::server_reflection_response::MessageResponse::FileDescriptorResponse(
        descriptor_response,
    )) = response.message_response
    {
        assert!(
            !descriptor_response.file_descriptor_proto.is_empty(),
            "File descriptor should contain proto definitions"
        );
    } else {
        panic!(
            "Expected FileDescriptorResponse, got: {:?}",
            response.message_response
        );
    }

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}

/// Tests that the gRPC reflection v1alpha service is registered and working.
///
/// This test ensures compatibility with clients like Bruno that only support
/// the v1alpha reflection API. The v1alpha API is the legacy version but still
/// widely used by various gRPC clients.
#[sqlx::test(migrations = "../migrations")]
async fn test_reflection_v1alpha_service_lists_services(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Start the gRPC server with reflection enabled
    let mock = MockServer::new().await.with_well_known();
    let config = Config::test(&pool, &mock).await?;
    let server_url = format!("http://{}", config.addr);
    let shutdown_tx = server::serve(config).await?;

    // Create a v1alpha reflection client
    let channel = Channel::from_shared(server_url)?.connect().await?;
    let mut client = ServerReflectionClientV1Alpha::new(channel);

    // Act: Request the list of all services using v1alpha API
    let request = tonic::Request::new(tokio_stream::iter(vec![ServerReflectionRequestV1Alpha {
        host: String::new(),
        message_request: Some(MessageRequestV1Alpha::ListServices(String::new())),
    }]));

    let mut response_stream = client.server_reflection_info(request).await?.into_inner();

    // Assert: Verify we get a response with service listings
    let response = response_stream.message().await?;

    assert!(response.is_some(), "Expected a v1alpha reflection response");

    let response = response.unwrap();

    // Extract service names from the response
    if let Some(
        tonic_reflection::pb::v1alpha::server_reflection_response::MessageResponse::ListServicesResponse(
            services,
        ),
    ) = response.message_response
    {
        let service_names: Vec<String> = services
            .service
            .iter()
            .map(|s| s.name.clone())
            .collect();

        // Verify that our services are listed via v1alpha reflection
        assert!(
            service_names
                .iter()
                .any(|name| name.contains("Hypervisors")),
            "Hypervisors service should be discoverable via v1alpha reflection. Found: {:?}",
            service_names
        );

        assert!(
            service_names
                .iter()
                .any(|name| name.contains("Organizations")),
            "Organizations service should be discoverable via v1alpha reflection. Found: {:?}",
            service_names
        );

        assert!(
            service_names
                .iter()
                .any(|name| name.contains("Instances")),
            "Instances service should be discoverable via v1alpha reflection. Found: {:?}",
            service_names
        );

        // Verify the v1alpha reflection service itself is listed
        assert!(
            service_names
                .iter()
                .any(|name| name.contains("grpc.reflection.v1alpha.ServerReflection")),
            "v1alpha ServerReflection service should be listed. Found: {:?}",
            service_names
        );
    } else {
        panic!(
            "Expected v1alpha ListServicesResponse, got: {:?}",
            response
        );
    }

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
