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
