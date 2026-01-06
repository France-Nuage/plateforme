mod error;

pub use error::Error;

use frn_core::authorization::Relationship;
use frn_core::longrunning::{OPERATIONS_CHANNEL, Operation, OperationKind};
use spicedb::SpiceDB;
use sqlx::postgres::PgListener;
use sqlx::{Pool, Postgres};

pub struct Worker<Authorizer> {
    authorizer: Authorizer,
    connection: Pool<Postgres>,
}

impl<A> Worker<A> {
    pub fn new(authorizer: A, connection: Pool<Postgres>) -> Self {
        Self {
            authorizer,
            connection,
        }
    }

    pub async fn consume(&self) -> Result<Option<Operation>, Error> {
        sqlx::query_as(
            "SELECT * FROM operations WHERE completed_at IS NULL ORDER BY created_at LIMIT 1 FOR UPDATE SKIP LOCKED",
        )
        .fetch_optional(&self.connection)
        .await
        .map_err(Into::into)
    }
}

impl Worker<SpiceDB> {
    pub async fn subscribe(&mut self) -> Result<(), Error> {
        let mut listener = PgListener::connect_with(&self.connection).await?;
        listener.listen(OPERATIONS_CHANNEL).await?;

        tracing::info!("listening on channel '{}'", OPERATIONS_CHANNEL);

        loop {
            listener.recv().await?;

            if let Some(operation) = self.consume().await? {
                self.execute(&operation).await?;
            }
        }
    }

    pub async fn execute(&mut self, operation: &Operation) -> Result<(), Error> {
        match &operation.kind {
            OperationKind::WriteRelationships => {
                let relationships: Vec<Relationship> =
                    serde_json::from_value(operation.payload.clone())?;

                for relationship in relationships {
                    self.authorizer
                        .write_relationship(
                            relationship.subject_type,
                            relationship.subject_id,
                            relationship.relation.to_string(),
                            relationship.object_type,
                            relationship.object_id,
                        )
                        .await?;

                    tracing::info!("wrote relationship");
                }

                Ok(())
            }
        }
    }
}
