use tonic::{Request, Response, Status};

use crate::{
    DatacenterService,
    v1::{ListDatacentersRequest, ListDatacentersResponse, datacenters_server::Datacenters},
};

pub struct DatacenterRpcService {
    service: DatacenterService,
}

impl DatacenterRpcService {
    /// Create a new instance of the Datacenter gRPC service.
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            service: DatacenterService::new(pool),
        }
    }
}

#[tonic::async_trait]
impl Datacenters for DatacenterRpcService {
    async fn list(
        &self,
        _: Request<ListDatacentersRequest>,
    ) -> Result<Response<ListDatacentersResponse>, Status> {
        let datacenters = self.service.list().await?;

        Ok(Response::new(ListDatacentersResponse {
            datacenters: datacenters.into_iter().map(Into::into).collect(),
        }))
    }
}
