use futures::{StreamExt, TryStreamExt, stream};
use hypervisor_connector::{InstanceConfig, InstanceInfo, InstanceService};
use hypervisors::HypervisorsService;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{model::Instance, problem::Problem, repository};

pub struct InstancesService {
    hypervisors_service: HypervisorsService,
    pool: PgPool,
}

impl InstancesService {
    pub async fn list(&self) -> Result<Vec<InstanceInfo>, Problem> {
        let hypervisors = self.hypervisors_service.list().await?;

        let instances: Vec<InstanceInfo> = stream::iter(hypervisors)
            .map(|hypervisor| async move {
                hypervisor_connector_resolver::resolve_for_hypervisor(&hypervisor)
                    .list()
                    .await
            })
            .buffer_unordered(4)
            .try_collect::<Vec<Vec<InstanceInfo>>>()
            .await?
            .into_iter()
            .flatten()
            .collect();

        Ok(instances)
    }

    pub async fn create(&self, options: InstanceConfig) -> Result<Instance, Problem> {
        let hypervisors = self.hypervisors_service.list().await?;
        let hypervisor = &hypervisors[0];

        let result = hypervisor_connector_resolver::resolve_for_hypervisor(hypervisor)
            .create(options)
            .await?;

        let instance = Instance {
            id: Uuid::new_v4(),
            hypervisor_id: hypervisor.id,
            distant_id: result,
        };
        repository::create(&self.pool, &instance).await?;

        Ok(instance)
    }

    pub async fn start(&self, id: Uuid) -> Result<(), Problem> {
        let instance = repository::read(&self.pool, id).await?;
        let hypervisor = self.hypervisors_service.read(id).await?;
        let connector = hypervisor_connector_resolver::resolve_for_hypervisor(&hypervisor);
        connector
            .start(&instance.distant_id)
            .await
            .map_err(Problem::from)
    }

    pub async fn stop(&self, id: Uuid) -> Result<(), Problem> {
        let instance = repository::read(&self.pool, id).await?;
        let hypervisor = self.hypervisors_service.read(id).await?;
        let connector = hypervisor_connector_resolver::resolve_for_hypervisor(&hypervisor);
        connector
            .stop(&instance.distant_id)
            .await
            .map_err(Problem::from)
    }

    pub fn new(pool: PgPool) -> Self {
        Self {
            hypervisors_service: HypervisorsService::new(pool.clone()),
            pool,
        }
    }
}
