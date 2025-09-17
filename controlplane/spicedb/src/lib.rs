pub mod google {
    pub mod rpc {
        tonic::include_proto!("google.rpc");
    }
}

mod authzed {
    pub mod api {
        pub mod v1 {
            tonic::include_proto!("authzed.api.v1");
        }
    }
}

pub use authzed::api;

#[cfg(feature = "mock")]
pub mod mock;

#[cfg(test)]
mod tests {
    use crate::api::v1::{
        CheckPermissionRequest, Consistency, ObjectReference, SubjectReference,
        check_permission_response::Permissionship,
        permissions_service_client::PermissionsServiceClient,
    };
    use crate::mock::SpiceDBServer;
    use tonic::Request;

    #[tokio::test]
    async fn test_permissions_work() -> Result<(), Box<dyn std::error::Error>> {
        let request = CheckPermissionRequest {
            consistency: Some(Consistency {
                requirement: Some(crate::api::v1::consistency::Requirement::FullyConsistent(
                    true,
                )),
            }),
            context: None,
            permission: "view".to_owned(),
            resource: Some(ObjectReference {
                object_id: "foobar".to_owned(),
                object_type: "instance".to_owned(),
            }),
            subject: Some(SubjectReference {
                object: Some(ObjectReference {
                    object_id: "foobar".to_owned(),
                    object_type: "user".to_owned(),
                }),
                optional_relation: "".to_owned(),
            }),
            with_tracing: false,
        };
        let request = Request::new(request);

        let channel = SpiceDBServer::new().serve().await;

        let mut client = PermissionsServiceClient::new(channel);

        let response = client
            .check_permission(request)
            .await
            .expect("call did not succeed")
            .into_inner();
        assert_eq!(response.permissionship(), Permissionship::HasPermission);

        Ok(())
    }
}
