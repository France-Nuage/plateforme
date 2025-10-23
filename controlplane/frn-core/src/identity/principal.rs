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
    const NAME: &'static str = "principal";

    fn id(&self) -> &Self::Id {
        match self {
            Self::ServiceAccount(inner) => inner.id(),
            Self::User(inner) => inner.id(),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::ServiceAccount(_) => ServiceAccount::NAME,
            Self::User(_) => User::NAME,
        }
    }
}

impl frn_core::authorization::Principal for Principal {
    async fn organizations(
        &self,
        _connection: &Pool<Postgres>,
    ) -> Result<Vec<Organization>, Error> {
        unimplemented!()
    }
}
