use uuid::Uuid;

use crate::model::Instance;

pub async fn list(pool: &sqlx::PgPool) -> Result<Vec<Instance>, sqlx::Error> {
    sqlx::query_as!(
        Instance,
        "SELECT id, hypervisor_id, distant_id FROM instances"
    )
    .fetch_all(pool)
    .await
}

pub async fn create(pool: &sqlx::PgPool, instance: &Instance) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO instances (id, hypervisor_id, distant_id) VALUES ($1, $2, $3)",
        &instance.id,
        &instance.hypervisor_id,
        &instance.distant_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn read(pool: &sqlx::PgPool, id: Uuid) -> Result<Instance, sqlx::Error> {
    sqlx::query_as!(
        Instance,
        "SELECT id, hypervisor_id, distant_id FROM instances WHERE id = $1",
        &id
    )
    .fetch_one(pool)
    .await
}
