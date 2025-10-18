use std::pin::Pin;

use crate::Error;
use crate::authorization::{Permission, Principal};
use spicedb::SpiceDB;

/// Represents an authorizable resource in the system.
///
/// Resources are entities that can have permissions checked against them.
/// Each resource has a name (type) and an identifier (instance).
pub trait Resource: Sync {
    /// Returns the resource type and instance identifier as a tuple.
    fn resource(&self) -> (String, String);

    /// Constructs a resource representation for the given instance ID.
    fn some(id: impl ToString) -> Box<dyn Resource + Send + Sync>
    where
        Self: Sized;
}

impl Resource for Box<dyn Resource + Send + Sync> {
    fn resource(&self) -> (String, String) {
        (**self).resource()
    }

    fn some(_id: impl ToString) -> Box<dyn Resource + Send + Sync>
    where
        Self: Sized,
    {
        unreachable!("Box<dyn Resource>::some should never be called")
    }
}

/// Represents a permission check query to the authorization server.
///
/// Generic over `P` (principal) to support fluent API for checking permissions
/// across different principal types (users, service accounts).
pub struct AuthorizationRequest<'a> {
    permission: Permission,
    principal: &'a dyn Principal,
    resource: &'a dyn Resource,
}

impl<'a> AuthorizationRequest<'a> {
    pub fn new(
        principal: &'a dyn Principal,
        permission: Permission,
        resource: &'a dyn Resource,
    ) -> Self {
        Self {
            permission,
            principal,
            resource,
        }
    }
}

/// Builder for constructing authorization checks.
pub struct AuthorizationBuilder<'a, S: AuthorizationServer> {
    server: &'a mut S,
    principal: Option<&'a dyn Principal>,
    permission: Option<Permission>,
    resource: Option<&'a dyn Resource>,
}

impl<'a, S: AuthorizationServer> AuthorizationBuilder<'a, S> {
    fn new(server: &'a mut S) -> Self {
        Self {
            server,
            principal: None,
            permission: None,
            resource: None,
        }
    }

    /// Sets the principal for the authorization check.
    pub fn principal(mut self, principal: &'a dyn Principal) -> Self {
        self.principal = Some(principal);
        self
    }

    /// Sets the permission for the authorization check.
    pub fn perform(mut self, permission: Permission) -> Self {
        self.permission = Some(permission);
        self
    }

    /// Sets the resource for the authorization check.
    pub fn over(mut self, resource: &'a dyn Resource) -> Self {
        self.resource = Some(resource);
        self
    }

    /// Executes the authorization check.
    pub async fn check(self) -> Result<(), Error> {
        let principal = self.principal.ok_or(Error::UnspecifiedPrincipal)?;
        let permission = self.permission.ok_or(Error::UnspecifiedPermission)?;
        let resource = self.resource.ok_or(Error::UnspecifiedResource)?;
        let request = AuthorizationRequest::new(principal, permission, resource);
        self.server.check(request).await
    }

    /// Returns all resources of the given type that the principal has the requested permission on.
    pub async fn lookup(self) -> Result<Vec<String>, Error> {
        let principal = self.principal.ok_or(Error::UnspecifiedPrincipal)?;
        let permission = self.permission.ok_or(Error::UnspecifiedPermission)?;
        let resource = self.resource.ok_or(Error::UnspecifiedResource)?;
        let request = AuthorizationRequest::new(principal, permission, resource);
        self.server.lookup(request).await
    }
}

/// Enables awaiting the builder at any step to execute the authorization check.
///
/// This allows flexible fluent API usage where `.await` can be called after any builder method:
/// - `.authorize().principal(p).await?` - check with just principal
/// - `.authorize().principal(p).perform(perm).await?` - check with principal and permission
/// - `.authorize().principal(p).perform(perm).over(r).await?` - full check
impl<'a, S: AuthorizationServer> IntoFuture for AuthorizationBuilder<'a, S> {
    type Output = Result<(), Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let principal = self.principal.ok_or(Error::UnspecifiedPrincipal)?;
            let permission = self.permission.ok_or(Error::UnspecifiedPermission)?;
            let resource = self.resource.ok_or(Error::UnspecifiedResource)?;
            let request = AuthorizationRequest::new(principal, permission, resource);
            self.server.check(request).await
        })
    }
}

/// Authorization server that can check permissions.
///
/// Abstraction over authorization backends for checking if principals
/// have permissions on resources.
pub trait AuthorizationServer: Clone + Send + Sync {
    fn can<'a>(&'a mut self, principal: &'a dyn Principal) -> AuthorizationBuilder<'a, Self>
    where
        Self: Sized,
    {
        AuthorizationBuilder::new(self).principal(principal)
    }

    /// Checks if the principal has permission to perform the action on the specific resource.
    fn check(
        &mut self,
        request: AuthorizationRequest<'_>,
    ) -> impl Future<Output = Result<(), Error>> + Send;

    /// Returns all resources of the given type that the principal has the requested permission on.
    fn lookup(
        &mut self,
        request: AuthorizationRequest<'_>,
    ) -> impl Future<Output = Result<Vec<String>, Error>> + Send;
}

impl AuthorizationServer for SpiceDB {
    async fn check(&mut self, request: AuthorizationRequest<'_>) -> Result<(), Error> {
        let (principal_type, principal_id) = request.principal.resource();
        let (resource_type, resource_id) = request.resource.resource();
        self.check_permission(
            (principal_type, principal_id),
            request.permission.to_string(),
            (resource_type, resource_id),
        )
        .await
        .map_err(Into::into)
    }

    async fn lookup(&mut self, request: AuthorizationRequest<'_>) -> Result<Vec<String>, Error> {
        let (principal_type, principal_id) = request.principal.resource();
        let (resource_type, _) = request.resource.resource();

        self.lookup(
            (principal_type, principal_id),
            request.permission.to_string(),
            resource_type,
        )
        .await
        .map_err(Into::into)
    }
}
