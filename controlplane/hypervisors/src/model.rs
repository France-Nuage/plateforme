use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Default, FromRow)]
pub struct Hypervisor {
    pub id: Uuid,
    pub url: String,
    pub authorization_token: String,
    pub storage_name: String,
}
