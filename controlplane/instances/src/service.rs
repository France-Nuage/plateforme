use futures::{StreamExt, TryStreamExt, stream};
use hypervisor_connector::{InstanceConfig, InstanceService};
use hypervisors::{Hypervisor, HypervisorsService};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{model::Instance, problem::Problem, repository};

pub struct InstancesService {
    hypervisors_service: HypervisorsService,
    pool: PgPool,
}

impl InstancesService {
    pub async fn list(&self) -> Result<Vec<Instance>, Problem> {
        self.sync().await
    }

    pub async fn sync(&self) -> Result<Vec<Instance>, Problem> {
        let hypervisors = self.hypervisors_service.list().await?;
        let instances = stream::iter(hypervisors)
            .map(|hypervisor| async move { self.sync_hypervisor_instances(&hypervisor).await })
            .buffer_unordered(4)
            .try_collect::<Vec<Vec<Instance>>>()
            .await?
            .into_iter()
            .flatten()
            .collect();

        Ok(instances)
    }

    pub async fn sync_hypervisor_instances(
        &self,
        hypervisor: &Hypervisor,
    ) -> Result<Vec<Instance>, Problem> {
        let distant_instances = hypervisor_connector_resolver::resolve_for_hypervisor(hypervisor)
            .list()
            .await?;

        let instances = stream::iter(distant_instances)
            .map(|distant_instance| async move {
                repository::find_one_by_distant_id(&self.pool, &distant_instance.id)
                    .await
                    .map(|result| {
                        let existing = result.unwrap_or(Instance {
                            id: Uuid::new_v4(),
                            ..Default::default()
                        });

                        Instance {
                            id: existing.id,
                            hypervisor_id: hypervisor.id,
                            ..distant_instance.into()
                        }
                    })
            })
            .buffer_unordered(4)
            .try_collect::<Vec<Instance>>()
            .await?;

        repository::upsert(&self.pool, &instances).await?;

        Ok(instances)
    }

    pub async fn clone(&self, id: Uuid) -> Result<Instance, Problem> {
        let existing = repository::read(&self.pool, id).await?;
        let hypervisor = self
            .hypervisors_service
            .read(existing.hypervisor_id)
            .await?;
        let connector = hypervisor_connector_resolver::resolve_for_hypervisor(&hypervisor);
        let new_id = connector.clone(&existing.distant_id).await?;

        let instance = Instance {
            id: Uuid::new_v4(),
            distant_id: new_id,
            ..existing
        };
        repository::create(&self.pool, &instance).await?;

        Ok(instance)
    }

    pub async fn create(&self, options: InstanceConfig) -> Result<Instance, Problem> {
        let hypervisors = self.hypervisors_service.list().await?;
        let hypervisor = &hypervisors
            .first()
            .ok_or_else(|| Problem::NoHypervisorsAvaible)?;

        let result = hypervisor_connector_resolver::resolve_for_hypervisor(hypervisor)
            .create(options)
            .await?;

        let instance = Instance {
            id: Uuid::new_v4(),
            hypervisor_id: hypervisor.id,
            distant_id: result,
            ..Default::default()
        };
        repository::create(&self.pool, &instance).await?;

        Ok(instance)
    }

    pub async fn start(&self, id: Uuid) -> Result<(), Problem> {
        let instance = repository::read(&self.pool, id).await?;
        let hypervisor = self
            .hypervisors_service
            .read(instance.hypervisor_id)
            .await?;
        let connector = hypervisor_connector_resolver::resolve_for_hypervisor(&hypervisor);
        connector
            .start(&instance.distant_id)
            .await
            .map_err(Problem::from)
    }

    pub async fn stop(&self, id: Uuid) -> Result<(), Problem> {
        let instance = repository::read(&self.pool, id).await?;
        let hypervisor = self
            .hypervisors_service
            .read(instance.hypervisor_id)
            .await?;
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
