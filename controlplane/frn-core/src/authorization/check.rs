#![allow(dead_code)]
use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
use std::pin::Pin;

use crate::Error;
use crate::authorization::resource::Resource;
use crate::authorization::{Authorize, Permission, Principal};

/// Typestate after specifying the principal
pub struct CheckWithPrincipal<'a, A: Authorize, P: Principal, R: Resource> {
    auth: A,
    resource: PhantomData<R>,
    subject_id: &'a P::Id,
}

/// Typestate after specifying the permission
pub struct CheckWithPermission<'a, A: Authorize, P: Principal, R: Resource> {
    auth: A,
    permission: Permission,
    resource: PhantomData<R>,
    subject_id: &'a P::Id,
}

/// Typestate with all parameters set, ready to execute the authorization check
pub struct CheckWithResource<'a, A: Authorize, P: Principal, R: Resource> {
    auth: A,
    principal: &'a P::Id,
    permission: Permission,
    resource_id: &'a R::Id,
}

impl<'a, A: Authorize, P: Principal, R: Resource> CheckWithPrincipal<'a, A, P, R> {
    pub fn new(auth: A, subject_id: &'a P::Id) -> Self {
        Self {
            auth,
            subject_id,
            resource: PhantomData,
        }
    }

    /// Specify the permission to check
    pub fn perform(self, permission: Permission) -> CheckWithPermission<'a, A, P, R> {
        CheckWithPermission {
            auth: self.auth,
            subject_id: self.subject_id,
            permission,
            resource: PhantomData,
        }
    }
}

impl<'a, A: Authorize, P: Principal, R: Resource> CheckWithPermission<'a, A, P, R> {
    /// Specify the resource to check permission against
    pub fn over(self, resource_id: &'a R::Id) -> CheckWithResource<'a, A, P, R> {
        CheckWithResource {
            auth: self.auth,
            principal: self.subject_id,
            permission: self.permission,
            resource_id,
        }
    }
}

impl<'a, A: Authorize, P: Principal, R: Resource> CheckWithResource<'a, A, P, R> {
    pub fn can(auth: A, subject_id: &P::Id) -> CheckWithPrincipal<'_, A, P, R> {
        CheckWithPrincipal {
            auth,
            subject_id,
            resource: PhantomData,
        }
    }
}

impl<'a, A: Authorize + 'a, P: Principal, R: Resource> IntoFuture
    for CheckWithResource<'a, A, P, R>
{
    type Output = Result<(), Error>;

    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(mut self) -> Self::IntoFuture {
        Box::pin(async move {
            self.auth
                ._check(
                    P::NAME.to_string(),
                    self.principal.to_string(),
                    self.permission.to_string(),
                    R::NAME.to_string(),
                    self.resource_id.to_string(),
                )
                .await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::User;
    use spicedb::SpiceDB;
    use uuid::Uuid;

    struct Anvil {
        id: Uuid,
    }

    impl Resource for Anvil {
        type Id = Uuid;

        const NAME: &'static str = "anvils";

        fn id(&self) -> &Self::Id {
            &self.id
        }
    }

    #[tokio::test]
    async fn test() {
        let auth = SpiceDB::mock().await;
        let user = User::default();
        let anvil = Anvil { id: Uuid::new_v4() };
        CheckWithResource::<SpiceDB, User, Anvil>::can(auth, &user.id)
            .perform(Permission::Create)
            .over(anvil.id());
    }
}
