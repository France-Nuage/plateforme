use crate::{error::Error, request::ExtractToken};
use frn_core::authorization::AuthorizationServer;
use frn_core::compute::{HypervisorCreateRequest, Hypervisors as Service};
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

pub struct Hypervisors<Auth: AuthorizationServer> {
    iam: IAM,
    _pool: Pool<Postgres>,
    service: Service<Auth>,
}

impl<Auth: AuthorizationServer> Hypervisors<Auth> {
    pub fn new(iam: IAM, pool: Pool<Postgres>, service: Service<Auth>) -> Self {
        Self {
            iam,
            _pool: pool,
            service,
        }
    }
}

#[tonic::async_trait]
impl<Auth: AuthorizationServer + 'static> hypervisors_server::Hypervisors for Hypervisors<Auth> {
    async fn detach(
        &self,
        request: Request<DetachHypervisorRequest>,
    ) -> Result<Response<DetachHypervisorResponse>, Status> {
        let principal = self.iam.user(request.access_token()).await?;
        let id = request.into_inner().id;
        let id = id.parse::<Uuid>().map_err(|_| Error::MalformedId(id))?;

        self.service.clone().delete(&principal, id).await?;

        Ok(Response::new(DetachHypervisorResponse {}))
    }

    async fn list(
        &self,
        request: Request<ListHypervisorsRequest>,
    ) -> Result<Response<ListHypervisorsResponse>, Status> {
        let principal = self.iam.user(request.access_token()).await?;

        let hypervisors = self.service.clone().list(&principal).await?;

        Ok(Response::new(ListHypervisorsResponse {
            hypervisors: hypervisors.into_iter().map(Into::into).collect(),
        }))
    }

    async fn register(
        &self,
        request: Request<RegisterHypervisorRequest>,
    ) -> Result<Response<RegisterHypervisorResponse>, Status> {
        let principal = self.iam.user(request.access_token()).await?;
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

impl From<frn_core::compute::Zone> for Zone {
    fn from(value: frn_core::compute::Zone) -> Self {
        Zone {
            id: value.id.to_string(),
            name: value.name,
        }
    }
}

pub struct Zones<Auth: AuthorizationServer> {
    iam: IAM,
    zones: frn_core::compute::Zones<Auth>,
}

impl<Auth: AuthorizationServer> Zones<Auth> {
    pub fn new(iam: IAM, zones: frn_core::compute::Zones<Auth>) -> Self {
        Self { iam, zones }
    }
}

#[tonic::async_trait]
impl<Auth: AuthorizationServer + 'static> zones_server::Zones for Zones<Auth> {
    async fn list(
        &self,
        request: Request<ListZonesRequest>,
    ) -> Result<Response<ListZonesResponse>, Status> {
        let principal = self.iam.user(request.access_token()).await?;

        let zones = self.zones.clone().list(&principal).await?;

        Ok(Response::new(ListZonesResponse {
            zones: zones.into_iter().map(Into::into).collect(),
        }))
    }
}
