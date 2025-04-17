use std::sync::Arc;

use hypervisor_connector_proxmox::mock::{
    MockServer, WithClusterNextId, WithClusterResourceList, WithVMCreateMock,
    WithVMStatusStartMock, WithVMStatusStopMock,
};
use hypervisors::v1::{
    ListHypervisorsRequest, RegisterHypervisorRequest, hypervisors_client::HypervisorsClient,
};
use instances::v1::{
    CreateInstanceRequest, CreateInstanceResponse, ListInstancesRequest, StartInstanceRequest,
    StopInstanceRequest, instances_client::InstancesClient,
};
use sea_orm::{MockDatabase, MockExecResult};
use server::{Server, ServerConfig};

#[tokio::test]
async fn test_the_server_starts() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::new(ServerConfig::default()).await?;
    let shutdown_tx = server.serve_with_shutdown().await?;
    shutdown_tx.send(()).ok();
    Ok(())
}

#[tokio::test]
async fn test_the_create_instance_procedure_works() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let connection = MockDatabase::new(sea_orm::DatabaseBackend::Postgres).into_connection();
    let mock = MockServer::new()
        .await
        .with_cluster_next_id()
        .with_vm_create();
    let config = ServerConfig::new(mock.url(), Arc::new(connection));
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = InstancesClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client
        .create_instance(CreateInstanceRequest {
            image: String::from("debian.qcow2"),
            cpu_cores: 1,
            memory_bytes: 536870912,
            name: String::from("acme-mgs"),
            snippet: String::from("acme-snippet.yaml"),
        })
        .await;

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(
        response.unwrap().into_inner(),
        CreateInstanceResponse {
            id: String::from("100")
        }
    );

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}

#[tokio::test]
async fn test_the_list_instances_procedure_works() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let connection = MockDatabase::new(sea_orm::DatabaseBackend::Postgres).into_connection();
    let mock = MockServer::new().await.with_cluster_resource_list();
    let config = ServerConfig::new(mock.url(), Arc::new(connection));
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = InstancesClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client.list_instances(ListInstancesRequest::default()).await;

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(response.unwrap().into_inner().instances.len(), 1);

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}

#[tokio::test]
async fn test_the_start_instance_procedure_works() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let connection = MockDatabase::new(sea_orm::DatabaseBackend::Postgres).into_connection();
    let mock = MockServer::new().await.with_vm_status_start();
    let config = ServerConfig::new(mock.url(), Arc::new(connection));
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = InstancesClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client
        .start_instance(StartInstanceRequest {
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
    let connection = MockDatabase::new(sea_orm::DatabaseBackend::Postgres).into_connection();
    let mock = MockServer::new().await.with_vm_status_stop();
    let config = ServerConfig::new(mock.url(), Arc::new(connection));
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = InstancesClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client
        .stop_instance(StopInstanceRequest {
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
async fn test_the_register_hypervisor_procedure_works() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let connection = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results([vec![hypervisors::model::Model::default()]])
        .into_connection();
    let mock = MockServer::new().await;
    let config = ServerConfig::new(mock.url(), Arc::new(connection));
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = HypervisorsClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let result = client
        .register_hypervisor(RegisterHypervisorRequest::default())
        .await;

    // Assert the result
    assert!(result.is_ok());

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}

#[tokio::test]
async fn test_the_list_hypervisors_procedure_works() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let connection = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([vec![hypervisors::model::Model::default()]])
        .into_connection();
    let mock = MockServer::new().await;
    let config = ServerConfig::new(mock.url(), Arc::new(connection));
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = HypervisorsClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let result = client
        .list_hypervisors(ListHypervisorsRequest::default())
        .await;

    // Assert the result
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().into_inner().hypervisors,
        vec![hypervisors::v1::Hypervisor::default()]
    );

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
