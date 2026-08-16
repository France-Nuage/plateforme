//! SpiceDB client
//!
//! Provides a gRPC client for connecting to SpiceDB authorization servers and
//! checking permissions. Use `SpiceDB::connect()` for production connections or
//! `SpiceDB::mock()` for testing with an in-memory server.

use crate::Error;
use crate::api::v1::check_permission_response::Permissionship;
use crate::api::v1::consistency::Requirement;
use crate::api::v1::relationship_update::Operation;
use crate::api::v1::{
    CheckPermissionRequest, Consistency, ObjectReference, RelationshipUpdate, SubjectReference,
    WriteRelationshipsRequest, ZedToken, permissions_service_client::PermissionsServiceClient,
};
use crate::api::v1::{LookupResourcesRequest, Relationship};
#[cfg(feature = "mock")]
use crate::mock::SpiceDBServer;
use std::str::FromStr;
use tonic::service::{Interceptor, interceptor::InterceptedService};
use tonic::transport::Channel;
use tonic::{Request, metadata::MetadataValue};
use tracing::error;

/// Reference to a SpiceDB object (type + id pair).
#[derive(Debug, Clone)]
pub struct ObjectRef {
    pub object_type: String,
    pub object_id: String,
}

impl ObjectRef {
    pub fn new(object_type: impl Into<String>, object_id: impl Into<String>) -> Self {
        Self {
            object_type: object_type.into(),
            object_id: object_id.into(),
        }
    }
}

/// Input for creating or deleting a SpiceDB relationship.
#[derive(Debug, Clone)]
pub struct RelationshipRef {
    pub subject_type: String,
    pub subject_id: String,
    pub relation: String,
    pub object_type: String,
    pub object_id: String,
}

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

    #[cfg(feature = "mock")]
    pub async fn mock() -> Self {
        let channel = SpiceDBServer::new().serve().await;
        Self::new(channel, String::new())
    }

    pub fn new(channel: Channel, token: String) -> Self {
        let client = PermissionsServiceClient::with_interceptor(
            channel,
            AuthenticationInterceptor::new(token),
        );
        Self { client }
    }

    pub async fn lookup(
        &mut self,
        subject: ObjectRef,
        permission: String,
        resource_type: String,
    ) -> Result<Vec<String>, Error> {
        let request = Request::new(LookupResourcesRequest {
            consistency: Some(Consistency {
                requirement: Some(Requirement::FullyConsistent(true)),
            }),
            context: None,
            optional_cursor: None,
            optional_limit: 0,
            resource_object_type: resource_type,
            permission,
            subject: Some(SubjectReference {
                object: Some(ObjectReference {
                    object_type: subject.object_type,
                    object_id: subject.object_id,
                }),
                optional_relation: String::new(),
            }),
        });

        let mut stream = self.client.lookup_resources(request).await?.into_inner();

        let mut resource_ids = Vec::new();

        while let Some(response) = stream.message().await? {
            resource_ids.push(response.resource_object_id);
        }

        Ok(resource_ids)
    }

    pub async fn check_permission(
        &mut self,
        subject: ObjectRef,
        permission: String,
        resource: ObjectRef,
    ) -> Result<(), Error> {
        let context = format!(
            "resource {}#{}, subject {}#{}, permission {}",
            resource.object_type,
            resource.object_id,
            subject.object_type,
            subject.object_id,
            permission
        );

        let request = Request::new(CheckPermissionRequest {
            consistency: Some(Consistency {
                requirement: Some(Requirement::FullyConsistent(true)),
            }),
            context: None,
            permission,
            resource: Some(ObjectReference {
                object_type: resource.object_type,
                object_id: resource.object_id,
            }),
            subject: Some(SubjectReference {
                object: Some(ObjectReference {
                    object_type: subject.object_type,
                    object_id: subject.object_id,
                }),
                optional_relation: String::new(),
            }),
            with_tracing: false,
        });

        let permissionship = self
            .client
            .check_permission(request)
            .await
            .inspect_err(|err| error!(%err, context, "spicedb permission check failed"))?
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

    pub async fn write_relationship(
        &mut self,
        rel: RelationshipRef,
    ) -> Result<Option<ZedToken>, Error> {
        self.apply_updates(vec![build_update(Operation::Touch, rel)])
            .await
    }

    /// Deletes a relationship from SpiceDB.
    pub async fn delete_relationship(
        &mut self,
        rel: RelationshipRef,
    ) -> Result<Option<ZedToken>, Error> {
        self.apply_updates(vec![build_update(Operation::Delete, rel)])
            .await
    }

    pub async fn write_relationships(
        &mut self,
        relationships: Vec<RelationshipRef>,
    ) -> Result<Option<ZedToken>, Error> {
        let updates = relationships
            .into_iter()
            .map(|rel| build_update(Operation::Touch, rel))
            .collect();
        self.apply_updates(updates).await
    }

    pub async fn delete_relationships(
        &mut self,
        relationships: Vec<RelationshipRef>,
    ) -> Result<Option<ZedToken>, Error> {
        let updates = relationships
            .into_iter()
            .map(|rel| build_update(Operation::Delete, rel))
            .collect();
        self.apply_updates(updates).await
    }

    async fn apply_updates(
        &mut self,
        updates: Vec<RelationshipUpdate>,
    ) -> Result<Option<ZedToken>, Error> {
        let request = Request::new(WriteRelationshipsRequest {
            optional_preconditions: vec![],
            updates,
        });

        self.client
            .write_relationships(request)
            .await
            .map(|response| response.into_inner().written_at)
            .map_err(Into::into)
    }
}

fn build_update(operation: Operation, rel: RelationshipRef) -> RelationshipUpdate {
    RelationshipUpdate {
        operation: operation as i32,
        relationship: Some(Relationship {
            optional_caveat: None,
            resource: Some(ObjectReference {
                object_id: rel.object_id,
                object_type: rel.object_type,
            }),
            relation: rel.relation,
            subject: Some(SubjectReference {
                object: Some(ObjectReference {
                    object_id: rel.subject_id,
                    object_type: rel.subject_type,
                }),
                optional_relation: String::new(),
            }),
        }),
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
