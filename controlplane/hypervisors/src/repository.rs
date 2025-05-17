use uuid::Uuid;

use crate::{Problem, model::Hypervisor};

pub async fn list(pool: &sqlx::PgPool) -> Result<Vec<Hypervisor>, Problem> {
    sqlx::query_as!(
        Hypervisor,
        "SELECT id, url, authorization_token, storage_name FROM hypervisors"
    )
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub async fn create(pool: &sqlx::PgPool, hypervisor: &Hypervisor) -> Result<(), Problem> {
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

pub async fn delete(pool: &sqlx::PgPool, id: Uuid) -> Result<(), Problem> {
    sqlx::query!("DELETE FROM hypervisors WHERE id = $1", &id)
        .execute(pool)
        .await?;
    Ok(())
}
