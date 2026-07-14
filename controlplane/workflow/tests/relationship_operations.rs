use frn_core::authorization::{Relation, Relationship};
use spicedb::SpiceDB;
use spicedb::mock::RelationshipStore;
use workflow::execution::WorkflowExecutionId;
use workflow::operations::Operation;
use workflow::operations::delete_relationships::DeleteRelationshipsOp;
use workflow::operations::write_relationships::WriteRelationshipsOp;

mod common;

fn sample_relationship() -> Relationship {
    Relationship {
        relation: Relation::Member,
        object_type: "organization".to_owned(),
        object_id: "org-1".to_owned(),
        subject_type: "user".to_owned(),
        subject_id: "alice".to_owned(),
    }
}

fn store_contains(store: &RelationshipStore, relationship: &Relationship) -> bool {
    store
        .lock()
        .expect("store poisoned")
        .contains(&relationship.to_string())
}

#[tokio::test]
async fn write_execute_persists_the_relationship() {
    let (spicedb, store) = SpiceDB::recording().await;
    let ctx = common::context_with_spicedb(spicedb);
    let relationship = sample_relationship();

    let op = WriteRelationshipsOp {
        relationships: vec![relationship.clone()],
    };
    op.execute(ctx, WorkflowExecutionId::new())
        .await
        .expect("write execute must succeed");

    assert!(store_contains(&store, &relationship));
}

#[tokio::test]
async fn write_rollback_removes_the_relationship() {
    let (spicedb, store) = SpiceDB::recording().await;
    let ctx = common::context_with_spicedb(spicedb);
    let relationship = sample_relationship();

    let op = WriteRelationshipsOp {
        relationships: vec![relationship.clone()],
    };
    let executed = op
        .execute(ctx.clone(), WorkflowExecutionId::new())
        .await
        .expect("write execute must succeed");
    executed
        .rollback(ctx, WorkflowExecutionId::new())
        .await
        .expect("write rollback must succeed");

    assert!(!store_contains(&store, &relationship));
}

#[tokio::test]
async fn delete_execute_removes_an_existing_relationship() {
    let (spicedb, store) = SpiceDB::recording().await;
    let ctx = common::context_with_spicedb(spicedb);
    let relationship = sample_relationship();
    store
        .lock()
        .expect("store poisoned")
        .insert(relationship.to_string());

    let op = DeleteRelationshipsOp {
        relationships: vec![relationship.clone()],
    };
    op.execute(ctx, WorkflowExecutionId::new())
        .await
        .expect("delete execute must succeed");

    assert!(!store_contains(&store, &relationship));
}

#[tokio::test]
async fn delete_rollback_restores_the_relationship() {
    let (spicedb, store) = SpiceDB::recording().await;
    let ctx = common::context_with_spicedb(spicedb);
    let relationship = sample_relationship();
    store
        .lock()
        .expect("store poisoned")
        .insert(relationship.to_string());

    let op = DeleteRelationshipsOp {
        relationships: vec![relationship.clone()],
    };
    let executed = op
        .execute(ctx.clone(), WorkflowExecutionId::new())
        .await
        .expect("delete execute must succeed");
    executed
        .rollback(ctx, WorkflowExecutionId::new())
        .await
        .expect("delete rollback must succeed");

    assert!(store_contains(&store, &relationship));
}
