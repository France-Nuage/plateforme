use crate::Error;
use crate::authorization::lookup::LookupWithResource;
use crate::authorization::{Principal, Resource, check::CheckWithPrincipal};
use crate::authorization::{Relationship, Zookie};
use fabrique::Persistable;
use spicedb::SpiceDB;

/// Authorization backend trait for checking and looking up permissions
pub trait Authorize: Clone + Send + Sync {
    /// Internal method to check if a subject has permission on a resource
    fn _check(
        &mut self,
        subject_type: String,
        subject_id: String,
        permission: String,
        resource_type: String,
        resource_id: String,
    ) -> impl Future<Output = Result<(), Error>> + Send;

    /// Internal method to lookup resources a subject has permission on
    fn _lookup(
        &mut self,
        subject_type: String,
        subject_id: String,
        permission: String,
        resource_type: String,
    ) -> impl Future<Output = Result<Vec<String>, Error>> + Send;

    fn write_relationship(
        &mut self,
        relationship: &Relationship,
    ) -> impl Future<Output = Result<Option<Zookie>, Error>>;

    /// Start a permission check with a principal
    fn can<'a, P: Principal>(&self, principal: &'a P) -> CheckWithPrincipal<'a, Self, P> {
        CheckWithPrincipal::new(self.clone(), principal)
    }

    /// Start a resource lookup query
    fn lookup<R: Resource + Persistable>(&self) -> LookupWithResource<Self, R> {
        LookupWithResource::new(self.clone())
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

    async fn _lookup(
        &mut self,
        subject_type: String,
        subject_id: String,
        permission: String,
        resource_type: String,
    ) -> Result<Vec<String>, Error> {
        tracing::info!(
            "looking up permission '{}' over resource '{}' for principal '{}'@'{}'",
            permission,
            resource_type,
            subject_type,
            subject_id
        );
        self.lookup((subject_type, subject_id), permission, resource_type)
            .await
            .map_err(Into::into)
    }

    async fn write_relationship(
        &mut self,
        relationship: &Relationship,
    ) -> Result<Option<Zookie>, Error> {
        self.write_relationship(
            relationship.subject_type.clone(),
            relationship.subject_id.clone(),
            relationship.relation.to_string(),
            relationship.object_type.clone(),
            relationship.object_id.clone(),
        )
        .await
        .map(|token| token.map(Into::into))
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
            .can(&principal)
            .perform(crate::authorization::Permission::Create)
            .over::<Organization>(&resource.id)
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
