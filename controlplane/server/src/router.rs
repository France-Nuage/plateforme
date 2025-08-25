//! gRPC service routing and composition for the server.
//!
//! This module provides the [`Router`] structure that manages the registration
//! and composition of gRPC services. It encapsulates tonic's routing system
//! and provides a builder pattern for progressive service registration with
//! proper dependency injection.

use instances::{InstancesRpcService, v1::instances_server::InstancesServer};
use sqlx::{Pool, Postgres};
use tonic::service::Routes;

/// gRPC service router for managing and composing service endpoints.
///
/// This structure encapsulates the tonic [`Routes`] collection and provides
/// a builder pattern for registering gRPC services. It serves as the central
/// registry for all available gRPC services in the application, enabling
/// progressive service composition and dependency injection.
///
/// [`Routes`]: https://docs.rs/tonic/latest/tonic/service/struct.Routes.html
pub struct Router {
    /// Collection of registered gRPC service routes.
    pub routes: Routes,
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    /// Creates a new [`Router`] instance with no registered services.
    ///
    /// This constructor initializes an empty router ready for service
    /// registration through the builder pattern methods.
    pub fn new() -> Self {
        Self {
            routes: Routes::default(),
        }
    }

    /// Registers the instances management service with the router.
    ///
    /// This method adds the instances gRPC service to the router, providing
    /// endpoints for virtual machine lifecycle management operations. The
    /// service is configured with the provided database pool for persistent
    /// storage operations.
    ///
    /// # Parameters
    ///
    /// * `pool` - Database connection pool for database operations (generic over DB type)
    pub fn instances(self, pool: Pool<Postgres>) -> Self
where {
        Self {
            routes: self
                .routes
                .add_service(InstancesServer::new(InstancesRpcService::new(pool))),
        }
    }
}
