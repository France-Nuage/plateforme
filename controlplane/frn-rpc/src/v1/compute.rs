use std::time::SystemTime;

use crate::error::Error;
use frn_core::authorization::Authorize;
use frn_core::compute::{
    HypervisorCreateRequest, Hypervisors as Service, InstanceCreateRequest, InstanceUpdateRequest,
};
use frn_core::identity::IAM;
use sqlx::{Pool, Postgres, types::Uuid};
use tonic::{Request, Response, Status};

tonic::include_proto!("francenuage.fr.v1.compute");

/// Converts a database Model to a Hypervisor API type.
impl From<frn_core::compute::Hypervisor> for Hypervisor {
    fn from(value: frn_core::compute::Hypervisor) -> Self {
        Hypervisor {
            id: value.id.to_string(),
            zone_id: value.zone_id.to_string(),
            organization_id: value.organization_id.to_string(),
            storage_name: value.storage_name,
            url: value.url,
        }
    }
}

pub struct Hypervisors<Auth: Authorize> {
    iam: IAM,
    _pool: Pool<Postgres>,
    service: Service<Auth>,
}

impl<Auth: Authorize> Hypervisors<Auth> {
    pub fn new(iam: IAM, pool: Pool<Postgres>, service: Service<Auth>) -> Self {
        Self {
            iam,
            _pool: pool,
            service,
        }
    }
}

#[tonic::async_trait]
impl<Auth: Authorize + 'static> hypervisors_server::Hypervisors for Hypervisors<Auth> {
    async fn detach(
        &self,
        request: Request<DetachHypervisorRequest>,
    ) -> Result<Response<DetachHypervisorResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();
        let id = inner
            .id
            .parse::<Uuid>()
            .map_err(|_| Error::MalformedId(inner.id))?;

        self.service.clone().delete(&principal, id).await?;

        Ok(Response::new(DetachHypervisorResponse {}))
    }

    async fn list(
        &self,
        request: Request<ListHypervisorsRequest>,
    ) -> Result<Response<ListHypervisorsResponse>, Status> {
        let principal = self.iam.principal(&request).await?;

        let hypervisors = self.service.clone().list(&principal).await?;

        Ok(Response::new(ListHypervisorsResponse {
            hypervisors: hypervisors.into_iter().map(Into::into).collect(),
        }))
    }

    async fn register(
        &self,
        request: Request<RegisterHypervisorRequest>,
    ) -> Result<Response<RegisterHypervisorResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let RegisterHypervisorRequest {
            authorization_token,
            organization_id,
            storage_name,
            url,
            zone_id,
        } = request.into_inner();

        let hypervisor = self
            .service
            .clone()
            .create(
                &principal,
                HypervisorCreateRequest {
                    authorization_token,
                    storage_name,
                    url,
                    organization_id: organization_id
                        .parse::<Uuid>()
                        .map_err(|_| Error::MalformedId(organization_id))?,
                    zone_id: zone_id
                        .parse::<Uuid>()
                        .map_err(|_| Error::MalformedId(zone_id))?,
                },
            )
            .await?;

        Ok(Response::new(RegisterHypervisorResponse {
            hypervisor: Some(hypervisor.into()),
        }))
    }
}

#[derive(Clone)]
pub struct Instances<A: Authorize> {
    iam: IAM,
    _pool: Pool<Postgres>,
    service: frn_core::compute::Instances<A>,
}

impl<A: Authorize> Instances<A> {
    pub fn new(iam: IAM, pool: Pool<Postgres>, service: frn_core::compute::Instances<A>) -> Self {
        Self {
            iam,
            _pool: pool,
            service,
        }
    }
}

impl From<frn_core::compute::Instance> for Instance {
    fn from(value: frn_core::compute::Instance) -> Self {
        Self {
            id: value.id.to_string(),
            status: InstanceStatus::from(value.status) as i32,
            max_cpu_cores: value.max_cpu_cores as u32,
            cpu_usage_percent: value.cpu_usage_percent as f32,
            max_memory_bytes: value.max_memory_bytes as u64,
            memory_usage_bytes: value.memory_usage_bytes as u64,
            max_disk_bytes: value.max_disk_bytes as u64,
            disk_usage_bytes: value.disk_usage_bytes as u64,
            name: value.name,
            ip_v4: value.ip_v4,
            hypervisor_id: value.hypervisor_id.to_string(),
            project_id: value.project_id.to_string(),
            zero_trust_network_id: value.zero_trust_network_id.map(Into::into),
            created_at: Some(SystemTime::from(value.created_at).into()),
            updated_at: Some(SystemTime::from(value.updated_at).into()),
        }
    }
}

impl From<hypervisor::instance::Status> for InstanceStatus {
    fn from(value: hypervisor::instance::Status) -> Self {
        match value {
            hypervisor::instance::Status::Running => InstanceStatus::Running,
            hypervisor::instance::Status::Stopped => InstanceStatus::Stopped,
            hypervisor::instance::Status::Unknown => InstanceStatus::UndefinedInstanceStatus,
        }
    }
}

#[tonic::async_trait]
impl<Auth: Authorize + 'static> instances_server::Instances for Instances<Auth> {
    /// CreateInstance provisions a new instance based on the specified configuration.
    /// Returns details of the newly created instance or a ProblemDetails on failure.
    async fn create(
        &self,
        request: tonic::Request<CreateInstanceRequest>,
    ) -> Result<tonic::Response<CreateInstanceResponse>, tonic::Status> {
        let principal = self.iam.principal(&request).await?;

        let request = request.into_inner();

        let request = InstanceCreateRequest {
            cores: request.cpu_cores as u8,
            project_id: Uuid::parse_str(&request.project_id)
                .map_err(|_| Error::MalformedId(request.project_id))?,
            disk_image: request.image,
            disk_size: request.disk_bytes,
            memory: request.memory_bytes as u32,
            name: request.name,
            snippet: request.snippet,
        };

        let instance = self.service.clone().create(&principal, request).await?;

        Ok(Response::new(CreateInstanceResponse {
            instance: Some(instance.into()),
        }))
    }

    /// DeleteInstance deletes a given instance.
    /// Returns an empty message or a ProblemDetails on failure.
    async fn delete(
        &self,
        request: tonic::Request<DeleteInstanceRequest>,
    ) -> std::result::Result<tonic::Response<DeleteInstanceResponse>, tonic::Status> {
        let principal = self.iam.principal(&request).await?;
        let id = request.into_inner().id;
        let id = Uuid::parse_str(&id).map_err(|_| Error::MalformedId(id))?;
        self.service.clone().delete(&principal, id).await?;
        Ok(Response::new(DeleteInstanceResponse {}))
    }

    /// CloneInstance provisions a new instance based on a given existing instance.
    /// Returns the cloned instance.
    async fn clone(
        &self,
        request: tonic::Request<CloneInstanceRequest>,
    ) -> std::result::Result<tonic::Response<Instance>, tonic::Status> {
        let principal = self.iam.principal(&request).await?;
        let id = request.into_inner().id;
        let id = Uuid::parse_str(&id).map_err(|_| Error::MalformedId(id))?;

        let instance = self.service.clone().clone_instance(&principal, id).await?;

        Ok(Response::new(instance.into()))
    }

    /// ListInstances retrieves information about all available instances.
    /// Returns a collection of instance details including their current status and resource usage.
    async fn list(
        &self,
        request: Request<ListInstancesRequest>,
    ) -> Result<Response<ListInstancesResponse>, Status> {
        tracing::info!("in rpc list...");

        let principal = self.iam.principal(&request).await?;
        let instances = self.service.clone().list(&principal).await?;

        Ok(Response::new(ListInstancesResponse {
            instances: instances.into_iter().map(Into::into).collect(),
        }))
    }

    /// StartInstance initiates a specific instance identified by its unique ID.
    /// Returns a response indicating success or a ProblemDetails on failure.
    async fn start(
        &self,
        request: Request<StartInstanceRequest>,
    ) -> Result<Response<StartInstanceResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let id = request.into_inner().id;
        let id = Uuid::parse_str(&id).map_err(|_| Error::MalformedId(id))?;

        self.service.clone().start(&principal, id).await?;
        Ok(Response::new(StartInstanceResponse {}))
    }

    /// StopInstance halts a specific instance identified by its unique ID.
    /// Returns a response indicating success or a ProblemDetails on failure.
    async fn stop(
        &self,
        request: Request<StopInstanceRequest>,
    ) -> Result<Response<StopInstanceResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let id = request.into_inner().id;
        let id = Uuid::parse_str(&id).map_err(|_| Error::MalformedId(id))?;
        self.service.clone().stop(&principal, id).await?;
        Ok(Response::new(StopInstanceResponse {}))
    }

    /// Update modifies an existing instance's properties.
    /// Returns the updated instance or a ProblemDetails on failure.
    async fn update(
        &self,
        request: Request<UpdateInstanceRequest>,
    ) -> Result<Response<UpdateInstanceResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();

        let id = Uuid::parse_str(&inner.id).map_err(|_| Error::MalformedId(inner.id))?;

        let project_id = inner
            .project_id
            .map(|id| Uuid::parse_str(&id).map_err(|_| Error::MalformedId(id)))
            .transpose()?;

        let request = InstanceUpdateRequest {
            id,
            name: inner.name,
            project_id,
        };

        let instance = self.service.clone().update(&principal, request).await?;

        Ok(Response::new(UpdateInstanceResponse {
            instance: Some(instance.into()),
        }))
    }
}

impl From<frn_core::compute::Zone> for Zone {
    fn from(value: frn_core::compute::Zone) -> Self {
        Zone {
            id: value.id.to_string(),
            name: value.name,
        }
    }
}

impl From<CreateZoneRequest> for frn_core::compute::ZoneCreateRequest {
    fn from(value: CreateZoneRequest) -> Self {
        Self { name: value.name }
    }
}

pub struct Zones<Auth: Authorize> {
    iam: IAM,
    zones: frn_core::compute::Zones<Auth>,
}

impl<Auth: Authorize> Zones<Auth> {
    pub fn new(iam: IAM, zones: frn_core::compute::Zones<Auth>) -> Self {
        Self { iam, zones }
    }
}

#[tonic::async_trait]
impl<Auth: Authorize + 'static> zones_server::Zones for Zones<Auth> {
    async fn list(
        &self,
        request: Request<ListZonesRequest>,
    ) -> Result<Response<ListZonesResponse>, Status> {
        let principal = self.iam.principal(&request).await?;

        let zones = self.zones.clone().list(&principal).await?;

        Ok(Response::new(ListZonesResponse {
            zones: zones.into_iter().map(Into::into).collect(),
        }))
    }

    async fn create(
        &self,
        request: Request<CreateZoneRequest>,
    ) -> Result<Response<CreateZoneResponse>, Status> {
        let principal = self.iam.principal(&request).await?;

        let zone = self
            .zones
            .clone()
            .create(&principal, request.into_inner().into())
            .await?;

        Ok(Response::new(CreateZoneResponse {
            zone: Some(zone.into()),
        }))
    }
}
