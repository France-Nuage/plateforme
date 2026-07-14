use std::error::Error as StdError;

use frn_core::authorization::Relationship;
use serde::{Deserialize, Serialize};

use crate::WorkerContext;
use crate::operations::Operations;
use crate::operations::write_relationships::WriteRelationshipsOp;
use crate::workflows::WorkflowDefinition;

#[derive(Debug, Serialize, Deserialize)]
pub struct WriteRelationshipsWorkflow {
    relationships: Vec<Relationship>,
    done: bool,
}

impl WriteRelationshipsWorkflow {
    pub fn new(relationships: Vec<Relationship>) -> Self {
        Self {
            relationships,
            done: false,
        }
    }
}

impl WorkflowDefinition for WriteRelationshipsWorkflow {
    type Error = Box<dyn StdError>;

    async fn next_operations(
        &mut self,
        _ctx: WorkerContext,
    ) -> Result<Vec<Operations>, Self::Error> {
        if self.done {
            return Ok(vec![]);
        }

        self.done = true;

        Ok(vec![Operations::WriteRelationships(WriteRelationshipsOp {
            relationships: self.relationships.clone(),
        })])
    }

    fn name(&self) -> &str {
        "WriteRelationships"
    }
}
