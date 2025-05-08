//! Repository module for Instance persistence operations
//!
//! This module provides database access functions for the Instance entity,
//! implemented using sqlx with PostgreSQL.

use uuid::Uuid;

use crate::{model::Instance, problem::Problem};

/// Retrieves all instances from the database.
///
/// # Arguments
///
/// * `pool` - PostgreSQL connection pool
///
/// # Returns
///
/// A vector of all Instance records or a Problem if the operation fails
pub async fn list(pool: &sqlx::PgPool) -> Result<Vec<Instance>, Problem> {
    sqlx::query_as!(
        Instance,
        "SELECT id, hypervisor_id, distant_id, cpu_usage_percent, max_cpu_cores, max_memory_bytes , memory_usage_bytes, name, status FROM instances"
    )
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub async fn find_one_by_distant_id(
    pool: &sqlx::PgPool,
    distant_id: &str,
) -> Result<Option<Instance>, Problem> {
    sqlx::query_as!(
        Instance,
        "SELECT id, hypervisor_id, distant_id, cpu_usage_percent, max_cpu_cores, max_memory_bytes, memory_usage_bytes, name, status FROM instances WHERE distant_id = $1",
        distant_id
    )
    .fetch_optional(pool)
    .await
    .map_err(Into::into)
}

/// Creates a new instance record in the database.
///
/// # Arguments
///
/// * `pool` - PostgreSQL connection pool
/// * `instance` - The Instance to be created
///
/// # Returns
///
/// Ok(()) on success or a Problem if the operation fails
pub async fn create(pool: &sqlx::PgPool, instance: &Instance) -> Result<(), Problem> {
    sqlx::query!(
        "INSERT INTO instances (id, hypervisor_id, distant_id, cpu_usage_percent, max_cpu_cores, max_memory_bytes, memory_usage_bytes, name, status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        &instance.id,
        &instance.hypervisor_id,
        &instance.distant_id,
        instance.cpu_usage_percent,
        &instance.max_cpu_cores,
        instance.max_memory_bytes as i64,
        instance.memory_usage_bytes as i64,
        &instance.name,
        instance.status.to_string()
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Retrieves a single instance by its ID.
///
/// # Arguments
///
/// * `pool` - PostgreSQL connection pool
/// * `id` - UUID of the instance to retrieve
///
/// # Returns
///
/// The requested Instance or InstanceNotFound Problem if not present
pub async fn read(pool: &sqlx::PgPool, id: Uuid) -> Result<Instance, Problem> {
    sqlx::query_as!(
        Instance,
        "SELECT id, hypervisor_id, distant_id, cpu_usage_percent, max_cpu_cores, max_memory_bytes, memory_usage_bytes, name, status FROM instances WHERE id = $1",
        &id
    )
    .fetch_one(pool)
    .await
    .map_err(|err| match err {
        sqlx::Error::RowNotFound => Problem::InstanceNotFound(id),
        err => Problem::Other(Box::new(err)),
    })
}

/// Upserts multiple instances in the database.
///
/// # Arguments
///
/// * `pool` - PostgreSQL connection pool
/// * `instances` - Slice of Instance objects to be upserted
///
/// # Returns
///
/// Ok(()) on success or a Problem if the operation fails
pub async fn upsert(pool: &sqlx::PgPool, instances: &[Instance]) -> Result<(), Problem> {
    // Extract the data into separate vectors
    let ids: Vec<Uuid> = instances.iter().map(|i| i.id).collect();
    let hypervisor_ids: Vec<Uuid> = instances.iter().map(|i| i.hypervisor_id).collect();
    let distant_ids: Vec<String> = instances.iter().map(|i| i.distant_id.clone()).collect();

    sqlx::query!(
        r#"
        INSERT INTO instances (id, hypervisor_id, distant_id)
        SELECT id, hypervisor_id, distant_id
        FROM UNNEST($1::uuid[], $2::uuid[], $3::text[]) AS t(id, hypervisor_id, distant_id)
        ON CONFLICT (id) DO UPDATE
        SET
            hypervisor_id = EXCLUDED.hypervisor_id,
            distant_id = EXCLUDED.distant_id
    "#,
        &ids,
        &hypervisor_ids,
        &distant_ids
    )
    .execute(pool)
    .await?;

    Ok(())
}
