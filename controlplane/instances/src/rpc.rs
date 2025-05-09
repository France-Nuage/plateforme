use crate::{
    problem::Problem,
    service::InstancesService,
    v1::{
        CloneInstanceRequest, CreateInstanceRequest, CreateInstanceResponse, DeleteInstanceRequest,
        DeleteInstanceResponse, Instance, ListInstancesRequest, ListInstancesResponse,
        StartInstanceRequest, StartInstanceResponse, StopInstanceRequest, StopInstanceResponse,
        instances_server::Instances,
    },
};
use sqlx::PgPool;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct InstancesRpcService {
    service: InstancesService,
}

#[tonic::async_trait]
impl Instances for InstancesRpcService {
    #[doc = " CreateInstance provisions a new instance based on the specified configuration."]
    #[doc = " Returns details of the newly created instance or a ProblemDetails on failure."]
    async fn create_instance(
        &self,
        request: tonic::Request<CreateInstanceRequest>,
    ) -> Result<tonic::Response<CreateInstanceResponse>, tonic::Status> {
        let instance = self.service.create(request.into_inner().into()).await?;

        Ok(Response::new(CreateInstanceResponse {
            instance: Some(instance.into()),
        }))
    }

    #[doc = " DeleteInstance deletes a given instance."]
    #[doc = " Returns an empty message or a ProblemDetails on failure."]
    async fn delete_instance(
        &self,
        request: tonic::Request<DeleteInstanceRequest>,
    ) -> std::result::Result<tonic::Response<DeleteInstanceResponse>, tonic::Status> {
        let id = request.into_inner().id;
        let id = Uuid::parse_str(&id).map_err(|_| Problem::MalformedInstanceId(id))?;
        self.service.delete(id).await?;
        Ok(Response::new(DeleteInstanceResponse {}))
    }

    #[doc = " CloneInstance provisions a new instance based on a given existing instance."]
    #[doc = " Returns the cloned instance."]
    async fn clone_instance(
        &self,
        request: tonic::Request<CloneInstanceRequest>,
    ) -> std::result::Result<tonic::Response<Instance>, tonic::Status> {
        let id = request.into_inner().id;
        let id = Uuid::parse_str(&id).map_err(|_| Problem::MalformedInstanceId(id))?;
        let instance = self.service.clone(id).await?;
        Ok(Response::new(instance.into()))
    }

    #[doc = " ListInstances retrieves information about all available instances."]
    #[doc = " Returns a collection of instance details including their current status and resource usage."]
    async fn list_instances(
        &self,
        _: Request<ListInstancesRequest>,
    ) -> Result<Response<ListInstancesResponse>, Status> {
        let instances = self.service.list().await?;

        Ok(Response::new(ListInstancesResponse {
            instances: instances.into_iter().map(Into::into).collect(),
        }))
    }

    #[doc = " StartInstance initiates a specific instance identified by its unique ID."]
    #[doc = " Returns a response indicating success or a ProblemDetails on failure."]
    async fn start_instance(
        &self,
        request: Request<StartInstanceRequest>,
    ) -> Result<Response<StartInstanceResponse>, Status> {
        let id = request.into_inner().id;
        let id = Uuid::parse_str(&id).map_err(|_| Problem::MalformedInstanceId(id))?;
        self.service.start(id).await?;
        Ok(Response::new(StartInstanceResponse {}))
    }

    #[doc = " StopInstance halts a specific instance identified by its unique ID."]
    #[doc = " Returns a response indicating success or a ProblemDetails on failure."]
    async fn stop_instance(
        &self,
        request: Request<StopInstanceRequest>,
    ) -> Result<Response<StopInstanceResponse>, Status> {
        let id = request.into_inner().id;
        let id = Uuid::parse_str(&id).map_err(|_| Problem::MalformedInstanceId(id))?;
        self.service.stop(id).await?;
        Ok(Response::new(StopInstanceResponse {}))
    }
}

impl InstancesRpcService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            service: InstancesService::new(pool),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::Instance,
        v1::{ListInstancesRequest, StartInstanceRequest},
    };
    use hypervisor_connector_proxmox::mock::{
        MockServer, WithClusterNextId, WithClusterResourceList, WithTaskStatusReadMock,
        WithVMCloneMock, WithVMDeleteMock, WithVMStatusStartMock, WithVMStatusStopMock,
    };
    use hypervisors::Hypervisor;

    #[sqlx::test(migrations = "../migrations")]
    async fn test_list_instances_works(pool: sqlx::PgPool) {
        // Arrange a service and a request for the list_instances procedure
        let server = MockServer::new().await.with_cluster_resource_list();
        let hypervisor = Hypervisor {
            url: server.url(),
            ..Default::default()
        };
        hypervisors::repository::create(&pool, &hypervisor)
            .await
            .expect("could not create hypervisor");
        let service = InstancesRpcService::new(pool);

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

    #[sqlx::test(migrations = "../migrations")]
    async fn test_clone_instance_works(pool: sqlx::PgPool) {
        // Arrange a service and a request for the start_instance procedure
        let server = MockServer::new()
            .await
            .with_cluster_next_id()
            .with_cluster_resource_list()
            .with_vm_clone()
            .with_task_status_read();
        let hypervisor = Hypervisor {
            url: server.url(),
            ..Default::default()
        };
        hypervisors::repository::create(&pool, &hypervisor)
            .await
            .expect("could not create hypervisor");

        let instance = Instance {
            hypervisor_id: hypervisor.id,
            distant_id: String::from("100"),
            ..Default::default()
        };
        crate::repository::create(&pool, &instance)
            .await
            .expect("could not create instance");
        let service = InstancesRpcService::new(pool);

        // Act the call to the start_instance procedure
        let request = Request::new(CloneInstanceRequest {
            id: instance.id.to_string(),
        });
        let result = service.clone_instance(request).await;

        // Assert the procedure result
        assert!(result.is_ok());
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn test_delete_instance_works(pool: sqlx::PgPool) {
        // Arrange a service and a request for the start_instance procedure
        let server = MockServer::new()
            .await
            .with_cluster_resource_list()
            .with_vm_delete()
            .with_task_status_read();
        let hypervisor = Hypervisor {
            url: server.url(),
            ..Default::default()
        };
        hypervisors::repository::create(&pool, &hypervisor)
            .await
            .expect("could not create hypervisor");

        let instance = Instance {
            hypervisor_id: hypervisor.id,
            distant_id: String::from("100"),
            ..Default::default()
        };
        crate::repository::create(&pool, &instance)
            .await
            .expect("could not create instance");
        let service = InstancesRpcService::new(pool);

        // Act the call to the start_instance procedure
        let request = Request::new(DeleteInstanceRequest {
            id: instance.id.to_string(),
        });
        let result = service.delete_instance(request).await;

        // Assert the procedure result
        assert!(result.is_ok());
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn test_start_instance_works(pool: sqlx::PgPool) {
        // Arrange a service and a request for the start_instance procedure
        let server = MockServer::new()
            .await
            .with_vm_status_start()
            .with_cluster_resource_list();
        let hypervisor = Hypervisor {
            url: server.url(),
            ..Default::default()
        };
        hypervisors::repository::create(&pool, &hypervisor)
            .await
            .expect("could not create hypervisor");

        let instance = Instance {
            hypervisor_id: hypervisor.id,
            distant_id: String::from("100"),
            ..Default::default()
        };
        crate::repository::create(&pool, &instance)
            .await
            .expect("could not create instance");
        let service = InstancesRpcService::new(pool);

        // Act the call to the start_instance procedure
        let request = Request::new(StartInstanceRequest {
            id: instance.id.to_string(),
        });
        let result = service.start_instance(request).await;

        // Assert the procedure result
        assert!(result.is_ok());
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn test_stop_instance_works(pool: sqlx::PgPool) {
        // Arrange a service and a request for the start_instance procedure
        let server = MockServer::new()
            .await
            .with_vm_status_stop()
            .with_cluster_resource_list();
        let hypervisor = Hypervisor {
            url: server.url(),
            ..Default::default()
        };
        hypervisors::repository::create(&pool, &hypervisor)
            .await
            .expect("could not create hypervisor");

        let instance = Instance {
            hypervisor_id: hypervisor.id,
            distant_id: String::from("100"),
            ..Default::default()
        };
        crate::repository::create(&pool, &instance)
            .await
            .expect("could not create instance");
        let service = InstancesRpcService::new(pool);

        // Act the call to the start_instance procedure
        let request = Request::new(crate::v1::StopInstanceRequest {
            id: instance.id.to_string(),
        });
        let result = service.stop_instance(request).await;
        println!("result: {:?}", &result);

        // Assert the procedure result
        assert!(result.is_ok());
    }
}
