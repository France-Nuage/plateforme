use crate::{Authz, Error};
use database::{Factory, Persistable, Repository};
use frn_core::authorization::Relation;
use sqlx::{FromRow, PgPool, Pool, Postgres, postgres::PgListener};
use std::fmt::Display;
use tracing::info;
use uuid::Uuid;

pub const RELATIONSHIP_QUEUE_NAME: &str = "relationship_queue_event";

#[derive(Debug, Default, Factory, FromRow, Repository)]
#[table(name = "relationship_queue")]
pub struct Relationship {
    #[repository(primary)]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub relation: Relation,
    pub object_id: String,
    pub object_type: String,
    pub subject_id: String,
    pub subject_type: String,
}

impl Relationship {
    pub fn new(object: (String, String), relation: Relation, subject: (String, String)) -> Self {
        let (object_type, object_id) = object;
        let (subject_type, subject_id) = subject;
        Self {
            id: Uuid::new_v4(),
            relation,
            object_id,
            object_type,
            subject_id,
            subject_type,
        }
    }

    pub async fn publish(self, pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
        // create the relationship
        let relationship = self.create(pool).await?;

        // notify the event
        sqlx::query!(
            "SELECT pg_notify($1, $2)",
            RELATIONSHIP_QUEUE_NAME,
            relationship.id.to_string()
        )
        .execute(pool)
        .await?;

        info!("relationship {} published", relationship.id);

        Ok(())
    }

    pub async fn subscribe<F, Fut>(pool: PgPool, authz: Authz, callback: F) -> Result<(), Error>
    where
        F: Fn(PgPool, Authz) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Option<Relationship>, Error>> + Send,
    {
        // connect a postgresql listener with the pool
        let mut listener = PgListener::connect_with(&pool).await?;

        // start listening for notification on the relationship channel, which acts as a queue
        listener.listen(RELATIONSHIP_QUEUE_NAME).await?;

        loop {
            listener.recv().await?;
            callback(pool.clone(), authz.clone()).await?;
        }
    }

    pub async fn consume(pool: PgPool, authz: Authz) -> Result<Option<Relationship>, Error> {
        println!("received notification");
        let mut tx = pool.begin().await?;
        let relationship = sqlx::query_as!(
            Relationship,
            r#"
            SELECT id, object_id, object_type, relation, subject_id, subject_type
            FROM relationship_queue
            ORDER BY created_at
            LIMIT 1
            FOR UPDATE SKIP LOCKED;
        "#
        )
        .fetch_optional(&mut *tx)
        .await?;

        let relationship = match relationship {
            Some(relationship) => relationship,
            None => return Ok(None),
        };

        authz.write_relationship(&relationship).await?;

        tracing::info!(
            "relationship {} written into the authz server",
            &relationship
        );

        sqlx::query!(
            "DELETE FROM relationship_queue WHERE id = $1",
            &relationship.id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        println!("committed");

        Ok(Some(relationship))
    }
}

impl Display for Relationship {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}#{}@{}:{}",
            self.object_type, self.object_id, self.relation, self.subject_type, self.subject_id
        )
    }
}
