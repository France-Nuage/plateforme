use hypervisor_connector::InstanceService;
use tonic::{Request, Response, Status};

use crate::{
    problem::Problem,
    v1::{
        ListInstancesRequest, ListInstancesResponse, StartInstanceRequest, StartInstanceResponse,
        StopInstanceRequest, StopInstanceResponse, instances_server::Instances,
    },
};

pub struct InstancesRpcService {
    api_url: String,
    client: reqwest::Client,
}

#[tonic::async_trait]
impl Instances for InstancesRpcService {
    #[doc = " ListInstances retrieves information about all available instances."]
    #[doc = " Returns a collection of instance details including their current status and resource usage."]
    async fn list_instances(
        &self,
        _: Request<ListInstancesRequest>,
    ) -> Result<Response<ListInstancesResponse>, Status> {
        let result =
            hypervisor_connector_resolver::resolve(self.api_url.clone(), self.client.clone())
                .list()
                .await;

        match result {
            Ok(instances) => Ok(Response::new(ListInstancesResponse {
                instances: instances.into_iter().map(Into::into).collect(),
            })),
            Err(_) => panic!(""),
        }
    }

    #[doc = " StartInstance initiates a specific instance identified by its unique ID."]
    #[doc = " Returns a response indicating success or a ProblemDetails on failure."]
    async fn start_instance(
        &self,
        _: Request<StartInstanceRequest>,
    ) -> Result<Response<StartInstanceResponse>, Status> {
        let result =
            hypervisor_connector_resolver::resolve(self.api_url.clone(), self.client.clone())
                .start()
                .await;

        match result {
            Ok(()) => Ok(Response::new(StartInstanceResponse {})),
            Err(error) => Err(Problem::from(error).into()),
        }
        // Ok(Response::new(result.into()))
    }

    #[doc = " StopInstance halts a specific instance identified by its unique ID."]
    #[doc = " Returns a response indicating success or a ProblemDetails on failure."]
    async fn stop_instance(
        &self,
        _: Request<StopInstanceRequest>,
    ) -> Result<Response<StopInstanceResponse>, Status> {
        let result =
            hypervisor_connector_resolver::resolve(self.api_url.clone(), self.client.clone())
                .stop()
                .await;

        match result {
            Ok(()) => Ok(Response::new(StopInstanceResponse {})),
            Err(_) => panic!(""),
        }
    }
}

impl InstancesRpcService {
    pub fn new(api_url: String, client: reqwest::Client) -> Self {
        Self { api_url, client }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v1::{ListInstancesRequest, StartInstanceRequest};
    use hypervisor_connector_proxmox::mock::{
        MockServer, WithClusterResourceList, WithVMStatusStartMock, WithVMStatusStopMock,
    };
    use tonic::Request;

    #[tokio::test]
    async fn test_list_instances_works() {
        // Arrange a service and a request for the list_instances procedure
        let server = MockServer::new().await.with_cluster_resource_list();
        let service = InstancesRpcService::new(server.url(), reqwest::Client::new());

        // Act the call to the list_instances procedure
        let result = service
            .list_instances(Request::new(ListInstancesRequest::default()))
            .await;

        // Assert the procedure result
        assert!(result.is_ok());
        let response = result.unwrap().into_inner();
        // Check that we have a Success result
        assert_eq!(response.instances.len(), 1);
    }

    #[tokio::test]
    async fn test_start_instance_works() {
        // Arrange a service and a request for the start_instance procedure
        let server = MockServer::new().await.with_vm_status_start();
        let service = InstancesRpcService::new(server.url(), reqwest::Client::new());

        // Act the call to the start_instance procedure
        let request = Request::new(StartInstanceRequest {
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
        let service = InstancesRpcService::new(server.url(), reqwest::Client::new());

        // Act the call to the start_instance procedure
        let request = Request::new(crate::v1::StopInstanceRequest {
            id: String::from("100"),
        });
        let result = service.stop_instance(request).await;

        // Assert the procedure result
        assert!(result.is_ok());
    }
}
