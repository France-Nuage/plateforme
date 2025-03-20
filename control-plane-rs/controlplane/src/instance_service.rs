use hypervisor::{Cluster, Instance as HypervisorInstance, Node};
use proto::instance_server::Instance;
use proto::{InstanceStatusRequest, InstanceStatusResponse};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct InstanceService {
    api_url: String,
    client: reqwest::Client,
}

impl InstanceService {
    pub fn new(api_url: String, client: reqwest::Client) -> Self {
        Self { api_url, client }
    }
}

#[tonic::async_trait]
impl Instance for InstanceService {
    async fn status(
        &self,
        request: Request<InstanceStatusRequest>,
    ) -> Result<Response<InstanceStatusResponse>, Status> {
        let request = request.into_inner();
        println!("Request: {:?}", request);
        let cluster = proxmox::Cluster::new(&self.api_url, &self.client);
        let result = cluster
            .node("pve-node1")
            .instance(&request.id)
            .status()
            .await;

        let status = result.unwrap();

        Ok(Response::new(InstanceStatusResponse {
            status: status.into(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proxmox::mock::{MockServer, WithVMStatusReadMock};

    #[tokio::test]
    async fn test_status_works() {
        // Arrange a service and a request for the status procedure
        let server = MockServer::new().await.with_vm_status_read();
        let service = InstanceService::new(server.url(), reqwest::Client::new());
        let request = Request::new(InstanceStatusRequest {
            id: String::from("666"),
        });

        // Act the call to the status procedure
        let result = service.status(request).await;

        // Assert the procedure result
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().into_inner().status,
            proto::InstanceStatus::Running as i32
        );
    }
}
