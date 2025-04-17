//! Implementation of the gRPC service for hypervisor management.
//!
//! This module provides the implementation of the Hypervisors gRPC service,
//! handling requests to list and register hypervisors.

use crate::{
    model::ActiveModel,
    problem::Problem,
    v1::{
        ListHypervisorsRequest, ListHypervisorsResponse, RegisterHypervisorRequest,
        RegisterHypervisorResponse, hypervisors_server::Hypervisors,
    },
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use std::{ops::Deref, sync::Arc};
use tonic::{Request, Response, Status};

/// Implementation of the Hypervisors gRPC service.
///
/// This service handles operations related to hypervisor management,
/// including listing and registering hypervisors. It uses a database
/// connection to persist and retrieve hypervisor information.
pub struct HypervisorsRpcService {
    /// Database connection used for hypervisor data persistence.
    database_connection: Arc<DatabaseConnection>,
}

#[tonic::async_trait]
impl Hypervisors for HypervisorsRpcService {
    /// Registers a new hypervisor in the system.
    ///
    /// This method persists the hypervisor information provided in the request
    /// to the database, generating a new UUID for the hypervisor.
    ///
    /// # Arguments
    ///
    /// * `request` - Contains the hypervisor details to be registered
    ///
    /// # Returns
    ///
    /// * `Ok(Response<RegisterHypervisorResponse>)` - On successful registration
    /// * `Err(Status)` - If registration fails, with appropriate status code
    async fn register_hypervisor(
        &self,
        request: Request<RegisterHypervisorRequest>,
    ) -> Result<Response<RegisterHypervisorResponse>, Status> {
        let model: ActiveModel = request.into_inner().into();

        model
            .insert(self.database_connection.deref())
            .await
            .map_err(Problem::from)?;

        Ok(Response::new(RegisterHypervisorResponse {}))
    }

    /// Lists all registered hypervisors.
    ///
    /// This method retrieves all hypervisor records from the database
    /// and returns them as a collection.
    ///
    /// # Arguments
    ///
    /// * `_` - Empty request
    ///
    /// # Returns
    ///
    /// * `Ok(Response<ListHypervisorsResponse>)` - Contains the list of hypervisors
    /// * `Err(Status)` - If retrieval fails, with appropriate status code
    async fn list_hypervisors(
        &self,
        _: tonic::Request<ListHypervisorsRequest>,
    ) -> std::result::Result<Response<ListHypervisorsResponse>, Status> {
        let hypervisors = crate::model::Entity::find()
            .all(self.database_connection.deref())
            .await
            .map_err(Problem::from)?
            .into_iter()
            .map(|hypervisor| hypervisor.into())
            .collect();

        Ok(Response::new(ListHypervisorsResponse { hypervisors }))
    }
}

impl HypervisorsRpcService {
    /// Creates a new instance of the Hypervisors gRPC service.
    ///
    /// # Arguments
    ///
    /// * `database_connection` - Database connection for hypervisor data persistence
    ///
    /// # Returns
    ///
    /// A new `HypervisorsRpcService` instance
    pub fn new(database_connection: Arc<DatabaseConnection>) -> Self {
        Self {
            database_connection,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

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
        let service = HypervisorsRpcService::new(Arc::new(connection));

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
        let service = HypervisorsRpcService::new(Arc::new(connection));

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
