use uuid::Uuid;

use crate::{Problem, model::Hypervisor};

pub async fn list(pool: &sqlx::PgPool) -> Result<Vec<Hypervisor>, sqlx::Error> {
    sqlx::query_as!(
        Hypervisor,
        "SELECT id, url, authorization_token, storage_name FROM hypervisors"
    )
    .fetch_all(pool)
    .await
}

pub async fn create(pool: &sqlx::PgPool, hypervisor: &Hypervisor) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO hypervisors (id, url, authorization_token, storage_name) VALUES ($1, $2, $3, $4)",
        &hypervisor.id,
        &hypervisor.url,
        &hypervisor.authorization_token,
        &hypervisor.storage_name
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn read(pool: &sqlx::PgPool, id: Uuid) -> Result<Hypervisor, Problem> {
    sqlx::query_as!(
        Hypervisor,
        "SELECT id, url, authorization_token, storage_name FROM hypervisors WHERE hypervisors.id = $1", id
    )
    .fetch_one(pool)
    .await.map_err(|err| {
            match err {
                sqlx::Error::RowNotFound => Problem::NotFound(id),
                err => Problem::Other { source: Box::new(err) }


            }
        })
}
