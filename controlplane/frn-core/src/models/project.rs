use database::{Factory, Persistable, Repository};
use frn_core::authorization::Authorize;
use sqlx::prelude::FromRow;
use sqlx::types::chrono;
use uuid::Uuid;

use crate::models::OrganizationFactory;

#[derive(Authorize, Debug, Default, Factory, FromRow, Repository)]
pub struct Project {
    /// The project id
    #[repository(primary)]
    pub id: Uuid,

    /// The project name
    pub name: String,

    /// The organization this project belongs to
    #[factory(relation = "OrganizationFactory")]
    pub organization_id: Uuid,

    /// Creation time of the project
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last update time of the project
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
