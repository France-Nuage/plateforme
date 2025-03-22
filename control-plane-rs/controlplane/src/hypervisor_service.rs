use hypervisor::{Cluster, Instance, Node};
use proto::v0::hypervisor_server::Hypervisor;

#[derive(Debug, Default)]
pub struct HypervisorService {
    api_url: String,
    client: reqwest::Client,
}

impl HypervisorService {
    pub fn new(api_url: String, client: reqwest::Client) -> Self {
        Self { api_url, client }
    }
}

#[tonic::async_trait]
impl Hypervisor for HypervisorService {
    /// TODO: docgen with prost
    async fn list_instances(
        &self,
        _: tonic::Request<()>,
    ) -> std::result::Result<tonic::Response<proto::v0::ListInstancesResponse>, tonic::Status> {
        Ok(tonic::Response::new(proto::v0::ListInstancesResponse {
            instances: proxmox::Cluster::new(&self.api_url, &self.client)
                .instances()
                .await
                .unwrap(),
        }))
    }

    /// TODO: docgen with prost
    async fn start_instance(
        &self,
        request: tonic::Request<proto::v0::StartInstanceRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        proxmox::Cluster::new(&self.api_url, &self.client)
            .node("pve-node1")
            .instance(&request.into_inner().id)
            .start()
            .await
            .unwrap();

        Ok(tonic::Response::new(()))
    }

    /// TODO: docgen with prost
    async fn stop_instance(
        &self,
        request: tonic::Request<proto::v0::StopInstanceRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        proxmox::Cluster::new(&self.api_url, &self.client)
            .node("pve-node1")
            .instance(&request.into_inner().id)
            .stop()
            .await
            .unwrap();
        Ok(tonic::Response::new(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proxmox::mock::{
        MockServer, WithClusterResourceList, WithVMStatusStartMock, WithVMStatusStopMock,
    };

    #[tokio::test]
    async fn test_list_instances_works() {
        // Arrange a service and a request for the list_instances procedure
        let server = MockServer::new().await.with_cluster_resource_list();
        let service = HypervisorService::new(server.url(), reqwest::Client::new());

        // Act the call to the list_instances procedure
        let result = service.list_instances(tonic::Request::new(())).await;

        // Assert the procedure result
        assert!(result.is_ok());
        assert_eq!(result.unwrap().into_inner().instances.len(), 1);
    }

    #[tokio::test]
    async fn test_start_instance_works() {
        // Arrange a service and a request for the start_instance procedure
        let server = MockServer::new().await.with_vm_status_start();
        let service = HypervisorService::new(server.url(), reqwest::Client::new());

        // Act the call to the start_instance procedure
        let request = tonic::Request::new(proto::v0::StartInstanceRequest {
            id: String::from("100"),
        });
        let result = service.start_instance(request).await;

        // Assert the procedure result
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stop_instance_works() {
        // Arrange a service and a request for the start_instance procedure
        let server = MockServer::new().await.with_vm_status_stop();
        let service = HypervisorService::new(server.url(), reqwest::Client::new());

        // Act the call to the start_instance procedure
        let request = tonic::Request::new(proto::v0::StopInstanceRequest {
            id: String::from("100"),
        });
        let result = service.stop_instance(request).await;

        // Assert the procedure result
        assert!(result.is_ok());
    }
}
