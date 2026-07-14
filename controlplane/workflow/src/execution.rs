use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;

use crate::workflows::WorkflowDefinitions;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(transparent)]
pub struct WorkflowExecutionId(Uuid);

impl WorkflowExecutionId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for WorkflowExecutionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for WorkflowExecutionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::str::FromStr for WorkflowExecutionId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self)
    }
}

#[derive(
    sqlx::Type, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Display, EnumString, Default,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum WorkflowExecutionStatus {
    #[default]
    Pending,
    Running,
    WillRetry,
    Completed,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum WorkflowInitiator {
    User(Uuid),
    Workflow(WorkflowExecutionId),
    System,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub execution_id: WorkflowExecutionId,
    pub initiated_by: WorkflowInitiator,
    pub status: WorkflowExecutionStatus,
    pub soft_try_count: i32,
    pub hard_try_count: i32,
    pub max_try_count: i32,
    pub next_retry_at: DateTime<Utc>,
    pub dependencies: Vec<WorkflowExecutionId>,
    pub definition: WorkflowDefinitions,
}

impl WorkflowExecution {
    pub fn new(
        initiated_by: WorkflowInitiator,
        max_retry: i32,
        definition: WorkflowDefinitions,
        schedule_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            execution_id: WorkflowExecutionId::new(),
            initiated_by,
            status: WorkflowExecutionStatus::Pending,
            soft_try_count: 0,
            hard_try_count: 0,
            max_try_count: max_retry,
            dependencies: Vec::new(),
            definition,
            next_retry_at: schedule_at.unwrap_or_else(Utc::now),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str::FromStr;

    use crate::workflows::write_relationships::WriteRelationshipsWorkflow;

    fn sample_definition() -> WorkflowDefinitions {
        WorkflowDefinitions::WriteRelationships(WriteRelationshipsWorkflow::new(vec![]))
    }

    #[test]
    fn execution_id_round_trips_through_uuid() {
        let uuid = Uuid::now_v7();

        let id = WorkflowExecutionId::from_uuid(uuid);

        assert_eq!(id.as_uuid(), uuid);
    }

    #[test]
    fn execution_id_round_trips_through_string() {
        let id = WorkflowExecutionId::new();

        let parsed = WorkflowExecutionId::from_str(&id.to_string()).unwrap();

        assert_eq!(parsed, id);
    }

    #[test]
    fn execution_id_from_str_rejects_non_uuid() {
        assert!(WorkflowExecutionId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn execution_id_new_yields_distinct_ids() {
        assert_ne!(WorkflowExecutionId::new(), WorkflowExecutionId::new());
    }

    #[test]
    fn status_display_and_from_str_round_trip_every_variant() {
        let variants = [
            WorkflowExecutionStatus::Pending,
            WorkflowExecutionStatus::Running,
            WorkflowExecutionStatus::WillRetry,
            WorkflowExecutionStatus::Completed,
            WorkflowExecutionStatus::Failed,
        ];

        let round_tripped: Vec<WorkflowExecutionStatus> = variants
            .iter()
            .map(|s| WorkflowExecutionStatus::from_str(&s.to_string()).unwrap())
            .collect();

        assert_eq!(round_tripped, variants);
    }

    #[test]
    fn status_serializes_as_snake_case() {
        let json = serde_json::to_string(&WorkflowExecutionStatus::WillRetry).unwrap();

        assert_eq!(json, "\"will_retry\"");
    }

    #[test]
    fn status_default_is_pending() {
        assert_eq!(
            WorkflowExecutionStatus::default(),
            WorkflowExecutionStatus::Pending
        );
    }

    #[test]
    fn new_execution_starts_pending_with_zeroed_counters() {
        let execution =
            WorkflowExecution::new(WorkflowInitiator::System, 5, sample_definition(), None);

        assert!(matches!(
            (
                execution.status,
                execution.soft_try_count,
                execution.hard_try_count,
                execution.max_try_count,
                execution.dependencies.is_empty(),
            ),
            (WorkflowExecutionStatus::Pending, 0, 0, 5, true)
        ));
    }

    #[test]
    fn new_execution_honours_explicit_schedule_at() {
        let when = Utc::now() + chrono::Duration::hours(2);

        let execution = WorkflowExecution::new(
            WorkflowInitiator::System,
            3,
            sample_definition(),
            Some(when),
        );

        assert_eq!(execution.next_retry_at, when);
    }

    #[test]
    fn new_execution_defaults_schedule_at_to_now() {
        let before = Utc::now();

        let execution =
            WorkflowExecution::new(WorkflowInitiator::System, 3, sample_definition(), None);

        let after = Utc::now();
        assert!((before..=after).contains(&execution.next_retry_at));
    }
}
