use hypervisor_connector::InstanceService;
use tonic::{Request, Response, Status};

use crate::v1::{
    InstanceList, ListInstancesRequest, ListInstancesResponse, StartInstanceRequest,
    StartInstanceResponse, StopInstanceRequest, StopInstanceResponse, instances_server::Instances,
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
        let instances =
            hypervisor_connector_resolver::resolve(self.api_url.clone(), self.client.clone())
                .list()
                .await
                .expect("could not list instances");
        Ok(Response::new(ListInstancesResponse {
            result: Some(crate::v1::list_instances_response::Result::Success(
                InstanceList {
                    instances: instances.into_iter().map(Into::into).collect(),
                },
            )),
        }))
    }

    #[doc = " StartInstance initiates a specific instance identified by its unique ID."]
    #[doc = " Returns a response indicating success or a ProblemDetails on failure."]
    async fn start_instance(
        &self,
        _: Request<StartInstanceRequest>,
    ) -> Result<Response<StartInstanceResponse>, Status> {
        hypervisor_connector_resolver::resolve(self.api_url.clone(), self.client.clone())
            .start()
            .await
            .expect("could not start instance");

        Ok(Response::new(StartInstanceResponse {
            result: Some(crate::v1::start_instance_response::Result::Success(())),
        }))
    }

    #[doc = " StopInstance halts a specific instance identified by its unique ID."]
    #[doc = " Returns a response indicating success or a ProblemDetails on failure."]
    async fn stop_instance(
        &self,
        _: Request<StopInstanceRequest>,
    ) -> Result<Response<StopInstanceResponse>, Status> {
        hypervisor_connector_resolver::resolve(self.api_url.clone(), self.client.clone())
            .stop()
            .await
            .expect("could not start instance");

        Ok(Response::new(StopInstanceResponse {
            result: Some(crate::v1::stop_instance_response::Result::Success(())),
        }))
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
        assert!(matches!(
            response.result,
            Some(crate::v1::list_instances_response::Result::Success(_))
        ));

        // If we need to access the instances
        if let Some(crate::v1::list_instances_response::Result::Success(instance_list)) =
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
