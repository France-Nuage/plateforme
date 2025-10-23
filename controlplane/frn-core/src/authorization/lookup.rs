use std::{
    future::{Future, IntoFuture},
    marker::PhantomData,
    pin::Pin,
};

use database::Persistable;

use crate::{
    Error,
    authorization::{Authorize, Permission, Principal, Resource},
};

/// Typestate after specifying the resource type
pub struct LookupWithResource<A: Authorize, R: Resource + Persistable> {
    auth: A,
    resource: PhantomData<R>,
}

/// Typestate after specifying the principal
pub struct LookupWithPrincipal<'a, A: Authorize, P: Principal, R: Resource + Persistable> {
    auth: A,
    resource: PhantomData<R>,
    subject_id: &'a P::Id,
    subject_type: &'static str,
}

/// Typestate after specifying the permission
pub struct LookupWithPermission<'a, A: Authorize, P: Principal, R: Resource + Persistable> {
    auth: A,
    permission: Permission,
    resource: PhantomData<R>,
    subject_id: &'a P::Id,
    subject_type: &'static str,
}

/// Typestate with all parameters set, ready to execute the resource lookup
pub struct LookupWithConnection<'a, A: Authorize, P: Principal, R: Resource + Persistable> {
    auth: A,
    connection: &'a R::Connection,
    permission: Permission,
    resource: PhantomData<R>,
    subject_id: &'a P::Id,
    subject_type: &'static str,
}

impl<A: Authorize, R: Resource + Persistable> LookupWithResource<A, R> {
    pub fn new(prev: A) -> Self {
        Self {
            auth: prev,
            resource: PhantomData,
        }
    }

    /// Specify the principal performing the lookup
    pub fn on_behalf_of<'a, P: Principal>(
        self,
        principal: &'a P,
    ) -> LookupWithPrincipal<'a, A, P, R> {
        LookupWithPrincipal::new(self, principal.name(), principal.id())
    }
}

impl<'a, A: Authorize, P: Principal, R: Resource + Persistable> LookupWithPrincipal<'a, A, P, R> {
    pub fn new(
        prev: LookupWithResource<A, R>,
        subject_type: &'static str,
        subject_id: &'a P::Id,
    ) -> Self {
        Self {
            auth: prev.auth,
            resource: prev.resource,
            subject_id,
            subject_type,
        }
    }

    /// Specify the permission to filter by
    pub fn with(self, permission: Permission) -> LookupWithPermission<'a, A, P, R> {
        LookupWithPermission::new(self, permission)
    }
}
impl<'a, A: Authorize, P: Principal, R: Resource + Persistable> LookupWithPermission<'a, A, P, R> {
    pub fn new(prev: LookupWithPrincipal<'a, A, P, R>, permission: Permission) -> Self {
        Self {
            auth: prev.auth,
            resource: prev.resource,
            subject_id: prev.subject_id,
            subject_type: prev.subject_type,
            permission,
        }
    }

    /// Specify the database connection to query resources from
    pub fn against(self, connection: &'a R::Connection) -> LookupWithConnection<'a, A, P, R> {
        LookupWithConnection::new(self, connection)
    }
}

impl<'a, A: Authorize, P: Principal, R: Resource + Persistable> LookupWithConnection<'a, A, P, R> {
    pub fn new(prev: LookupWithPermission<'a, A, P, R>, connection: &'a R::Connection) -> Self {
        Self {
            auth: prev.auth,
            permission: prev.permission,
            resource: prev.resource,
            subject_id: prev.subject_id,
            subject_type: prev.subject_type,
            connection,
        }
    }
}

impl<'a, A: Authorize + 'a, P: Principal, R: Resource + Persistable> IntoFuture
    for LookupWithConnection<'a, A, P, R>
{
    type Output = Result<Vec<R>, Error>;

    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(mut self) -> Self::IntoFuture {
        Box::pin(async move {
            let ids = self
                .auth
                ._lookup(
                    self.subject_type.to_string(),
                    self.subject_id.to_string(),
                    self.permission.to_string(),
                    R::NAME.to_string(),
                )
                .await?;

            let models = R::list(self.connection)
                .await
                .map_err(|_| Error::Other("persistence layer failure".to_owned()))?
                .into_iter()
                .filter(|resource| ids.contains(&resource.id().to_string()))
                .collect();

            Ok(models)
        })
    }
}
