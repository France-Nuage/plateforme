use crate::{
    Error,
    authorization::{Principal, Resource, check::WithPrincipal},
};
use spicedb::SpiceDB;

pub trait Authorize: Clone + Send + Sync {
    fn _check(
        &mut self,
        subject_type: String,
        subject_id: String,
        permission: String,
        resource_type: String,
        resource_id: String,
    ) -> impl Future<Output = Result<(), Error>> + Send;

    fn check<'a, P: Principal, R: Resource>(&self, id: &'a P::Id) -> WithPrincipal<'a, Self, P, R> {
        WithPrincipal::new(self.clone(), id)
    }
}

impl Authorize for SpiceDB {
    async fn _check(
        &mut self,
        subject_type: String,
        subject_id: String,
        permission: String,
        resource_type: String,
        resource_id: String,
    ) -> Result<(), Error> {
        self.check_permission(
            (subject_type, subject_id),
            permission,
            (resource_type, resource_id),
        )
        .await
        .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{identity::ServiceAccount, resourcemanager::Organization};

    #[tokio::test]
    async fn test_check_works() {
        let auth = SpiceDB::mock().await;
        let principal = ServiceAccount::default();
        let resource = Organization::default();

        let result = auth
            .check::<ServiceAccount, Organization>(&principal.id)
            .perform(crate::authorization::Permission::Create)
            .over(&resource.id)
            .await;

        assert!(result.is_ok())
    }
}

// impl AuthorizationServer for SpiceDB {
//     async fn check(&mut self, request: AuthorizationRequest<'_>) -> Result<(), Error> {
//         let (principal_type, principal_id) = request.principal.resource();
//         let (resource_type, resource_id) = request.resource.resource();
//         self.check_permission(
//             (principal_type, principal_id),
//             request.permission.to_string(),
//             (resource_type, resource_id),
//         )
//         .await
//         .map_err(Into::into)
//     }
//
//     // async fn lookup(&mut self, request: AuthorizationRequest<'_>) -> Result<Vec<String>, Error> {
//     //     let (principal_type, principal_id) = request.principal.resource();
//     //     let (resource_type, _) = request.resource.resource();
//     //
//     //     self.lookup(
//     //         (principal_type, principal_id),
//     //         request.permission.to_string(),
//     //         resource_type,
//     //     )
//     //     .await
//     //     .map_err(Into::into)
//     // }
// }
