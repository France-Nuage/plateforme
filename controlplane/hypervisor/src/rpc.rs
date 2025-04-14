use sea_orm::{ActiveModelTrait, DatabaseConnection};
use tonic::{Request, Response, Status};

use crate::{
    model::ActiveModel,
    v1::{RegisterHypervisorRequest, RegisterHypervisorResponse, hypervisors_server::Hypervisors},
};

pub struct HypervisorsRpcService {
    database_connection: DatabaseConnection,
}

#[tonic::async_trait]
impl Hypervisors for HypervisorsRpcService {
    async fn register_hypervisor(
        &self,
        request: Request<RegisterHypervisorRequest>,
    ) -> Result<Response<RegisterHypervisorResponse>, Status> {
        let model: ActiveModel = request
            .into_inner()
            .hypervisor
            .expect("hypervisor required")
            .into();

        model
            .insert(&self.database_connection)
            .await
            .expect("could not insert into database");

        Ok(Response::new(RegisterHypervisorResponse {}))
    }
}

impl HypervisorsRpcService {
    pub fn new(database_connection: DatabaseConnection) -> Self {
        Self {
            database_connection,
        }
    }
}
