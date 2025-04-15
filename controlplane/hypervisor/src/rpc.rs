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
        let model: ActiveModel = request.into_inner().into();

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

#[cfg(test)]
mod tests {
    use sea_orm::{MockDatabase, MockExecResult};
    use tonic::Request;

    use crate::v1::{
        ListHypervisorsRequest, RegisterHypervisorRequest, hypervisors_server::Hypervisors,
    };

    use super::HypervisorsRpcService;

    #[tokio::test]
    async fn test_register_hypervisor_works() {
        // Arrange a service
        let connection = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
            .append_exec_results([MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            }])
            .append_query_results([vec![crate::model::Model::default()]])
            .into_connection();
        let service = HypervisorsRpcService::new(connection);

        // Act the call to the register_hypervisor procedure
        let result = service
            .register_hypervisor(Request::new(RegisterHypervisorRequest::default()))
            .await;

        // Assert the procedure result
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_hypervisors_works() {
        // Arrange a service
        let connection = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
            .append_query_results([vec![
                crate::model::Model::default(),
                crate::model::Model::default(),
            ]])
            .into_connection();
        let service = HypervisorsRpcService::new(connection);

        // Act the call to the register_hypervisor procedure
        let result = service
            .list_hypervisors(Request::new(ListHypervisorsRequest::default()))
            .await;

        // Assert the procedure result
        assert!(result.is_ok());
        let response = result.unwrap().into_inner();
        assert_eq!(response.hypervisors.len(), 2);
    }
}
