use hypervisor_connector_proxmox::mock::{
    MockServer, WithClusterNextId, WithClusterResourceList, WithTaskStatusReadMock,
    WithVMCreateMock, WithVMStatusStartMock, WithVMStatusStopMock,
};
use hypervisors::{
    Hypervisor,
    v1::{
        ListHypervisorsRequest, RegisterHypervisorRequest, hypervisors_client::HypervisorsClient,
    },
};
use instances::{
    Instance,
    v1::{
        CreateInstanceRequest, CreateInstanceResponse, ListInstancesRequest, StartInstanceRequest,
        StopInstanceRequest, instances_client::InstancesClient,
    },
};
use server::{Server, ServerConfig};

#[sqlx::test(migrations = "../migrations")]
async fn test_the_server_starts(pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::new(ServerConfig::new(pool)).await?;
    let shutdown_tx = server.serve_with_shutdown().await?;
    shutdown_tx.send(()).ok();
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_the_create_instance_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new()
        .await
        .with_cluster_next_id()
        .with_cluster_resource_list()
        .with_task_status_read()
        .with_vm_create();
    let hypervisor = Hypervisor {
        url: mock.url(),
        ..Default::default()
    };
    hypervisors::repository::create(&pool, &hypervisor)
        .await
        .unwrap();

    let config = ServerConfig::new(pool.clone());
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = InstancesClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client
        .create_instance(CreateInstanceRequest {
            image: String::from("debian.qcow2"),
            cpu_cores: 1,
            memory_bytes: 536870912,
            name: String::from("acme-mgs"),
            snippet: String::from("acme-snippet.yaml"),
        })
        .await;

    // Get the instance generated in the database
    let instance = &instances::repository::list(&pool).await.unwrap()[0];

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(
        response.unwrap().into_inner(),
        CreateInstanceResponse {
            id: instance.id.to_string()
        }
    );

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_instances_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new().await.with_cluster_resource_list();
    let hypervisor = Hypervisor {
        url: mock.url(),
        ..Default::default()
    };
    hypervisors::repository::create(&pool, &hypervisor)
        .await
        .unwrap();

    let config = ServerConfig::new(pool);
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = InstancesClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client.list_instances(ListInstancesRequest::default()).await;

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(response.unwrap().into_inner().instances.len(), 1);

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_the_start_instance_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new()
        .await
        .with_vm_status_start()
        .with_cluster_resource_list();
    let hypervisor = Hypervisor {
        url: mock.url(),
        ..Default::default()
    };
    hypervisors::repository::create(&pool, &hypervisor)
        .await
        .unwrap();
    let instance = Instance {
        hypervisor_id: hypervisor.id,
        distant_id: String::from("100"),
        ..Default::default()
    };
    instances::repository::create(&pool, &instance)
        .await
        .unwrap();

    let config = ServerConfig::new(pool);
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = InstancesClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client
        .start_instance(StartInstanceRequest {
            id: instance.id.to_string(),
        })
        .await;

    // Assert the result
    assert!(response.is_ok());

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_the_stop_instance_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new()
        .await
        .with_vm_status_stop()
        .with_cluster_resource_list();
    let hypervisor = Hypervisor {
        url: mock.url(),
        ..Default::default()
    };
    hypervisors::repository::create(&pool, &hypervisor)
        .await
        .unwrap();
    let instance = Instance {
        hypervisor_id: hypervisor.id,
        distant_id: String::from("100"),
        ..Default::default()
    };
    instances::repository::create(&pool, &instance)
        .await
        .unwrap();

    let config = ServerConfig::new(pool);
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = InstancesClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client
        .stop_instance(StopInstanceRequest {
            id: instance.id.to_string(),
        })
        .await;

    // Assert the result
    assert!(response.is_ok());

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_the_register_hypervisor_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new().await;
    let hypervisor = Hypervisor {
        url: mock.url(),
        ..Default::default()
    };
    hypervisors::repository::create(&pool, &hypervisor)
        .await
        .unwrap();

    let config = ServerConfig::new(pool);
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = HypervisorsClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let result = client
        .register_hypervisor(RegisterHypervisorRequest::default())
        .await;

    // Assert the result
    assert!(result.is_ok());

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_hypervisors_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new().await;
    let hypervisor = Hypervisor {
        url: mock.url(),
        ..Default::default()
    };
    hypervisors::repository::create(&pool, &hypervisor)
        .await
        .unwrap();

    let config = ServerConfig::new(pool);
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = HypervisorsClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let result = client
        .list_hypervisors(ListHypervisorsRequest::default())
        .await;

    // Assert the result
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().into_inner().hypervisors,
        vec![hypervisors::v1::Hypervisor {
            id: String::from("00000000-0000-0000-0000-000000000000"),
            storage_name: String::from(""),
            url: mock.url()
        }]
    );

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
