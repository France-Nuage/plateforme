//! gRPC service routing and composition for the server.
//!
//! This module provides the [`Router`] structure that manages the registration
//! and composition of gRPC services. It encapsulates tonic's routing system
//! and provides a builder pattern for progressive service registration with
//! proper dependency injection.

use frn_core::identity::IAM;
use frn_rpc::v1::compute::Hypervisors;
use frn_rpc::v1::compute::Instances;
use frn_rpc::v1::compute::Zones;
use frn_rpc::v1::compute::hypervisors_server::HypervisorsServer;
use frn_rpc::v1::compute::instances_server::InstancesServer;
use frn_rpc::v1::compute::zones_server::ZonesServer;
use frn_rpc::v1::iam::Invitations;
use frn_rpc::v1::iam::invitations_server::InvitationsServer;
use frn_rpc::v1::resourcemanager::Organizations;
use frn_rpc::v1::resourcemanager::Projects;
use frn_rpc::v1::resourcemanager::organizations_server::OrganizationsServer;
use frn_rpc::v1::resourcemanager::projects_server::ProjectsServer;
use infrastructure::ZeroTrustNetworkRpcService;
use infrastructure::ZeroTrustNetworkTypeRpcService;
use infrastructure::v1::zero_trust_network_types_server::ZeroTrustNetworkTypesServer;
use infrastructure::v1::zero_trust_networks_server::ZeroTrustNetworksServer;
use spicedb::SpiceDB;
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
                health_reporter.set_serving::<HypervisorsServer<Hypervisors<SpiceDB>>>(),
                health_reporter.set_serving::<InstancesServer<Instances<SpiceDB>>>(),
                health_reporter.set_serving::<InvitationsServer<Invitations<SpiceDB>>>(),
                health_reporter.set_serving::<OrganizationsServer<Organizations<SpiceDB>>>(),
                health_reporter.set_serving::<ProjectsServer<Projects<SpiceDB>>>(),
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
    pub fn hypervisors(
        self,
        iam: IAM,
        pool: Pool<Postgres>,
        service: frn_core::compute::Hypervisors<SpiceDB>,
    ) -> Self {
        Self {
            routes: self
                .routes
                .add_service(HypervisorsServer::new(Hypervisors::new(iam, pool, service))),
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
    pub fn instances(
        self,
        iam: IAM,
        pool: Pool<Postgres>,
        instances: frn_core::compute::Instances<SpiceDB>,
    ) -> Self {
        Self {
            routes: self
                .routes
                .add_service(InstancesServer::new(Instances::new(iam, pool, instances))),
        }
    }

    pub fn invitations(
        self,
        iam: IAM,
        invitations: frn_core::identity::Invitations<SpiceDB>,
        users: frn_core::identity::Users<SpiceDB>,
    ) -> Self {
        Self {
            routes: self
                .routes
                .add_service(InvitationsServer::new(Invitations::new(
                    iam,
                    invitations,
                    users,
                ))),
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
    pub fn resources(
        self,
        iam: IAM,
        organizations: frn_core::resourcemanager::Organizations<SpiceDB>,
        pool: Pool<Postgres>,
    ) -> Self {
        Self {
            routes: self
                .routes
                .add_service(OrganizationsServer::new(Organizations::new(
                    iam.clone(),
                    organizations.clone(),
                    pool.clone(),
                )))
                .add_service(ProjectsServer::new(Projects::<SpiceDB>::new(iam, pool))),
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

    pub fn zones(self, iam: IAM, zones: frn_core::compute::Zones<SpiceDB>) -> Self {
        Self {
            routes: self
                .routes
                .add_service(ZonesServer::new(Zones::new(iam, zones))),
        }
    }

    /// Registers the gRPC reflection services (v1 and v1alpha) with the router.
    ///
    /// This method adds both v1 and v1alpha gRPC server reflection services, which
    /// enable runtime introspection of the available gRPC services and their schemas.
    /// This is particularly useful for debugging with tools like grpcurl, grpcui, or
    /// Bruno that can dynamically discover service definitions.
    ///
    /// Both versions are registered to ensure compatibility with different clients:
    /// - v1: The current stable reflection API (grpcurl, grpcui)
    /// - v1alpha: Legacy reflection API (Bruno, older clients)
    ///
    /// The reflection services provide metadata about all registered gRPC services,
    /// allowing clients to query available methods, message types, and service
    /// definitions without requiring access to .proto files.
    pub fn reflection(self) -> Self {
        // Build v1 reflection service
        let reflection_v1 = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(frn_rpc::REFLECTION_DESCRIPTOR_V1)
            .build_v1()
            .expect("failed to build v1 reflection service");

        // Build v1alpha reflection service for compatibility with clients like Bruno
        let reflection_v1alpha = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(frn_rpc::REFLECTION_DESCRIPTOR_V1ALPHA)
            .build_v1alpha()
            .expect("failed to build v1alpha reflection service");

        Self {
            routes: self
                .routes
                .add_service(reflection_v1)
                .add_service(reflection_v1alpha),
        }
    }
}
