use super::proto::instance_server::Instance;
use super::proto::{InstanceStatusRequest, InstanceStatusResponse};
use hypervisor::{Cluster, Instance as HypervisorInstance, Node};
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
        println!("result: {:?}", result);
        Ok(Response::new(InstanceStatusResponse {
            status: String::from("OK"),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_status_works() {
        // Arrange a service and a request for the status procedure
        let service = InstanceService::default();
        let request = Request::new(InstanceStatusRequest {
            id: String::from("666"),
        });

        // Act the call to the status procedure
        let result = service.status(request).await;

        // Assert the procedure result
        assert!(result.is_ok());
        assert_eq!(result.unwrap().into_inner().status, String::from("OK"));
    }
}
