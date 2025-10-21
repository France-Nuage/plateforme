#![allow(dead_code)]
use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
use std::pin::Pin;

use crate::Error;
use crate::authorization::resource::Resource;
use crate::authorization::{Authorize, Permission, Principal};

pub struct WithPrincipal<'a, A: Authorize, P: Principal, R: Resource> {
    auth: A,
    subject_id: &'a P::Id,
    resource: PhantomData<R>,
}

pub struct WithPermission<'a, A: Authorize, P: Principal, R: Resource> {
    auth: A,
    subject_id: &'a P::Id,
    permission: Permission,
    resource: PhantomData<R>,
}

pub struct CheckRequest<'a, A: Authorize, P: Principal, R: Resource> {
    auth: A,
    principal: &'a P::Id,
    permission: Permission,
    resource_id: &'a R::Id,
}

impl<'a, A: Authorize, P: Principal, R: Resource> WithPrincipal<'a, A, P, R> {
    pub fn new(auth: A, subject_id: &'a P::Id) -> Self {
        Self {
            auth,
            subject_id,
            resource: PhantomData,
        }
    }

    pub fn perform(self, permission: Permission) -> WithPermission<'a, A, P, R> {
        WithPermission {
            auth: self.auth,
            subject_id: self.subject_id,
            permission,
            resource: PhantomData,
        }
    }
}

impl<'a, A: Authorize, P: Principal, R: Resource> WithPermission<'a, A, P, R> {
    pub fn over(self, resource_id: &'a R::Id) -> CheckRequest<'a, A, P, R> {
        CheckRequest {
            auth: self.auth,
            principal: self.subject_id,
            permission: self.permission,
            resource_id,
        }
    }
}

impl<'a, A: Authorize, P: Principal, R: Resource> CheckRequest<'a, A, P, R> {
    pub fn can(auth: A, subject_id: &P::Id) -> WithPrincipal<'_, A, P, R> {
        WithPrincipal {
            auth,
            subject_id,
            resource: PhantomData,
        }
    }
}

impl<'a, A: Authorize + 'a, P: Principal, R: Resource> IntoFuture for CheckRequest<'a, A, P, R> {
    type Output = Result<(), Error>;

    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        let mut auth = self.auth.clone();
        Box::pin(async move {
            auth._check(
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
        CheckRequest::<SpiceDB, User, Anvil>::can(auth, &user.id)
            .perform(Permission::Create)
            .over(anvil.id());
    }
}
