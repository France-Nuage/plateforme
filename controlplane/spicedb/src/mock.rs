use crate::SpiceDB;
use crate::api::v1::{
    CheckBulkPermissionsRequest, CheckBulkPermissionsResponse, CheckPermissionRequest,
    CheckPermissionResponse, DeleteRelationshipsRequest, DeleteRelationshipsResponse,
    ExpandPermissionTreeRequest, ExpandPermissionTreeResponse, LookupResourcesRequest,
    LookupResourcesResponse, LookupSubjectsRequest, LookupSubjectsResponse,
    ReadRelationshipsRequest, ReadRelationshipsResponse, Relationship, WriteRelationshipsRequest,
    WriteRelationshipsResponse, ZedToken,
    check_permission_response::Permissionship,
    permissions_service_server::{PermissionsService, PermissionsServiceServer},
    relationship_update::Operation,
};
use futures::Stream;
use std::collections::HashSet;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{
    Request, Response, Status,
    transport::{Channel, Server},
};

/// The single resource id that the mock's `LookupResources` authorizes.
///
/// The mock grants every permission, but `LookupResources` still has to return
/// concrete ids, and the lookup layer only keeps resources whose id is in that
/// set. Tests that expect a resource to appear in a lookup-filtered list
/// (organizations, projects) must seed it with this id/slug so it matches.
///
/// The value is a valid organization/project slug (letters and hyphens, per the
/// `check_*_slug_pattern` constraints) so those slug-keyed resources can be
/// seeded with it directly.
pub const AUTHORIZED_RESOURCE_ID: &str = "authorized-resource";

async fn serve_mock(svc: impl PermissionsService) -> Channel {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("cannot reserve a port");
    let addr = listener
        .local_addr()
        .expect("cannot convert to a local address");

    tokio::spawn(async move {
        Server::builder()
            .add_service(PermissionsServiceServer::new(svc))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .expect("could not serve mock spicedb server");
    });

    Channel::from_shared(format!("http://{addr}"))
        .expect("could not create channel")
        .connect()
        .await
        .expect("could not connect to channel")
}

pub struct SpiceDBServer {
    permissionship: Permissionship,
}

impl Default for SpiceDBServer {
    fn default() -> Self {
        Self::new()
    }
}

impl SpiceDBServer {
    pub fn new() -> Self {
        Self {
            permissionship: Permissionship::HasPermission,
        }
    }

    pub fn denying() -> Self {
        Self {
            permissionship: Permissionship::NoPermission,
        }
    }

    pub async fn serve(self) -> Channel {
        serve_mock(self).await
    }
}

#[tonic::async_trait]
impl PermissionsService for SpiceDBServer {
    type ReadRelationshipsStream =
        Pin<Box<dyn Stream<Item = Result<ReadRelationshipsResponse, Status>> + Send>>;
    type LookupResourcesStream =
        Pin<Box<dyn Stream<Item = Result<LookupResourcesResponse, Status>> + Send>>;
    type LookupSubjectsStream =
        Pin<Box<dyn Stream<Item = Result<LookupSubjectsResponse, Status>> + Send>>;

    async fn check_bulk_permissions(
        &self,
        _: Request<CheckBulkPermissionsRequest>,
    ) -> Result<Response<CheckBulkPermissionsResponse>, Status> {
        unimplemented!()
    }

    /// Checks permission matching the request.
    ///
    /// CheckPermission determines for a given resource whether a subject
    /// computes to having a permission or is a direct member of a particular
    /// relation.
    async fn check_permission(
        &self,
        _: Request<CheckPermissionRequest>,
    ) -> Result<Response<CheckPermissionResponse>, Status> {
        Ok(Response::new(CheckPermissionResponse {
            checked_at: None,
            debug_trace: None,
            partial_caveat_info: None,
            permissionship: self.permissionship as i32,
        }))
    }

    /// Delete the relationships matching the request.
    async fn delete_relationships(
        &self,
        _: Request<DeleteRelationshipsRequest>,
    ) -> Result<Response<DeleteRelationshipsResponse>, Status> {
        unimplemented!()
    }

    /// Expand the permission tree matching the request.
    async fn expand_permission_tree(
        &self,
        _: Request<ExpandPermissionTreeRequest>,
    ) -> Result<Response<ExpandPermissionTreeResponse>, Status> {
        unimplemented!()
    }

    /// Lookup the resources matching the request.
    async fn lookup_resources(
        &self,
        _: Request<LookupResourcesRequest>,
    ) -> Result<Response<Self::LookupResourcesStream>, Status> {
        // Create a stream with a single response item
        let response = LookupResourcesResponse {
            after_result_cursor: None,
            looked_up_at: None,
            partial_caveat_info: None,
            permissionship: Permissionship::HasPermission as i32,
            resource_object_id: AUTHORIZED_RESOURCE_ID.to_owned(),
        };

        let stream = futures::stream::iter(vec![Ok(response)]);
        Ok(Response::new(Box::pin(stream)))
    }

    /// Lookup the subjects matching the request.
    async fn lookup_subjects(
        &self,
        _: Request<LookupSubjectsRequest>,
    ) -> Result<Response<Self::LookupSubjectsStream>, Status> {
        unimplemented!()
    }

    /// Read the relationships matching the request.
    async fn read_relationships(
        &self,
        _: Request<ReadRelationshipsRequest>,
    ) -> Result<Response<Self::ReadRelationshipsStream>, Status> {
        unimplemented!()
    }

    /// Write the relationships matching the request.
    async fn write_relationships(
        &self,
        _: Request<WriteRelationshipsRequest>,
    ) -> Result<Response<WriteRelationshipsResponse>, Status> {
        Ok(Response::new(WriteRelationshipsResponse {
            written_at: Some(ZedToken::default()),
        }))
    }
}

/// Shared, in-memory relationship store keyed by SpiceDB's textual form
/// `object_type:object_id#relation@subject_type:subject_id`.
pub type RelationshipStore = Arc<Mutex<HashSet<String>>>;

/// Builds the textual key for a proto relationship, matching the `Display`
/// impl of `frn_core::authorization::Relationship` so tests can assert on it
/// directly.
fn relationship_key(rel: &Relationship) -> Option<String> {
    let resource = rel.resource.as_ref()?;
    let subject = rel.subject.as_ref()?.object.as_ref()?;
    Some(format!(
        "{}:{}#{}@{}:{}",
        resource.object_type,
        resource.object_id,
        rel.relation,
        subject.object_type,
        subject.object_id
    ))
}

/// An in-process SpiceDB test double that records relationship writes and
/// deletes into a shared [`RelationshipStore`].
///
/// The real SpiceDB client routes both writes and deletes through the
/// `write_relationships` RPC (`Operation::Touch` vs `Operation::Delete`), so
/// only that RPC is stateful here: it lets a test assert exactly which
/// relationships an operation persisted or removed at the gRPC boundary. The
/// read and permission RPCs are not implemented because the operations under
/// test never call them.
pub struct RecordingSpiceDBServer {
    store: RelationshipStore,
}

impl RecordingSpiceDBServer {
    /// Creates a recording server and returns it alongside the shared store its
    /// `write_relationships` RPC mutates.
    pub fn new() -> (Self, RelationshipStore) {
        let store: RelationshipStore = Arc::new(Mutex::new(HashSet::new()));
        (
            Self {
                store: store.clone(),
            },
            store,
        )
    }

    pub async fn serve(self) -> Channel {
        serve_mock(self).await
    }
}

#[tonic::async_trait]
impl PermissionsService for RecordingSpiceDBServer {
    type ReadRelationshipsStream =
        Pin<Box<dyn Stream<Item = Result<ReadRelationshipsResponse, Status>> + Send>>;
    type LookupResourcesStream =
        Pin<Box<dyn Stream<Item = Result<LookupResourcesResponse, Status>> + Send>>;
    type LookupSubjectsStream =
        Pin<Box<dyn Stream<Item = Result<LookupSubjectsResponse, Status>> + Send>>;

    async fn check_bulk_permissions(
        &self,
        _: Request<CheckBulkPermissionsRequest>,
    ) -> Result<Response<CheckBulkPermissionsResponse>, Status> {
        unimplemented!()
    }

    async fn check_permission(
        &self,
        _: Request<CheckPermissionRequest>,
    ) -> Result<Response<CheckPermissionResponse>, Status> {
        unimplemented!()
    }

    async fn delete_relationships(
        &self,
        _: Request<DeleteRelationshipsRequest>,
    ) -> Result<Response<DeleteRelationshipsResponse>, Status> {
        unimplemented!()
    }

    async fn expand_permission_tree(
        &self,
        _: Request<ExpandPermissionTreeRequest>,
    ) -> Result<Response<ExpandPermissionTreeResponse>, Status> {
        unimplemented!()
    }

    async fn lookup_resources(
        &self,
        _: Request<LookupResourcesRequest>,
    ) -> Result<Response<Self::LookupResourcesStream>, Status> {
        unimplemented!()
    }

    async fn lookup_subjects(
        &self,
        _: Request<LookupSubjectsRequest>,
    ) -> Result<Response<Self::LookupSubjectsStream>, Status> {
        unimplemented!()
    }

    async fn read_relationships(
        &self,
        _: Request<ReadRelationshipsRequest>,
    ) -> Result<Response<Self::ReadRelationshipsStream>, Status> {
        unimplemented!()
    }

    /// Applies each update to the shared store: `Touch`/`Create` inserts the
    /// relationship, `Delete` removes it.
    async fn write_relationships(
        &self,
        request: Request<WriteRelationshipsRequest>,
    ) -> Result<Response<WriteRelationshipsResponse>, Status> {
        let mut store = self.store.lock().expect("relationship store poisoned");
        for update in request.into_inner().updates {
            let Some(rel) = update.relationship.as_ref().and_then(relationship_key) else {
                continue;
            };
            match Operation::try_from(update.operation) {
                Ok(Operation::Touch) | Ok(Operation::Create) => {
                    store.insert(rel);
                }
                Ok(Operation::Delete) => {
                    store.remove(&rel);
                }
                _ => {}
            }
        }

        Ok(Response::new(WriteRelationshipsResponse {
            written_at: Some(ZedToken::default()),
        }))
    }
}

impl SpiceDB {
    /// A recording in-process SpiceDB for tests. Returns a connected client and
    /// the shared store that its writes and deletes mutate.
    pub async fn recording() -> (Self, RelationshipStore) {
        let (server, store) = RecordingSpiceDBServer::new();
        let channel = server.serve().await;
        (Self::new(channel, String::new()), store)
    }

    /// An in-process SpiceDB that always denies permission checks.
    pub async fn denying() -> Self {
        let channel = SpiceDBServer::denying().serve().await;
        Self::new(channel, String::new())
    }
}
