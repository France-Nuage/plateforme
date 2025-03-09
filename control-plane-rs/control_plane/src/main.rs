use dotenv::dotenv;
use hypervisor_connector::hypervisor::{Hypervisor, Instance, InstanceConfig, Node};
use hypervisor_connector::proxmox;
use std::env;

#[tokio::main]
async fn main() {
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

    let cluster =
        proxmox::ProxmoxCluster::new("https://pve-poc01-internal.france-nuage.fr", &client);

    let node = cluster.node("pve-node1");
    let instance = node.instance(666);

    let result = instance.status().await;
    match result {
        Ok(status) => println!("VM 666 status: {:?}", status),
        Err(err) => println!(
            "Error while reading VM 666 status, does the VM exists? error: {:?}",
            err
        ),
    }

    let result = instance.delete().await;
    match result {
        Ok(_) => println!("VM 666 has been deleted"),
        Err(err) => println!(
            "Error while deleting VM 666, does the VM exists? error: {:?}",
            err
        ),
    }

    let options = InstanceConfig {
        id: 666,
        name: "instance-from-control-plane",
    };
    let result = instance.create(&options).await;
    match result {
        Ok(_) => println!("VM 666 has been created"),
        Err(err) => println!(
            "Error while creating VM 666, does the VM exists? error: {:?}",
            err
        ),
    }

    // todo: write a simpler vm config api, and adapt in proxmox implementation
}
