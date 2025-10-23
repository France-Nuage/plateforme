use crate::api::v1::{
    CheckBulkPermissionsRequest, CheckBulkPermissionsResponse, CheckPermissionRequest,
    CheckPermissionResponse, DeleteRelationshipsRequest, DeleteRelationshipsResponse,
    ExpandPermissionTreeRequest, ExpandPermissionTreeResponse, LookupResourcesRequest,
    LookupResourcesResponse, LookupSubjectsRequest, LookupSubjectsResponse,
    ReadRelationshipsRequest, ReadRelationshipsResponse, WriteRelationshipsRequest,
    WriteRelationshipsResponse, ZedToken,
    check_permission_response::Permissionship,
    permissions_service_server::{PermissionsService, PermissionsServiceServer},
};
use futures::Stream;
use std::pin::Pin;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{
    Request, Response, Status,
    transport::{Channel, Server},
};

pub struct SpiceDBServer {}

impl Default for SpiceDBServer {
    fn default() -> Self {
        Self::new()
    }
}

impl SpiceDBServer {
    pub fn new() -> Self {
        Self {}
    }
}

impl SpiceDBServer {
    pub async fn serve(self) -> Channel {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("cannot reserve a port");
        let addr = listener
            .local_addr()
            .expect("cannot convert to a local address");

        tokio::spawn(async move {
            Server::builder()
                .add_service(PermissionsServiceServer::new(self))
                .serve_with_incoming(TcpListenerStream::new(listener))
                .await
                .expect("could not serve the mock spicedb server");
        });

        Channel::from_shared(format!("http://{}", addr))
            .expect("could not create channel")
            .connect()
            .await
            .expect("could not connect to channel")
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
            permissionship: Permissionship::HasPermission as i32,
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
            resource_object_id: "00000000-0000-0000-0000-000000000000".to_owned(),
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
