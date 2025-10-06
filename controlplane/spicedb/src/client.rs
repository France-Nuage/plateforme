//! SpiceDB client
//!
//! Provides a gRPC client for connecting to SpiceDB authorization servers and
//! checking permissions. Use `SpiceDB::connect()` for production connections or
//! `SpiceDB::mock()` for testing with an in-memory server.

use crate::Error;
use crate::api::v1::check_permission_response::Permissionship;
use crate::api::v1::{
    CheckPermissionRequest, ObjectReference, SubjectReference,
    permissions_service_client::PermissionsServiceClient,
};
use crate::mock::SpiceDBServer;
use std::str::FromStr;
use tonic::service::{Interceptor, interceptor::InterceptedService};
use tonic::transport::Channel;
use tonic::{Request, metadata::MetadataValue};

/// A client for interacting with SpiceDB's grpc API.
#[derive(Clone)]
pub struct SpiceDB {
    client: PermissionsServiceClient<InterceptedService<Channel, AuthenticationInterceptor>>,
}

impl SpiceDB {
    pub async fn connect(url: &str, token: &str) -> Result<Self, Error> {
        let channel = Channel::from_shared(url.to_owned())
            .map_err(|_| Error::UnparsableUrl)?
            .connect()
            .await
            .map_err(|_| Error::UnreachableServer)?;

        let client = SpiceDB::new(channel, token.to_owned());

        Ok(client)
    }

    pub async fn mock() -> Self {
        let channel = SpiceDBServer::new().serve().await;
        Self::new(channel, "".to_owned())
    }

    pub fn new(channel: Channel, token: String) -> Self {
        let client = PermissionsServiceClient::with_interceptor(
            channel,
            AuthenticationInterceptor::new(token),
        );
        Self { client }
    }

    pub async fn check_permission(
        &mut self,
        (subject_type, subject_id): (String, String),
        permission: String,
        (resource_type, resource_id): (String, String),
    ) -> Result<(), Error> {
        // forge the check permission request
        let request = Request::new(CheckPermissionRequest {
            consistency: None,
            context: None,
            permission,
            resource: Some(ObjectReference {
                object_type: resource_type,
                object_id: resource_id,
            }),
            subject: Some(SubjectReference {
                object: Some(ObjectReference {
                    object_type: subject_type,
                    object_id: subject_id,
                }),
                optional_relation: "".to_owned(),
            }),
            with_tracing: false,
        });

        let permissionship = self
            .client
            .check_permission(request)
            .await?
            .into_inner()
            .permissionship();

        match permissionship {
            Permissionship::HasPermission => Ok(()),
            Permissionship::NoPermission => Err(Error::Forbidden),
            Permissionship::Unspecified => Err(Error::Internal(
                "Permissionship::Unspecified is not implemented".to_owned(),
            )),
            Permissionship::ConditionalPermission => Err(Error::Internal(
                "Permissionship::ConditionalPermission is not implemented".to_owned(),
            )),
        }
    }
}

/// Interceptor that adds authentication tokens to gRPC requests.
#[derive(Clone)]
pub struct AuthenticationInterceptor {
    token: String,
}

impl AuthenticationInterceptor {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

impl Interceptor for AuthenticationInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        let value = MetadataValue::from_str(&format!("Bearer {}", self.token))
            .map_err(|_| tonic::Status::internal("unparsable token"))?;
        request.metadata_mut().insert("authorization", value);
        Ok(request)
    }
}
