use crate::Error;
use crate::authorization::{Authorize, Principal, Relationship};
use chrono::{DateTime, Utc};
use fabrique::{Model, Persist, Query};
use serde_json::Value;
use sqlx::postgres::PgListener;
use sqlx::{Pool, Postgres};
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

/// Channel for notifying workers of new operations to process.
pub const OPERATIONS_CHANNEL: &str = "operations";

/// Channel for notifying clients that an operation has completed.
pub const OPERATIONS_COMPLETED_CHANNEL: &str = "operations_completed";

/// Types d'opérations supportées par le système.
#[derive(Clone, Debug, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum OperationKind {
    /// Écriture de relations dans SpiceDB
    WriteRelationships,
}

impl TryFrom<String> for OperationKind {
    type Error = crate::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        OperationKind::from_str(&value).map_err(|_| Error::UnknownOperation(value))
    }
}

impl TryFrom<OperationKind> for String {
    type Error = crate::Error;

    fn try_from(value: OperationKind) -> Result<Self, Self::Error> {
        Ok(value.to_string())
    }
}

/// Opération asynchrone traitée par un worker.
#[derive(Debug, Model)]
pub struct Operation {
    #[fabrique(primary_key)]
    pub id: Uuid,
    #[fabrique(as = "String")]
    pub kind: OperationKind,
    pub payload: Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Operation {
    /// Crée une opération d'écriture de relations.
    pub fn write_relationships(relationships: Vec<Relationship>) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind: OperationKind::WriteRelationships,
            payload: serde_json::to_value(relationships).unwrap(),
            status: "pending".to_string(),
            created_at: Utc::now(),
            completed_at: None,
        }
    }

    /// Dispatches the operation to the queue for processing by a worker.
    pub async fn dispatch(self, pool: &Pool<Postgres>) -> Result<Uuid, Error> {
        let operation = self.create(pool).await?;

        sqlx::query("SELECT pg_notify($1, $2)")
            .bind(OPERATIONS_CHANNEL)
            .bind(operation.id.to_string())
            .execute(pool)
            .await?;

        tracing::info!("operation {} dispatched", operation.id);

        Ok(operation.id)
    }

    /// Marks the operation as completed and notifies waiting clients.
    pub async fn mark_completed(id: Uuid, pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE operations SET completed_at = now(), status = 'completed' WHERE id = $1",
        )
        .bind(id)
        .execute(pool)
        .await?;

        sqlx::query("SELECT pg_notify($1, $2)")
            .bind(OPERATIONS_COMPLETED_CHANNEL)
            .bind(id.to_string())
            .execute(pool)
            .await?;

        tracing::info!("operation {} completed", id);

        Ok(())
    }
}

#[derive(Clone)]
pub struct Operations<A: Authorize> {
    auth: A,
    db: Pool<Postgres>,
}

impl<A: Authorize> Operations<A> {
    pub fn new(auth: A, db: Pool<Postgres>) -> Self {
        Self { auth, db }
    }

    /// Get the operation matching the given id
    pub async fn get<P: Principal>(
        &mut self,
        _principal: &P,
        id: Uuid,
    ) -> Result<Operation, Error> {
        Operation::find(&self.db, id).await.map_err(Into::into)
    }

    /// Wait for operation to complete.
    ///
    /// Listens for completion notifications via PostgreSQL LISTEN/NOTIFY.
    /// If a timeout is provided and expires before completion, returns the current state.
    pub async fn wait<P: Principal>(
        &mut self,
        principal: &P,
        id: Uuid,
        timeout: Option<std::time::Duration>,
    ) -> Result<Operation, Error> {
        let mut listener = PgListener::connect_with(&self.db).await?;
        listener.listen(OPERATIONS_COMPLETED_CHANNEL).await?;

        // Check if already completed after setting up listener
        let operation = self.get(principal, id).await?;
        if operation.completed_at.is_some() {
            return Ok(operation);
        }

        // Wait for completion notification with optional timeout
        let wait_future = async {
            loop {
                let notification = listener.recv().await?;
                let notified_id = Uuid::parse_str(notification.payload())
                    .map_err(|_| Error::Other("Invalid UUID in notification".to_string()))?;

                if notified_id == id {
                    return self.get(principal, id).await;
                }
            }
        };

        match timeout {
            Some(duration) => match tokio::time::timeout(duration, wait_future).await {
                Ok(result) => result,
                Err(_) => self.get(principal, id).await,
            },
            None => wait_future.await,
        }
    }
}
