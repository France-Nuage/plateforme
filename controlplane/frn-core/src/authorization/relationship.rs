use crate::{
    Error,
    authorization::{Authorize, Resource},
};
use database::{Factory, Persistable, Repository};
use sqlx::{FromRow, Pool, Postgres, postgres::PgListener};
use std::fmt::Display;
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

pub const RELATIONSHIP_QUEUE_NAME: &str = "relationship_queue_event";

#[derive(Debug, Default, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Relation {
    #[strum(serialize = "project")]
    BelongsToProject,
    Member,
    #[default]
    Unspecified,
}

impl From<Relation> for String {
    fn from(value: Relation) -> Self {
        value.to_string()
    }
}

impl From<String> for Relation {
    fn from(value: String) -> Self {
        Relation::from_str(&value).expect("could not parse permission")
    }
}

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
    pub fn new<Subject: Resource, Object: Resource>(
        subject: &Subject,
        relation: Relation,
        object: &Object,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            object_id: object.id().to_string(),
            object_type: object.name().to_string(),
            relation,
            subject_id: subject.id().to_string(),
            subject_type: subject.name().to_string(),
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

        tracing::info!("relationship {} published", relationship.id);

        Ok(())
    }

    pub async fn subscribe<A>(pool: Pool<Postgres>, authz: &mut A) -> Result<(), Error>
    where
        A: Authorize,
    {
        // connect a postgresql listener with the pool
        let mut listener = PgListener::connect_with(&pool).await?;

        // start listening for notification on the relationship channel, which acts as a queue
        listener.listen(RELATIONSHIP_QUEUE_NAME).await?;

        loop {
            listener.recv().await?;
            Self::consume(pool.clone(), authz).await?;
        }
    }

    pub async fn consume<A: Authorize>(
        pool: Pool<Postgres>,
        auth: &mut A,
    ) -> Result<Option<Relationship>, Error> {
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

        auth.write_relationship(&relationship).await?;

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
