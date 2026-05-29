use sqlx::PgConnection;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
struct FsmTransition {
    state_from: String,
    state_to: String,
    event: String,
}

#[derive(Debug, Error)]
pub enum TransitionError {
    #[error("{0}")]
    Database(#[from] sqlx::Error),
    #[error("cannot transition status from {0} to {1}, available transitions: {2}")]
    InvalidTransition(String, String, String),
}

pub struct FsmRepository;

impl FsmRepository {
    pub async fn current_state_name(
        conn: &mut PgConnection,
        state_machine_id: &Uuid,
    ) -> Result<String, sqlx::Error> {
        sqlx::query_scalar::<_, String>(
            r#"SELECT abs.name
               FROM lib_fsm.state_machine sm
               INNER JOIN lib_fsm.abstract_state abs
                   ON abs.abstract_state__id = sm.abstract_state__id
               WHERE sm.state_machine__id = $1"#,
        )
        .bind(state_machine_id)
        .fetch_one(conn)
        .await
    }

    async fn fetch_available_transitions(
        conn: &mut PgConnection,
        state_machine_id: &Uuid,
    ) -> Result<Vec<FsmTransition>, TransitionError> {
        Ok(sqlx::query_as::<_, FsmTransition>(
            r#"WITH available AS (
                    SELECT abt.to_abstract_state__id, abt.event, abs.name AS state_from
                    FROM lib_fsm.state_machine sm
                    JOIN lib_fsm.abstract_state abs ON abs.abstract_state__id = sm.abstract_state__id
                    JOIN lib_fsm.abstract_transition abt ON abt.from_abstract_state__id = abs.abstract_state__id
                    WHERE sm.state_machine__id = $1
                )
                SELECT a.state_from,
                       abs.name AS state_to,
                       a.event
                FROM lib_fsm.abstract_state abs
                JOIN available a ON a.to_abstract_state__id = abs.abstract_state__id"#,
        )
        .bind(state_machine_id)
        .fetch_all(conn)
        .await?)
    }

    pub async fn state_machine_transition(
        conn: &mut PgConnection,
        state_machine_id: &Uuid,
        new_status: String,
    ) -> Result<(), TransitionError> {
        let current_state_name = sqlx::query_scalar::<_, String>(
            r#"SELECT abs.name
               FROM lib_fsm.state_machine sm
               INNER JOIN lib_fsm.abstract_state abs ON abs.abstract_state__id = sm.abstract_state__id
               WHERE sm.state_machine__id = $1"#,
        )
        .bind(state_machine_id)
        .fetch_one(&mut *conn)
        .await?;

        if let Some(event) = sqlx::query_scalar::<_, String>(
            r#"SELECT abt.event
               FROM lib_fsm.state_machine sm
               JOIN lib_fsm.abstract_transition abt ON abt.from_abstract_state__id = sm.abstract_state__id
               JOIN lib_fsm.abstract_state target ON target.abstract_state__id = abt.to_abstract_state__id
               WHERE sm.state_machine__id = $1
                 AND target.name = $2"#,
        )
        .bind(state_machine_id)
        .bind(&new_status)
        .fetch_optional(&mut *conn)
        .await?
        {
            sqlx::query(r#"SELECT lib_fsm.state_machine_transition($1, $2)"#)
                .bind(state_machine_id)
                .bind(event)
                .execute(conn)
                .await?;

            return Ok(());
        }

        let available = Self::fetch_available_transitions(&mut *conn, state_machine_id).await?;

        let available_str = available
            .iter()
            .map(|t| format!("{} -> {} (event '{}')", t.state_from, t.state_to, t.event))
            .collect::<Vec<_>>()
            .join(", ");

        Err(TransitionError::InvalidTransition(
            current_state_name,
            new_status,
            available_str,
        ))
    }
}
