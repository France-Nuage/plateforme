//! SpiceDB-based authorization client for fine-grained access control.
//!
//! This module provides the `Authz` struct, which offers a fluent API for performing
//! authorization checks using SpiceDB's permission system. SpiceDB is a graph-based
//! authorization service that provides Google Zanzibar-style access control with
//! real-time consistency guarantees.
//!
//! ## Design Philosophy
//!
//! The authorization client follows a builder pattern for constructing permission checks:
//! - **Fluent API**: Chainable methods for readable authorization queries
//! - **Type Safety**: Compile-time validation of permission and resource types
//! - **Real-time Consistency**: Direct integration with SpiceDB for immediate results
//! - **Mock Support**: Built-in testing capabilities with mock SpiceDB server
//! - **Connection Management**: Efficient gRPC connection handling with reuse
//!
//! ## Usage Pattern
//!
//! Authorization checks follow the pattern: "Can [subject] perform [permission] on [resource]?"
//!
//! ```
//! # use auth::{Authz, Permission, model::User};
//! # async fn example() -> Result<(), auth::Error> {
//! # let authz = Authz::mock().await;
//! # let user = User::default();
//! let authorized = authz
//!     .can(&user)
//!     .perform(Permission::Get)
//!     .on("instance", "instance-123")
//!     .check()
//!     .await;
//!
//! match authorized {
//!     Ok(_) => println!("Access granted"),
//!     Err(_) => println!("Access denied"),
//! }
//! # Ok(())
//! # }
//! ```

use std::str::FromStr;

use crate::{Error, Permission, model::User};
use spicedb::{
    api::v1::{
        CheckPermissionRequest, Consistency, ObjectReference, SubjectReference,
        check_permission_response::Permissionship,
        permissions_service_client::PermissionsServiceClient,
    },
    mock::SpiceDBServer,
};
use tonic::{Request, metadata::MetadataValue, transport::Channel};
use uuid::Uuid;

/// SpiceDB authorization client with fluent API for permission checking.
///
/// `Authz` provides a builder-pattern interface for constructing and executing
/// authorization queries against a SpiceDB server. Each authorization check
/// follows the pattern: "Can [subject] perform [permission] on [resource]?"
///
/// ## Connection Management
///
/// The client maintains a persistent gRPC connection to the SpiceDB server,
/// allowing for efficient reuse across multiple authorization checks. The
/// connection is established once during client creation and reused for all
/// subsequent operations.
///
/// ## Thread Safety
///
/// `Authz` instances are thread-safe and can be cloned cheaply for use across
/// multiple concurrent operations. Each clone shares the same underlying gRPC
/// connection, making it efficient for use in multi-threaded environments.
///
/// ## Examples
///
/// ### Basic Authorization Check
///
/// ```
/// # use auth::{Authz, Permission, model::User};
/// # async fn example() -> Result<(), auth::Error> {
/// let authz = Authz::connect("http://spicedb:50051".to_owned(), "Bearer f00ba3".to_string()).await?;
/// let user = User::default();
///
/// authz
///     .can(&user)
///     .perform(Permission::Get)
///     .on("instance", "my-instance")
///     .check()
///     .await?;
///
/// println!("User authorized to list instances");
/// # Ok(())
/// # }
/// ```
///
/// ### Using Mock Server for Testing
///
/// ```
/// # use auth::{Authz, Permission, model::User};
/// # async fn example() -> Result<(), auth::Error> {
/// let authz = Authz::mock().await;
/// let user = User::default();
///
/// // Mock server allows all permissions by default
/// let result = authz
///     .can(&user)
///     .perform(Permission::Get)
///     .on("instance", "test-instance")
///     .check()
///     .await;
///
/// assert!(result.is_ok());
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Authz {
    client: PermissionsServiceClient<Channel>,
    consistency: Option<Consistency>,
    permission: Option<String>,
    resource: Option<ObjectReference>,
    subject: Option<SubjectReference>,
    token: Option<String>,
}

impl Authz {
    /// Creates a new authorization client with the provided gRPC channel.
    ///
    /// This constructor initializes an `Authz` client with a pre-established
    /// gRPC channel to a SpiceDB server. The client will reuse this connection
    /// for all authorization requests.
    ///
    /// # Arguments
    ///
    /// * `channel` - A connected gRPC channel to the SpiceDB server
    ///
    /// # Examples
    ///
    /// ```
    /// # use tonic::transport::Channel;
    /// # use auth::Authz;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let channel = Channel::from_static("http://spicedb:50051")
    ///     .connect()
    ///     .await?;
    ///
    /// let authz = Authz::new(channel, None);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(channel: Channel, token: Option<String>) -> Self {
        let client = PermissionsServiceClient::new(channel);
        Self {
            client,
            consistency: None,
            permission: None,
            resource: None,
            subject: None,
            token,
        }
    }

    /// Sets the subject (user) for the authorization check.
    ///
    /// This method configures the authorization check to validate permissions
    /// for the specified user. The user's ID is extracted and used as the
    /// subject identifier in the SpiceDB permission check.
    ///
    /// # Arguments
    ///
    /// * `user` - The user whose permissions should be checked
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining in the builder pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// # use auth::{Authz, model::User};
    /// # async fn example() -> Result<(), auth::Error> {
    /// # let authz = Authz::mock().await;
    /// let user = User::default();
    ///
    /// let authz = authz.can(&user);
    /// # Ok(())
    /// # }
    /// ```
    pub fn can(self, user: &User) -> Self {
        self.with_subject(&user.id.to_string(), "user")
    }

    /// Sets the permission to check for the authorization request.
    ///
    /// This method specifies which permission should be validated in the
    /// authorization check. The permission is converted to its string
    /// representation as defined in the SpiceDB schema.
    ///
    /// # Arguments
    ///
    /// * `permission` - The permission to validate (e.g., `Permission::Get`)
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining in the builder pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// # use auth::{Authz, Permission};
    /// # async fn example() -> Result<(), auth::Error> {
    /// # let authz = Authz::mock().await;
    /// let authz = authz.perform(Permission::Get);
    /// # Ok(())
    /// # }
    /// ```
    pub fn perform(self, permission: Permission) -> Self {
        self.with_permission(permission.to_string())
    }

    /// Sets the resource target for the authorization check.
    ///
    /// This method specifies the resource that the permission check should
    /// be performed against. Resources are identified by their type and
    /// unique identifier within that type.
    ///
    /// # Arguments
    ///
    /// * `resource_type` - The type of resource (e.g., "instance", "organization")
    /// * `resource_id` - The unique identifier for the specific resource
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining in the builder pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// # use auth::Authz;
    /// # async fn example() -> Result<(), auth::Error> {
    /// # let authz = Authz::mock().await;
    /// let authz = authz.on("instance", "i-1234567890abcdef0");
    /// # Ok(())
    /// # }
    /// ```
    pub fn on(self, resource: (&'static str, &Uuid)) -> Self {
        let (resource_type, resource_id) = resource;
        self.with_resource(resource_type, &resource_id.to_string())
    }

    /// Executes the authorization check against the SpiceDB server.
    ///
    /// This method performs the actual permission check by sending a request
    /// to the SpiceDB server with the configured subject, permission, and resource.
    /// It returns `Ok(())` if the permission is granted, or an error if denied
    /// or if the check fails.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The subject has the specified permission on the resource
    /// * `Err(Error::Forbidden)` - The subject does not have the permission
    /// * `Err(Error::UnspecifiedPermission)` - No permission was set before calling check
    /// * `Err(Error::UnspecifiedResource)` - No resource was set before calling check
    /// * `Err(Error::UnspecifiedSubject)` - No subject was set before calling check
    /// * `Err(Error::AuthorizationServerError)` - SpiceDB server communication failed
    /// * `Err(Error::Internal)` - Unsupported permission types (Conditional/Unspecified)
    ///
    /// # Examples
    ///
    /// ```
    /// # use auth::{Authz, Permission, model::User};
    /// # async fn example() -> Result<(), auth::Error> {
    /// let authz = Authz::mock().await;
    /// let user = User::default();
    ///
    /// match authz
    ///     .can(&user)
    ///     .perform(Permission::Get)
    ///     .on("instance", "my-instance")
    ///     .check()
    ///     .await
    /// {
    ///     Ok(_) => println!("Access granted"),
    ///     Err(auth::Error::Forbidden) => println!("Access denied"),
    ///     Err(e) => println!("Check failed: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn check(mut self) -> Result<(), Error> {
        // prepare the request
        let mut request = Request::new(CheckPermissionRequest {
            consistency: self.consistency,
            context: None,
            permission: self.permission.ok_or(Error::UnspecifiedPermission)?,
            resource: Some(self.resource.ok_or(Error::UnspecifiedResource)?),
            subject: Some(self.subject.ok_or(Error::UnspecifiedSubject)?),
            with_tracing: false,
        });

        // inject the authorization token if present
        if let Some(token) = &self.token {
            let value = MetadataValue::from_str(&format!("bearer {}", token))
                .map_err(|_| Error::UnparsableAuthzToken)?;
            request.metadata_mut().insert("authorization", value);
        }

        // Perform the request and extract the permissionship in the response
        let permissionship = self
            .client
            .check_permission(request)
            .await
            .map_err(|err| Error::AuthorizationServerError(err.message().to_owned()))?
            .into_inner()
            .permissionship();

        // check the permissionship
        match permissionship {
            Permissionship::HasPermission => Ok(()),
            Permissionship::NoPermission => Err(Error::Forbidden),
            Permissionship::Unspecified => Err(Error::Internal(
                "Permissionship::Unspecified is not implemented".to_owned(),
            )),
            Permissionship::ConditionalPermission => Err(Error::Internal(
                "Permissionship::ConditionalPermission is not implemented".to_owned(),
            )),
        }
    }

    /// Connects to a SpiceDB server and creates a new authorization client.
    ///
    /// This method establishes a gRPC connection to the specified SpiceDB server
    /// and returns a configured `Authz` client ready for use. The connection
    /// is persistent and will be reused for all authorization requests.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the SpiceDB server (e.g., "http://spicedb:50051")
    ///
    /// # Returns
    ///
    /// * `Ok(Authz)` - Successfully connected authorization client
    /// * `Err(Error::UnreachableAuthzServer)` - Failed to connect to the server
    /// * `Err(Error)` - Invalid URL format or other connection errors
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use auth::Authz;
    /// # async fn example() -> Result<(), auth::Error> {
    /// let authz = Authz::connect("http://spicedb.example.com:50051".to_owned(), "Bearer f00ba3".to_owned()).await?;
    /// println!("Connected to SpiceDB server");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(url: String, preshared_key: String) -> Result<Self, Error> {
        let channel = Channel::from_shared(url.clone())?
            .connect()
            .await
            .map_err(|_| Error::UnreachableAuthzServer(url))?;

        Ok(Authz::new(channel, Some(preshared_key)))
    }

    /// Creates a mock SpiceDB server for testing and returns a connected client.
    ///
    /// This method starts an embedded SpiceDB mock server and returns a client
    /// connected to it. The mock server is designed for testing and allows all
    /// permission checks by default, making it suitable for unit tests where
    /// authorization logic needs to be bypassed.
    ///
    /// # Returns
    ///
    /// A connected `Authz` client using the mock SpiceDB server.
    ///
    /// # Examples
    ///
    /// ```
    /// # use auth::{Authz, Permission, model::User};
    /// # async fn test_authorization() {
    /// let authz = Authz::mock().await;
    /// let user = User::default();
    ///
    /// // Mock server allows all permissions
    /// let result = authz
    ///     .can(&user)
    ///     .perform(Permission::Get)
    ///     .on("instance", "test-instance")
    ///     .check()
    ///     .await;
    ///
    /// assert!(result.is_ok());
    /// # }
    /// ```
    pub async fn mock() -> Self {
        let channel = SpiceDBServer::new().serve().await;

        Authz::new(channel, None)
    }

    fn with_resource(mut self, resource_type: &str, resource_id: &str) -> Self {
        self.resource = Some(ObjectReference {
            object_id: resource_id.to_owned(),
            object_type: resource_type.to_owned(),
        });

        self
    }

    fn with_subject(mut self, object_id: &str, object_type: &str) -> Self {
        self.subject = Some(SubjectReference {
            object: Some(ObjectReference {
                object_id: object_id.to_owned(),
                object_type: object_type.to_owned(),
            }),
            optional_relation: "".to_owned(),
        });

        self
    }

    fn with_permission(mut self, permission: String) -> Self {
        self.permission = Some(permission);

        self
    }
}

#[cfg(test)]
mod tests {
    use crate::Authorize;

    use super::*;
    use uuid::Uuid;

    struct Anvil {
        id: Uuid,
    }

    impl Authorize for Anvil {
        type Id = Uuid;

        fn any_resource() -> (&'static str, &'static str) {
            ("anvil", "*")
        }

        fn resource(&self) -> (&'static str, &Self::Id) {
            ("anvil", &self.id)
        }

        fn resource_name() -> &'static str {
            "anvil"
        }
    }

    #[tokio::test]
    async fn test_the_check_function_works() {
        let authz = Authz::mock().await;
        let anvil = Anvil { id: Uuid::new_v4() };
        let user = User::default();

        let result = authz
            .can(&user)
            .perform(Permission::Get)
            .on(anvil.resource())
            .check()
            .await;

        assert!(result.is_ok());
    }
}
