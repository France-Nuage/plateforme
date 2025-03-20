use controlplane::server::{Server, ServerConfig};
use proto::instance_client::InstanceClient;
use proto::{InstanceStatusRequest, InstanceStatusResponse};
use proxmox::mock::{MockServer, WithVMStatusReadMock};

#[tokio::test]
async fn test_the_server_starts() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::new(ServerConfig::default()).await?;
    let shutdown_tx = server.serve_with_shutdown().await?;
    shutdown_tx.send(()).ok();
    Ok(())
}

#[tokio::test]
async fn test_the_status_procedure_works() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new().await.with_vm_status_read();
    let config = ServerConfig::new(mock.url());
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = InstanceClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let request = tonic::Request::new(InstanceStatusRequest {
        id: String::from("666"),
    });
    let response = client.status(request).await;

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(
        response.unwrap().into_inner(),
        InstanceStatusResponse {
            status: String::from("Running")
        }
    );

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
