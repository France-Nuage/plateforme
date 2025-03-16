use dotenv::dotenv;
use hypervisor::{Cluster, Instance, InstanceConfig, Node};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let authentication_header =
        env::var("AUTHENTICATION_HEADER").expect("Missing env var AUTHENTICATION_HEADER");

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(authentication_header.as_ref()).unwrap(),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    let cluster = proxmox::Cluster::new("https://pve-poc01-internal.france-nuage.fr", &client);

    let node = cluster.node("pve-node1");
    let instance = node.instance(666);

    let result = instance.status().await?;
    println!("VM 666 status: {:?}", result);

    instance.delete().await?;
    println!("VM 666 has been deleted");

    let options = InstanceConfig {
        id: 666,
        name: "instance-from-control-plane",
    };
    instance.create(&options).await?;
    println!("VM 666 has been created");

    Ok(())
    // todo: write a simpler vm config api, and adapt in proxmox implementation
}
