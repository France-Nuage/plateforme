#![allow(dead_code)]
use std::future::{Future, IntoFuture};
use std::pin::Pin;

use crate::Error;
use crate::authorization::resource::Resource;
use crate::authorization::{Authorize, Permission, Principal};

/// Typestate after specifying the principal
pub struct CheckWithPrincipal<'a, A: Authorize, P: Principal> {
    auth: A,
    subject_id: &'a P::Id,
    subject_type: &'static str,
}

/// Typestate after specifying the permission
pub struct CheckWithPermission<'a, A: Authorize, P: Principal> {
    auth: A,
    permission: Permission,
    subject_id: &'a P::Id,
    subject_type: &'static str,
}

/// Typestate with all parameters set, ready to execute the authorization check
pub struct CheckWithResource<'a, A: Authorize, P: Principal, R: Resource> {
    auth: A,
    permission: Permission,
    resource_id: &'a R::Id,
    subject_id: &'a P::Id,
    subject_type: &'static str,
}

impl<'a, A: Authorize, P: Principal> CheckWithPrincipal<'a, A, P> {
    pub fn new(auth: A, principal: &'a P) -> Self {
        Self {
            auth,
            subject_id: principal.id(),
            subject_type: principal.name(),
        }
    }

    /// Specify the permission to check
    pub fn perform(self, permission: Permission) -> CheckWithPermission<'a, A, P> {
        CheckWithPermission {
            auth: self.auth,
            permission,
            subject_id: self.subject_id,
            subject_type: self.subject_type,
        }
    }
}

impl<'a, A: Authorize, P: Principal> CheckWithPermission<'a, A, P> {
    /// Specify the resource to check permission against
    pub fn over<R: Resource>(self, resource_id: &'a R::Id) -> CheckWithResource<'a, A, P, R> {
        CheckWithResource {
            auth: self.auth,
            permission: self.permission,
            resource_id,
            subject_id: self.subject_id,
            subject_type: self.subject_type,
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
                    self.subject_type.to_string(),
                    self.subject_id.to_string(),
                    self.permission.to_string(),
                    R::RESOURCE_NAME.to_string(),
                    self.resource_id.to_string(),
                )
                .await
        })
    }
}
