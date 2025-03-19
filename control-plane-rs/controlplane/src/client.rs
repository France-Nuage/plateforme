use controlplane::InstanceStatusRequest;
use controlplane::instance_client::InstanceClient;

mod controlplane {
    tonic::include_proto!("controlplane");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = InstanceClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(InstanceStatusRequest {
        id: String::from("666"),
    });

    let response = client.status(request).await?;

    println!("response: {:?}", response);
    Ok(())
}
