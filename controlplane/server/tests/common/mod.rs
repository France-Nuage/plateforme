use auth::mock::WithWellKnown;
use frn_core::identity::ServiceAccount;
use frn_rpc::v1::compute::hypervisors_client::HypervisorsClient;
use mock_server::MockServer;
use server::{Config, error::Error};
use sqlx::{Pool, Postgres};
use std::str::FromStr;
use tokio::sync::oneshot;
use tonic::{Request, metadata::MetadataValue, transport::Channel};

/// gRPC clients for compute services.
pub struct Compute {
    pub hypervisors: HypervisorsClient<Channel>,
}

impl Compute {
    pub async fn create(dst: String) -> Result<Self, Error> {
        let hypervisors = HypervisorsClient::connect(dst).await?;

        Ok(Self { hypervisors })
    }
}

/// Test API wrapper that manages a gRPC server lifecycle.
pub struct Api {
    pub compute: Compute,
    pub service_account: ServiceAccount,
    shutdown: Option<oneshot::Sender<()>>,
}

impl Api {
    /// Starts a test server with an in-memory database and mock authentication.
    pub async fn start(pool: &Pool<Postgres>) -> Result<Self, Error> {
        let mock = MockServer::new().await.with_well_known();
        let config = Config::test(pool, &mock).await?;
        let server_url = format!("http://{}", config.addr);
        let shutdown = server::serve(config).await?;

        Ok(Self {
            compute: Compute::create(server_url).await?,
            service_account: ServiceAccount::default(),
            shutdown: Some(shutdown),
        })
    }
}

impl Drop for Api {
    fn drop(&mut self) {
        // Ensure server shutdown to prevent port conflicts in subsequent tests
        if let Some(tx) = self.shutdown.take() {
            tx.send(()).expect("failed to send shutdown signal");
        }
    }
}

/// Adds authentication headers to gRPC requests.
pub trait OnBehalfOf {
    /// Attaches a service account's bearer token to the request.
    fn on_behalf_of(self, principal: &ServiceAccount) -> Self;
}

impl<T> OnBehalfOf for Request<T> {
    fn on_behalf_of(mut self, principal: &ServiceAccount) -> Self {
        let metadata_value = MetadataValue::from_str(&format!("Bearer {}", &principal.key))
            .expect("could not create metadata value from service account key");
        self.metadata_mut().insert("authorization", metadata_value);

        self
    }
}
