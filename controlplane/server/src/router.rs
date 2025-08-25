//! gRPC service routing and composition for the server.
//!
//! This module provides the [`Router`] structure that manages the registration
//! and composition of gRPC services. It encapsulates tonic's routing system
//! and provides a builder pattern for progressive service registration with
//! proper dependency injection.

use hypervisors::{rpc::HypervisorsRpcService, v1::hypervisors_server::HypervisorsServer};
use infrastructure::DatacenterRpcService;
use infrastructure::ZeroTrustNetworkRpcService;
use infrastructure::ZeroTrustNetworkTypeRpcService;
use infrastructure::v1::datacenters_server::DatacentersServer;
use infrastructure::v1::zero_trust_network_types_server::ZeroTrustNetworkTypesServer;
use infrastructure::v1::zero_trust_networks_server::ZeroTrustNetworksServer;
use instances::{InstancesRpcService, v1::instances_server::InstancesServer};
use resources::{rpc::ResourcesRpcService, v1::resources_server::ResourcesServer};
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

    /// Registers the datacenters service with the router.
    ///
    /// This method adds the datacenters gRPC service to the router, providing
    /// endpoints for datacenter management and location operations. The service
    /// is configured with the provided database pool for persistent storage
    /// operations.
    ///
    /// # Parameters
    ///
    /// * `pool` - PostgreSQL database connection pool for database operations
    pub fn datacenters(self, pool: Pool<Postgres>) -> Self {
        Self {
            routes: self
                .routes
                .add_service(DatacentersServer::new(DatacenterRpcService::new(pool))),
        }
    }

    /// Registers the health check service with the router.
    ///
    /// This method adds the gRPC health check service to the router, providing
    /// health status endpoints for service monitoring and load balancer health
    /// checks. The health reporter is configured to monitor all registered
    /// services in the background.
    ///
    /// # Background Task
    ///
    /// This method spawns a background task to set all services as serving.
    /// The task is detached and runs independently of the main application
    /// lifecycle.
    pub fn health(self) -> Self {
        let (health_reporter, health_service) = tonic_health::server::health_reporter();
        // Start the health reporter in a background task and forget about it.
        // This is definitely not a graceful pattern, but it is as of now the
        // only part of the application builder that would require async. As
        // we don't care for the future output (which is a unit type `()`), I
        // deemed it better than introducing async in the builder.
        //
        // If this part later introduces a nasty bug, I take full responsability
        // that i "wiped the shit under the carpet" here.
        tokio::spawn(async move {
            tokio::join!(
                health_reporter.set_serving::<DatacentersServer<DatacenterRpcService>>(),
                health_reporter.set_serving::<HypervisorsServer<HypervisorsRpcService>>(),
                health_reporter.set_serving::<InstancesServer<InstancesRpcService>>(),
                health_reporter.set_serving::<ResourcesServer<ResourcesRpcService>>(),
                health_reporter
                    .set_serving::<ZeroTrustNetworkTypesServer<ZeroTrustNetworkTypeRpcService>>(),
                health_reporter
                    .set_serving::<ZeroTrustNetworksServer<ZeroTrustNetworkRpcService>>(),
            )
        });

        Self {
            routes: self.routes.add_service(health_service),
        }
    }

    /// Registers the hypervisors management service with the router.
    ///
    /// This method adds the hypervisors gRPC service to the router, providing
    /// endpoints for hypervisor registration, management, and monitoring
    /// operations. The service is configured with the provided database pool
    /// for persistent storage operations.
    ///
    /// # Parameters
    ///
    /// * `pool` - PostgreSQL database connection pool for database operations
    pub fn hypervisors(self, pool: Pool<Postgres>) -> Self {
        Self {
            routes: self
                .routes
                .add_service(HypervisorsServer::new(HypervisorsRpcService::new(pool))),
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
    pub fn instances(self, pool: Pool<Postgres>) -> Self {
        Self {
            routes: self
                .routes
                .add_service(InstancesServer::new(InstancesRpcService::new(pool))),
        }
    }

    /// Registers the resources management service with the router.
    ///
    /// This method adds the resources gRPC service to the router, providing
    /// endpoints for resource allocation, monitoring, and management operations.
    /// The service is configured with the provided database pool for persistent
    /// storage operations.
    ///
    /// # Parameters
    ///
    /// * `pool` - PostgreSQL database connection pool for database operations
    pub fn resources(self, pool: Pool<Postgres>) -> Self {
        Self {
            routes: self
                .routes
                .add_service(ResourcesServer::new(ResourcesRpcService::new(pool))),
        }
    }

    /// Registers the zero trust network types service with the router.
    ///
    /// This method adds the zero trust network types gRPC service to the router,
    /// providing endpoints for managing different types of zero trust network
    /// configurations. The service is configured with the provided database pool
    /// for persistent storage operations.
    ///
    /// # Parameters
    ///
    /// * `pool` - PostgreSQL database connection pool for database operations
    pub fn zero_trust_network_types(self, pool: Pool<Postgres>) -> Self {
        Self {
            routes: self.routes.add_service(ZeroTrustNetworkTypesServer::new(
                ZeroTrustNetworkTypeRpcService::new(pool),
            )),
        }
    }

    /// Registers the zero trust networks service with the router.
    ///
    /// This method adds the zero trust networks gRPC service to the router,
    /// providing endpoints for zero trust network creation, configuration, and
    /// management operations. The service is configured with the provided
    /// database pool for persistent storage operations.
    ///
    /// # Parameters
    ///
    /// * `pool` - PostgreSQL database connection pool for database operations
    pub fn zero_trust_networks(self, pool: Pool<Postgres>) -> Self {
        Self {
            routes: self.routes.add_service(ZeroTrustNetworksServer::new(
                ZeroTrustNetworkRpcService::new(pool),
            )),
        }
    }
}
