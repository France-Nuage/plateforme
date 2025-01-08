use uuid::Uuid;
use sqlx::{query_as, PgPool};
use strum::VariantNames;
use sqlx::Type;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct Node {
    id: Uuid,
    name: String,
    token: String,
    url: String,
}

#[derive(Debug)]
pub struct InstanceRecord {
    id: Uuid,
    pve_vm_id: Option<i32>,
    status: Option<StatusFsm>,
    node: Option<Node>,
}

#[derive(Debug, Clone, Copy, strum::Display, VariantNames, Type)]
#[sqlx(type_name = "text")]
pub enum StatusFsm {
    #[strum(serialize = "PROVISIONING")]
    PROVISIONING,
    #[strum(serialize = "STAGING")]
    STAGING,
    #[strum(serialize = "STOPPING")]
    STOPPING,
    #[strum(serialize = "DELETING")]
    DELETING,
}

pub async fn get_vm_instances(pool: PgPool) -> Result<Vec<InstanceRecord>, Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;

    struct RecordSQLResult {
        id: Uuid,
        status: Option<StatusFsm>,
        pve_vm_id: Option<i32>,
        node_id: Option<Uuid>,
        node_name: Option<String>,
        node_url: Option<String>,
        node_token: Option<String>,
    }

    let records_raw = query_as!(
        RecordSQLResult,
        r#"
            SELECT
                inst.instance__id AS id,
                inst.status AS "status?: StatusFsm",
                CAST(pve_vm_id AS integer) AS pve_vm_id,
                nd.node__id AS node_id,
                nd.name AS node_name,
                nd.token AS node_token,
                nd.url AS node_url
            FROM infrastructure.instances AS inst
            LEFT JOIN infrastructure.nodes AS nd
                ON nd.node__id = inst.node__id
            WHERE inst.status in ('PROVISIONING', 'STAGING', 'STOPPING', 'DELETING')
            FOR UPDATE OF inst
            SKIP LOCKED
        "#
    )
    .fetch_all(&mut *tx)
    .await?;

    let records = records_raw
        .into_iter()
        .map(|raw| {
            let node = match (raw.node_id, raw.node_name, raw.node_token, raw.node_url) {
                (Some(nid), Some(nname), Some(ntoken), Some(nurl)) => Some(Node {
                    id: nid,
                    name: nname,
                    token: ntoken,
                    url: nurl,
                }),
                _ => None,
            };
            InstanceRecord {
                id: raw.id,
                pve_vm_id: raw.pve_vm_id,
                status: raw.status,
                node,
            }
        })
        .collect::<Vec<InstanceRecord>>();


    println!("Processing {} records", records.len());
    sleep(Duration::from_millis(8)).await;

    for record in &records {
        if let Some(node) = &record.node {
            if let Some(_pve_vm_id) = record.pve_vm_id {
                if let Some(s) = record.status {
                    process_instance_from_status(record, s).await?;
                }
            }
        }
    }

    tx.commit().await?;
    Ok(records)
}

pub async fn process_instance_from_status(
    instance: &InstanceRecord,
    status: StatusFsm,
) -> Result<(), Box<dyn std::error::Error>> {
    match status {
        StatusFsm::PROVISIONING => {
            println!("Processing instance {} with status {}", instance.id, status);
        }
        StatusFsm::STAGING => {
            println!("Processing instance {} with status {}", instance.id, status);
        }
        StatusFsm::STOPPING => {
            println!("Processing instance {} with status {}", instance.id, status);
        }
        StatusFsm::DELETING => {
            println!("Deleting instance {}", instance.id);
        }
    }
    Ok(())
}

pub async fn retrieve_vm_status_from_hypervisor(
    node: &Node,
    pve_vm_id: i32,
) -> Result<StatusFsm, Box<dyn std::error::Error>> {
    println!("Retrieving status for VM {} on node {}", pve_vm_id, node.name);
    Ok(StatusFsm::PROVISIONING)
}
