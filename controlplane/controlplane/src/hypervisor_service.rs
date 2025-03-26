use hypervisor::{Cluster, Instance, Node};
use proto::v0::{
    InstanceList, ListInstancesRequest, ListInstancesResponse, StartInstanceRequest,
    StartInstanceResponse, StopInstanceRequest, StopInstanceResponse,
    hypervisor_server::Hypervisor,
};
use std::result::Result;
use tonic::{Request, Response};

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
        _: Request<ListInstancesRequest>,
    ) -> Result<Response<ListInstancesResponse>, tonic::Status> {
        Ok(Response::new(ListInstancesResponse {
            result: Some(proto::v0::list_instances_response::Result::Success(
                InstanceList {
                    instances: proxmox::Cluster::new(&self.api_url, &self.client)
                        .instances()
                        .await
                        .unwrap(),
                },
            )),
        }))
    }

    /// TODO: docgen with prost
    async fn start_instance(
        &self,
        request: Request<StartInstanceRequest>,
    ) -> Result<Response<StartInstanceResponse>, tonic::Status> {
        proxmox::Cluster::new(&self.api_url, &self.client)
            .node("pve-node1")
            .instance(&request.into_inner().id)
            .start()
            .await
            .unwrap();

        Ok(Response::new(StartInstanceResponse {
            result: Some(proto::v0::start_instance_response::Result::Success(())),
        }))
    }

    /// TODO: docgen with prost
    async fn stop_instance(
        &self,
        request: Request<StopInstanceRequest>,
    ) -> Result<Response<StopInstanceResponse>, tonic::Status> {
        proxmox::Cluster::new(&self.api_url, &self.client)
            .node("pve-node1")
            .instance(&request.into_inner().id)
            .stop()
            .await
            .unwrap();
        Ok(Response::new(StopInstanceResponse {
            result: Some(proto::v0::stop_instance_response::Result::Success(())),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proto::v0::ListInstancesRequest;
    use proxmox::mock::{
        MockServer, WithClusterResourceList, WithVMStatusStartMock, WithVMStatusStopMock,
    };
    use tonic::Request;

    #[tokio::test]
    async fn test_list_instances_works() {
        // Arrange a service and a request for the list_instances procedure
        let server = MockServer::new().await.with_cluster_resource_list();
        let service = HypervisorService::new(server.url(), reqwest::Client::new());

        // Act the call to the list_instances procedure
        let result = service
            .list_instances(Request::new(ListInstancesRequest::default()))
            .await;

        // Assert the procedure result
        assert!(result.is_ok());
        let response = result.unwrap().into_inner();
        // Check that we have a Success result
        assert!(matches!(
            response.result,
            Some(proto::v0::list_instances_response::Result::Success(_))
        ));

        // If we need to access the instances
        if let Some(proto::v0::list_instances_response::Result::Success(instance_list)) =
            response.result
        {
            assert_eq!(instance_list.instances.len(), 1);
        } else {
            panic!("Expected Success result variant");
        }
    }

    #[tokio::test]
    async fn test_start_instance_works() {
        // Arrange a service and a request for the start_instance procedure
        let server = MockServer::new().await.with_vm_status_start();
        let service = HypervisorService::new(server.url(), reqwest::Client::new());

        // Act the call to the start_instance procedure
        let request = Request::new(proto::v0::StartInstanceRequest {
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
        let request = Request::new(proto::v0::StopInstanceRequest {
            id: String::from("100"),
        });
        let result = service.stop_instance(request).await;

        // Assert the procedure result
        assert!(result.is_ok());
    }
}
