//! Temporary database model for user authorization and access control.
//!
//! This module provides an interim data structure for managing user authorization
//! within organizations. This model serves as a transitional solution before
//! migrating to SpiceDB (Zanzibar-like authorization system) for stateless
//! access control.
//!
//! ## Key Features
//!
//! - **User-Organization Mapping**: Associates authenticated users with organizations
//! - **Access Rights Verification**: Enables organization-scoped authorization checks
//! - **Email-Based Lookup**: Retrieves user authorization context by email
//! - **Temporary Implementation**: Will be replaced by SpiceDB integration
//!
//! ## Design Philosophy
//!
//! This is an **interim authorization model** that:
//! - **Bridges Authentication to Authorization**: Links JWT-authenticated users to org access rights
//! - **Maintains Organization Boundaries**: Ensures users can only access their organization's resources  
//! - **Provides Transition Path**: Enables authorization logic before SpiceDB migration
//! - **Will Be Removed**: This table and model will be deleted once SpiceDB integration is complete
//!
//! ## Migration Path
//!
//! **Current State**: User authorization stored in controlplane database
//! **Future State**: Stateless authorization via SpiceDB queries
//! **Timeline**: This model is temporary pending SpiceDB integration
//!
//! ## Usage Pattern
//!
//! Used to verify that authenticated users have access to organization-scoped resources:
//!
//! ```
//! # use sqlx::PgPool;
//! # use auth::model::User;
//! # async fn example(pool: &PgPool, jwt_email: &str) -> Result<(), sqlx::Error> {
//! // Check if authenticated user has access to organization resources
//! let user = User::find_one_by_email(pool, jwt_email).await?;
//!
//! match user {
//!     Some(user) => {
//!         // User authorized for organization: user.organization_id
//!         println!("User authorized for org: {}", user.organization_id);
//!     },
//!     None => {
//!         // User not found - deny access
//!         println!("User not authorized");
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use database::Persistable;
use derive_factory::Factory;
use derive_repository::Repository;
use resources::organizations::OrganizationFactory;
use sqlx::{Postgres, types::chrono};
use uuid::Uuid;

/// Temporary user authorization model for organization access control.
///
/// This struct represents a user within the authorization system, primarily used
/// to associate authenticated users with organizations for access control purposes.
/// This is an interim solution that will be replaced by SpiceDB integration.
///
/// ## Database Mapping
///
/// Maps to the `users` table with the following schema:
/// - `id`: Primary key (UUID)
/// - `organization_id`: Foreign key to organizations table
/// - `email`: User's email address (used for JWT claim matching)
/// - `created_at`: Record creation timestamp
/// - `updated_at`: Last modification timestamp
///
/// ## Derives
///
/// - `Factory`: Enables test data generation
/// - `Repository`: Provides CRUD operations
/// - `FromRow`: SQLx automatic row mapping
#[derive(Debug, Default, Factory, PartialEq, Repository, sqlx::FromRow)]
pub struct User {
    /// Unique identifier for the user
    #[repository(primary)]
    pub id: Uuid,
    /// The organization this user is attached to
    #[factory(relation = "OrganizationFactory")]
    pub organization_id: Uuid,
    /// The user email
    pub email: String,
    // Creation time of the instance
    pub created_at: chrono::DateTime<chrono::Utc>,
    // Time of the instance last update
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    /// Finds a user by their email address for authorization lookup.
    ///
    /// This method is primarily used during the authorization flow to determine
    /// which organization a JWT-authenticated user belongs to. It performs an
    /// exact email match against the database.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool for executing the query
    /// * `email` - Email address from JWT claims or user input
    ///
    /// # Returns
    ///
    /// * `Ok(Some(User))` - User found with matching email
    /// * `Ok(None)` - No user found with the given email
    /// * `Err(sqlx::Error)` - Database query error
    ///
    /// # Examples
    ///
    /// ```
    /// # use sqlx::PgPool;
    /// # use auth::model::User;
    /// # async fn example(pool: &PgPool) -> Result<(), sqlx::Error> {
    /// let user = User::find_one_by_email(pool, "user@example.com").await?;
    ///
    /// match user {
    ///     Some(user) => println!("User found in org: {}", user.organization_id),
    ///     None => println!("User not authorized"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_one_by_email(
        pool: &sqlx::Pool<Postgres>,
        email: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id, organization_id, email, created_at, updated_at 
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test(migrations = "../migrations")]
    async fn test_find_one_by_email(pool: PgPool) {
        // Arrange a test user
        let user = User::factory()
            .for_default_organization()
            .email("wile.coyote@acme.org".to_owned())
            .create(&pool)
            .await
            .unwrap();

        // Act the call to the method
        let retrieved = User::find_one_by_email(&pool, "wile.coyote@acme.org")
            .await
            .unwrap();

        // Assert the user is retrieved
        assert!(retrieved.is_some());
        assert_eq!(user, retrieved.unwrap());
    }
}
