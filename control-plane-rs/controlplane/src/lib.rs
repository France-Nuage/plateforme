pub mod instance_service;
pub mod server;

pub mod proto {
    tonic::include_proto!("controlplane");
}
