//! Repository module for Instance persistence operations
//!
//! This module provides database access functions for the Instance entity,
//! implemented using sqlx with PostgreSQL.

use uuid::Uuid;

use crate::model::Instance;

/// Retrieves all instances from the database.
///
/// # Arguments
///
/// * `pool` - PostgreSQL connection pool
///
/// # Returns
///
/// A vector of all Instance records or a Problem if the operation fails
pub async fn list(pool: &sqlx::PgPool) -> Result<Vec<Instance>, sqlx::Error> {
    sqlx::query_as!(
        Instance,
        "SELECT id, hypervisor_id, project_id, distant_id, cpu_usage_percent, max_cpu_cores, max_memory_bytes , memory_usage_bytes, name, status, created_at, updated_at FROM instances"
    )
    .fetch_all(pool)
    .await
}

pub async fn find_one_by_distant_id(
    pool: &sqlx::PgPool,
    distant_id: &str,
) -> Result<Option<Instance>, sqlx::Error> {
    sqlx::query_as!(
        Instance,
        "SELECT id, hypervisor_id, project_id, distant_id, cpu_usage_percent, max_cpu_cores, max_memory_bytes, memory_usage_bytes, name, status, created_at, updated_at FROM instances WHERE distant_id = $1",
        distant_id
    )
    .fetch_optional(pool)
    .await
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
pub async fn create(pool: &sqlx::PgPool, instance: Instance) -> Result<Instance, sqlx::Error> {
    sqlx::query_as!(
        Instance,
        r#"
        INSERT INTO instances (id, hypervisor_id, project_id, distant_id, cpu_usage_percent, max_cpu_cores, max_memory_bytes, memory_usage_bytes, name, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, hypervisor_id, project_id, distant_id, cpu_usage_percent, max_cpu_cores, max_memory_bytes, memory_usage_bytes, name, status, created_at, updated_at
        "#,
        &instance.id,
        &instance.hypervisor_id,
        &instance.project_id,
        &instance.distant_id,
        instance.cpu_usage_percent,
        &instance.max_cpu_cores,
        instance.max_memory_bytes as i64,
        instance.memory_usage_bytes as i64,
        &instance.name,
        instance.status.to_string()
    )
    .fetch_one(pool)
    .await
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
pub async fn read(pool: &sqlx::PgPool, id: Uuid) -> Result<Instance, sqlx::Error> {
    sqlx::query_as!(
        Instance,
        "SELECT id, hypervisor_id, project_id, distant_id, cpu_usage_percent, max_cpu_cores, max_memory_bytes, memory_usage_bytes, name, status, created_at, updated_at FROM instances WHERE id = $1",
        &id
    )
    .fetch_one(pool)
    .await
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
pub async fn upsert(pool: &sqlx::PgPool, instances: &[Instance]) -> Result<(), sqlx::Error> {
    // Extract the data into separate vectors
    let ids: Vec<Uuid> = instances.iter().map(|i| i.id).collect();
    let hypervisor_ids: Vec<Uuid> = instances.iter().map(|i| i.hypervisor_id).collect();
    let project_ids: Vec<Uuid> = instances.iter().map(|i| i.project_id).collect();
    let distant_ids: Vec<String> = instances.iter().map(|i| i.distant_id.clone()).collect();
    let cpu_usage_percents: Vec<f64> = instances.iter().map(|i| i.cpu_usage_percent).collect();
    let max_cpu_cores: Vec<i32> = instances.iter().map(|i| i.max_cpu_cores).collect();
    let max_memory_bytes: Vec<i64> = instances.iter().map(|i| i.max_memory_bytes).collect();
    let memory_usage_bytes: Vec<i64> = instances.iter().map(|i| i.memory_usage_bytes).collect();
    let names: Vec<String> = instances.iter().map(|i| i.name.clone()).collect();
    let statuses: Vec<String> = instances.iter().map(|i| i.status.to_string()).collect();

    sqlx::query!(
        r#"
        INSERT INTO instances (id, hypervisor_id, project_id, distant_id, cpu_usage_percent, max_cpu_cores, max_memory_bytes, memory_usage_bytes, name, status)
        SELECT id, hypervisor_id, project_id, distant_id, cpu_usage_percent, max_cpu_cores, max_memory_bytes, memory_usage_bytes, name, status
        FROM UNNEST($1::uuid[], $2::uuid[], $3::uuid[], $4::text[], $5::float8[], $6::int4[], $7::int8[], $8::int8[], $9::text[], $10::text[]) AS t(id, hypervisor_id, project_id, distant_id, cpu_usage_percent, max_cpu_cores, max_memory_bytes, memory_usage_bytes, name, status)
        ON CONFLICT (id) DO UPDATE
        SET
            hypervisor_id = EXCLUDED.hypervisor_id,
            project_id = EXCLUDED.project_id,
            distant_id = EXCLUDED.distant_id,
            cpu_usage_percent = EXCLUDED.cpu_usage_percent,
            max_cpu_cores = EXCLUDED.max_cpu_cores,
            max_memory_bytes = EXCLUDED.max_memory_bytes,
            memory_usage_bytes = EXCLUDED.memory_usage_bytes,
            name = EXCLUDED.name,
            status = EXCLUDED.status,
            updated_at = NOW()
    "#,
        &ids,
        &hypervisor_ids,
        &project_ids,
        &distant_ids,
        &cpu_usage_percents,
        &max_cpu_cores,
        &max_memory_bytes,
        &memory_usage_bytes,
        &names,
        &statuses,
    )
    .execute(pool)
    .await?;
    Ok(())
}
