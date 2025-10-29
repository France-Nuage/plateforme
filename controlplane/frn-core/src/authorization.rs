//! Authorization and permission checking
//!
//! Provides traits and types for checking permissions on resources. The
//! `AuthorizationServer` trait abstracts over authorization backends like
//! SpiceDB, enabling fluent permission checks via the builder API.
//! Includes `Resource` for authorizable entities, `Principal` for actors,
//! and `Permission` for actions.
//!
//! Use `server.can(principal).perform(permission).over(resource).await?` to
//! check permissions with the fluent API.

mod authorize;
mod check;
mod lookup;
mod permission;
mod principal;
mod relationship;
mod resource;
mod zookie;

pub use authorize::*;
pub use frn_derive::*;
pub use permission::*;
pub use principal::*;
pub use relationship::*;
pub use resource::*;
pub use zookie::*;
