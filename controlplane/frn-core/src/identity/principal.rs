use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
    Error,
    authorization::Resource,
    identity::{ServiceAccount, User},
    resourcemanager::Organization,
};

#[derive(Debug)]
pub enum Principal {
    ServiceAccount(ServiceAccount),
    User(User),
}

impl Resource for Principal {
    type Id = Uuid;
    const RESOURCE_NAME: &'static str = "principal";

    fn id(&self) -> &Self::Id {
        match self {
            Self::ServiceAccount(inner) => inner.id(),
            Self::User(inner) => inner.id(),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::ServiceAccount(_) => ServiceAccount::RESOURCE_NAME,
            Self::User(_) => User::RESOURCE_NAME,
        }
    }

    /// `some()` builds a placeholder resource for relationship writes, which is
    /// only ever done against concrete `User`/`ServiceAccount` resources, never
    /// against the `Principal` enum itself. Reaching this is a programming bug.
    #[allow(refining_impl_trait)]
    fn some(_id: Self::Id) -> User {
        unreachable!("`some()` is not valid on the `Principal` enum; use a concrete resource type")
    }
}

impl frn_core::authorization::Principal for Principal {
    async fn organizations(&self, connection: &Pool<Postgres>) -> Result<Vec<Organization>, Error> {
        match self {
            Principal::ServiceAccount(principal) => principal.organizations(connection).await,
            Principal::User(principal) => principal.organizations(connection).await,
        }
    }

    fn is_platform_admin(&self) -> bool {
        match self {
            Principal::ServiceAccount(principal) => principal.is_platform_admin(),
            Principal::User(principal) => principal.is_platform_admin(),
        }
    }
}
