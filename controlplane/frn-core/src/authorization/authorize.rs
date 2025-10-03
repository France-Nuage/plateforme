use std::pin::Pin;

use crate::Error;
use crate::authorization::{Permission, Principal};
use spicedb::SpiceDB;

/// Represents an authorizable resource in the system.
///
/// Resources are entities that can have permissions checked against them.
/// Each resource has a name (type) and an identifier (instance).
pub trait Resource {
    type Id: ToString;
    const NAME: &'static str;

    fn any() -> impl Resource<Id = String>;

    fn resource_identifier(&self) -> (&'static str, &Self::Id);
}

/// Represents a permission check query to the authorization server.
///
/// Generic over `P` (principal) and `R` (resource) to support fluent API for checking
/// permissions across different principal types (users, service accounts) and resources.
pub struct AuthorizationRequest<'a, P: Principal, R: Resource> {
    permission: Permission,
    principal: &'a P,
    resource: &'a R,
}

impl<'a, P: Principal, R: Resource> AuthorizationRequest<'a, P, R> {
    pub fn new(principal: &'a P, permission: Permission, resource: &'a R) -> Self {
        Self {
            permission,
            principal,
            resource,
        }
    }
}

/// Builder for constructing authorization checks.
pub struct AuthorizationBuilder<'a, P: Principal, R: Resource, S: AuthorizationServer> {
    server: &'a mut S,
    principal: Option<&'a P>,
    permission: Option<Permission>,
    resource: Option<&'a R>,
}

impl<'a, P: Principal + Sync, R: Resource + Sync, S: AuthorizationServer>
    AuthorizationBuilder<'a, P, R, S>
{
    fn new(server: &'a mut S) -> Self {
        Self {
            server,
            principal: None,
            permission: None,
            resource: None,
        }
    }

    /// Sets the principal for the authorization check.
    pub fn principal(mut self, principal: &'a P) -> Self {
        self.principal = Some(principal);
        self
    }

    /// Sets the permission for the authorization check.
    pub fn perform(mut self, permission: Permission) -> Self {
        self.permission = Some(permission);
        self
    }

    /// Sets the resource for the authorization check.
    pub fn over(mut self, resource: &'a R) -> Self {
        self.resource = Some(resource);
        self
    }

    /// Executes the authorization check.
    pub async fn execute(self) -> Result<(), Error> {
        let request = AuthorizationRequest::new(
            self.principal.unwrap(),
            self.permission.unwrap(),
            self.resource.unwrap(),
        );
        self.server.check(request).await
    }
}

/// Enables awaiting the builder at any step to execute the authorization check.
///
/// This allows flexible fluent API usage where `.await` can be called after any builder method:
/// - `.authorize().principal(p).await?` - check with just principal
/// - `.authorize().principal(p).perform(perm).await?` - check with principal and permission
/// - `.authorize().principal(p).perform(perm).over(r).await?` - full check
impl<'a, P: Principal + Sync, R: Resource + Sync, S: AuthorizationServer> IntoFuture
    for AuthorizationBuilder<'a, P, R, S>
{
    type Output = Result<(), Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let request = AuthorizationRequest::new(
                self.principal.unwrap(),
                self.permission.unwrap(),
                self.resource.unwrap(),
            );
            self.server.check(request).await
        })
    }
}

/// Authorization server that can check permissions.
///
/// Abstraction over authorization backends for checking if principals
/// have permissions on resources.
pub trait AuthorizationServer: Clone + Send {
    fn can<'a, P: Principal + Sync, R: Resource + Sync>(
        &'a mut self,
        principal: &'a P,
    ) -> AuthorizationBuilder<'a, P, R, Self>
    where
        Self: Sized,
    {
        AuthorizationBuilder::new(self).principal(principal)
    }

    fn check<P: Principal + Sync, R: Resource + Sync>(
        &mut self,
        request: AuthorizationRequest<'_, P, R>,
    ) -> impl Future<Output = Result<(), Error>> + Send;
}

impl AuthorizationServer for SpiceDB {
    async fn check<P: Principal, R: Resource>(
        &mut self,
        request: AuthorizationRequest<'_, P, R>,
    ) -> Result<(), Error> {
        let (principal_type, principal_id) = request.principal.resource_identifier();
        let (resource_type, resource_id) = request.resource.resource_identifier();
        self.check_permission(
            (principal_type.to_string(), principal_id.to_string()),
            request.permission.to_string(),
            (resource_type.to_string(), resource_id.to_string()),
        )
        .await
        .map_err(Into::into)
    }
}
