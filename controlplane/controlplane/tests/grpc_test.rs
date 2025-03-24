use controlplane::server::{Server, ServerConfig};
use proto::v0::hypervisor_client::HypervisorClient;
use proxmox::mock::{
    MockServer, WithClusterResourceList, WithVMStatusStartMock, WithVMStatusStopMock,
};

#[tokio::test]
async fn test_the_server_starts() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::new(ServerConfig::default()).await?;
    let shutdown_tx = server.serve_with_shutdown().await?;
    shutdown_tx.send(()).ok();
    Ok(())
}

#[tokio::test]
async fn test_the_list_instances_procedure_works() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new().await.with_cluster_resource_list();
    let config = ServerConfig::new(mock.url());
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = HypervisorClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client
        .list_instances(proto::v0::ListInstancesRequest::default())
        .await;

    // Assert the result
    assert!(response.is_ok());

    // Get the inner response
    let inner_response = response.unwrap().into_inner();

    // Check that we have a Success result
    match inner_response.result {
        Some(proto::v0::list_instances_response::Result::Success(instance_list)) => {
            assert_eq!(instance_list.instances.len(), 1);
        }
        Some(proto::v0::list_instances_response::Result::Problem(problem)) => {
            panic!("Expected Success result, got Problem: {}", problem.title);
        }
        None => {
            panic!("Expected Some result, got None");
        }
    }

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}

#[tokio::test]
async fn test_the_start_instance_procedure_works() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new().await.with_vm_status_start();
    let config = ServerConfig::new(mock.url());
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = HypervisorClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client
        .start_instance(proto::v0::StartInstanceRequest {
            id: String::from("100"),
        })
        .await;

    // Assert the result
    assert!(response.is_ok());

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}

#[tokio::test]
async fn test_the_stop_instance_procedure_works() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new().await.with_vm_status_stop();
    let config = ServerConfig::new(mock.url());
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = HypervisorClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client
        .stop_instance(proto::v0::StopInstanceRequest {
            id: String::from("100"),
        })
        .await;

    // Assert the result
    assert!(response.is_ok());

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
