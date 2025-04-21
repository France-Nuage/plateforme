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
        "SELECT id, hypervisor_id, distant_id FROM instances"
    )
    .fetch_all(pool)
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
        "INSERT INTO instances (id, hypervisor_id, distant_id) VALUES ($1, $2, $3)",
        &instance.id,
        &instance.hypervisor_id,
        &instance.distant_id,
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
        "SELECT id, hypervisor_id, distant_id FROM instances WHERE id = $1",
        &id
    )
    .fetch_one(pool)
    .await
    .map_err(|err| match err {
        sqlx::Error::RowNotFound => Problem::InstanceNotFound(id),
        err => Problem::Other(Box::new(err)),
    })
}
