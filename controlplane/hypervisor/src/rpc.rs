use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use tonic::{Request, Response, Status};

use crate::{
    model::ActiveModel,
    v1::{
        ListHypervisorsRequest, ListHypervisorsResponse, RegisterHypervisorRequest,
        RegisterHypervisorResponse, hypervisors_server::Hypervisors,
    },
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

        // TODO: use problem and match the result

        Ok(Response::new(RegisterHypervisorResponse {}))
    }

    async fn list_hypervisors(
        &self,
        _: tonic::Request<ListHypervisorsRequest>,
    ) -> std::result::Result<Response<ListHypervisorsResponse>, Status> {
        let hypervisors = crate::model::Entity::find()
            .all(&self.database_connection)
            .await
            .expect("could not fetch from database")
            .into_iter()
            .map(|hypervisor| hypervisor.into())
            .collect();

        // TODO: use problem and match the result

        Ok(Response::new(ListHypervisorsResponse { hypervisors }))
    }
}

impl HypervisorsRpcService {
    pub fn new(database_connection: DatabaseConnection) -> Self {
        Self {
            database_connection,
        }
    }
}
