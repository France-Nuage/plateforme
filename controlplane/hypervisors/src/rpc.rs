//! Implementation of the gRPC service for hypervisor management.
//!
//! This module provides the implementation of the Hypervisors gRPC service,
//! handling requests to list and register hypervisors.

use crate::{
    model::Hypervisor,
    repository,
    v1::{
        ListHypervisorsRequest, ListHypervisorsResponse, RegisterHypervisorRequest,
        RegisterHypervisorResponse, hypervisors_server::Hypervisors,
    },
};
use sqlx::PgPool;
use tonic::{Request, Response, Status};

/// Implementation of the Hypervisors gRPC service.
///
/// This service handles operations related to hypervisor management,
/// including listing and registering hypervisors. It uses a database
/// connection to persist and retrieve hypervisor information.
pub struct HypervisorsRpcService {
    /// Database connection used for hypervisor data persistence.
    pool: sqlx::PgPool,
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
        let model: Hypervisor = request.into_inner().into();

        repository::create(&self.pool, &model).await?;

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
        let hypervisors = repository::list(&self.pool)
            .await?
            .into_iter()
            .map(Into::into)
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
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use crate::v1::{
        ListHypervisorsRequest, RegisterHypervisorRequest, hypervisors_server::Hypervisors,
    };
    use tonic::Request;

    use super::HypervisorsRpcService;

    #[sqlx::test(migrations = "../migrations")]
    async fn test_register_hypervisor_works(pool: sqlx::PgPool) {
        // Arrange a service
        let service = HypervisorsRpcService::new(pool);

        // Act the call to the register_hypervisor procedure
        let result = service
            .register_hypervisor(Request::new(RegisterHypervisorRequest::default()))
            .await;

        // Assert the procedure result
        assert!(result.is_ok());
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn test_list_hypervisors_works(pool: sqlx::PgPool) {
        // Arrange a service
        let service = HypervisorsRpcService::new(pool);

        // Act the call to the register_hypervisor procedure
        let result = service
            .list_hypervisors(Request::new(ListHypervisorsRequest::default()))
            .await;

        // Assert the procedure result
        assert!(result.is_ok());
        let response = result.unwrap().into_inner();
        assert_eq!(response.hypervisors.len(), 0);
    }
}
