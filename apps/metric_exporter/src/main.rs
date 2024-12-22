use tokio::time::interval;
use std::time::Duration;
use sqlx::postgres::PgPoolOptions;
use sqlx::query_as;
use std::env;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug)]
struct Node {
    id: Uuid,
    name: String,
    token: String,
    url: String,
}

#[derive(Debug)]
struct VmRecord {
    id: Uuid,
    name: Option<std::string::String>,
    pve_vm_id: Option<i32>,
    node: Option<Node>,
}

async fn get_vm_records() -> Result<Vec<VmRecord>, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:5433/postgres".to_string());

    // Création d’une PoolConnection
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    struct VmRecordRaw {
      id: Uuid,
      name: Option<String>,
      pve_vm_id: Option<i32>,
      node_id: Option<Uuid>,
      node_name: Option<String>,
      node_url: Option<String>,
      node_token: Option<String>,
    }

    // fetch_all pour récupérer *tous* les enregistrements
    // Si tu veux un seul enregistrement, utilise fetch_one
    let records_raw = query_as!(
        VmRecordRaw,
        r#"
            SELECT instance__id AS id, vms.name, CAST(pve_vm_id AS integer) as pve_vm_id, nd.node__id as node_id, nd.name as node_name, nd.token as node_token, nd.url as node_url
            FROM infrastructure.instances as vms
                 LEFT JOIN infrastructure.nodes as nd ON nd.node__id = vms.node__id
        "#,
    )
    .fetch_all(&pool)
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
                VmRecord {
                    id: raw.id,
                    name: raw.name,
                    pve_vm_id: raw.pve_vm_id,
                    node,
                }
            })
            .collect::<Vec<VmRecord>>();

    Ok(records)
}

#[derive(Debug, Deserialize)]
struct VMMetricResponse {
    data: VMData,
}

#[derive(Debug, Deserialize)]
struct VMData {
    status: String,
    disk: u64,
    maxmem: u64,
    netin: u64,
    ha: HA,
    diskwrite: u64,
    cpus: u64,
    name: String,
    maxdisk: u64,
    netout: u64,
    mem: u64,
    vmid: i32,
    cpu: f64,
    uptime: u64,
    diskread: u64,
    qmpstatus: String,
}

#[derive(Debug, Deserialize)]
struct HA {
    managed: u64,
}

async fn get_vm_metrics_from_proxmox(node: &Node, vm_id: i32) -> Result<VMMetricResponse, reqwest::Error> {
    let url = format!("{}/api2/json/nodes/{}/qemu/{}/status/current", node.url, node.name, vm_id);

    let client = reqwest::Client::new();
    let response = client.get(&url).header("Authorization", format!("{}", node.token)).send().await?;
    let body = response.text().await?;

    let vm_status: VMMetricResponse = serde_json::from_str(&body)
            .expect("Impossible de parser le JSON");

    Ok(vm_status)
}

#[derive(Debug, Serialize)]
struct MimirPayload {
    streams: Vec<MimirStream>,
}

#[derive(Debug, Serialize)]
struct MimirStream {
    metric: serde_json::Value,
    values: Vec<(String, String)>,
}

fn build_mimir_payload(vmid: Uuid, metric: VMMetricResponse) -> MimirPayload {
    let timestamp = Utc::now().timestamp_millis().to_string();
    let cpu_metric = serde_json::json!({
            "__name__": "vm_cpu_usage",
            "vm_id": vmid,
//             "project": vm.project,
//             "folder": vm.folder,
//             "organization": vm.organization
        });
        let mem_metric = serde_json::json!({
            "__name__": "vm_mem_usage",
            "vm_id": vmid,
//             "project": vm.project,
//             "folder": vm.folder,
//             "organization": vm.organization
        });

        let cpu_stream = MimirStream {
                metric: cpu_metric,
                values: vec![(timestamp.clone(), format!("{}", metric.data.cpu))],
            };

        let mem_stream = MimirStream {
            metric: mem_metric,
            values: vec![(timestamp, format!("{}", metric.data.mem))],
        };

        MimirPayload {
            streams: vec![cpu_stream, mem_stream],
        }
}

async fn push_metric_to_mimir(payload: &MimirPayload) -> Result<(), Box<dyn std::error::Error>> {

    let mimir_url = env::var("MIMIR_URL")
        .unwrap_or_else(|_| "https://localhost:8080".to_string());
    let url = format!("{}/api/v1/push", mimir_url);
        println!("Url {:?}", url);
        println!("Payload {:?}", payload);

    let client = reqwest::Client::new();
    let response = client
            .post(url)
            .header("X-Prometheus-Remote-Write-Version", "0.1.0")
            .json(payload)
            .send()
            .await?
            .error_for_status()?;

        println!("Plop : {:?}", response);
//         let status = response.status();
//         println!("Métriques envoyées à Mimir (status = {}).", status);
        Ok(())
}

#[tokio::main]
async fn main() {
    let mut ticker = interval(Duration::from_secs(5));

    loop {
        ticker.tick().await;
        let records = get_vm_records().await.unwrap();

        for record in records {
            if let Some(node) = &record.node {
                if let Some(pve_vm_id) = record.pve_vm_id {

                    let metric = get_vm_metrics_from_proxmox(node, pve_vm_id).await;
                    let payload = build_mimir_payload(record.id, metric.unwrap());

                    match push_metric_to_mimir(&payload).await {
                        Ok(_) => {
                            println!("Métriques envoyées avec succès !");
                        },
                        Err(e) => {
                            eprintln!("Erreur lors de l'envoi des métriques : {}", e);
                            // éventuellement : return Err(e)
                        }
                    }
                }
            }
        }

    }
}
