use crate::{Error, authorization::Relationship};
use chrono::{DateTime, Utc};
use fabrique::{Model, Persist};
use serde_json::Value;
use sqlx::{Pool, Postgres};
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

pub const OPERATIONS_CHANNEL: &str = "operations";

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
}
