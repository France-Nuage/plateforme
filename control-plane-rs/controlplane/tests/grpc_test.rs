use controlplane::proto::instance_client::InstanceClient;
use controlplane::proto::{InstanceStatusRequest, InstanceStatusResponse};
use controlplane::server::{Server, ServerConfig};

#[tokio::test]
async fn test_the_server_starts() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::new(ServerConfig::default()).await?;
    let shutdown_tx = server.serve_with_shutdown().await?;
    shutdown_tx.send(()).ok();
    Ok(())
}

#[tokio::test]
async fn test_the_status_procedure_works() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::new(ServerConfig::default()).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;

    let mut client = InstanceClient::connect(format!("http://{}", addr)).await?;

    let request = tonic::Request::new(InstanceStatusRequest {
        id: String::from("666"),
    });

    let response = client.status(request).await;
    assert!(response.is_ok());
    assert_eq!(
        response.unwrap().into_inner(),
        InstanceStatusResponse {
            status: String::from("OK")
        }
    );

    shutdown_tx.send(()).ok();
    Ok(())
}
