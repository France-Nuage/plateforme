use crate::{model::Instance, problem::Problem, repository};
use auth::{Relation, Relationship};
use database::Persistable;
use frn_core::{iam::Authorize, resourcemanager::Project};
use futures::{StreamExt, TryStreamExt, stream};
use hypervisor_connector::{InstanceConfig, InstanceService};
use hypervisors::{Hypervisor, HypervisorsService};
use resources::service::ResourcesService;
use sqlx::{PgPool, types::chrono};
use std::sync::Arc;
use uuid::Uuid;

pub struct InstancesService {
    hypervisors_service: HypervisorsService,
    resources_service: ResourcesService,
    pool: PgPool,
}

impl InstancesService {
    pub async fn list(&self) -> Result<Vec<Instance>, Problem> {
        Instance::list(&self.pool).await.map_err(Into::into)
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
        // Get the default project for the hypervisor
        let default_project = self
            .resources_service
            .get_default_project(&hypervisor.organization_id)
            .await?;

        // Get a instance_service instance
        let instance_service = Arc::new(hypervisor_connector_resolver::resolve_for_hypervisor(
            hypervisor,
        ));

        // Retrieve the distant instances
        let distant_instances = instance_service.list().await?;

        let instances = stream::iter(distant_instances)
            .map(|distant_instance| {
                let instance_service = instance_service.clone();
                async move {
                    let result =
                        repository::find_one_by_distant_id(&self.pool, &distant_instance.id).await;

                    match result {
                        Ok(maybe_instance) => {
                            let mut existing = maybe_instance.unwrap_or(Instance {
                                id: Uuid::new_v4(),
                                project_id: default_project.id,
                                ..Default::default()
                            });

                            // Try to retrieve the ip address if it is not known yet
                            if existing.ip_v4 == *"0.0.0.0" {
                                let ip = match instance_service
                                    .get_ip_address(&distant_instance.id)
                                    .await
                                {
                                    Ok(value) => Ok(value),
                                    Err(hypervisor_connector::Problem::InstanceNotRunning(_)) => {
                                        Ok(None)
                                    }
                                    Err(err) => Err(err),
                                }?;

                                if let Some(ip) = ip {
                                    existing.ip_v4 = ip.to_string();
                                }
                            }

                            Ok(Instance {
                                id: existing.id,
                                hypervisor_id: hypervisor.id,
                                project_id: existing.project_id,
                                zero_trust_network_id: existing.zero_trust_network_id,
                                distant_id: distant_instance.id,
                                cpu_usage_percent: distant_instance.cpu_usage_percent as f64,
                                disk_usage_bytes: distant_instance.disk_usage_bytes as i64,
                                ip_v4: existing.ip_v4,
                                max_cpu_cores: distant_instance.max_cpu_cores as i32,
                                max_disk_bytes: distant_instance.max_disk_bytes as i64,
                                max_memory_bytes: distant_instance.max_memory_bytes as i64,
                                memory_usage_bytes: distant_instance.memory_usage_bytes as i64,
                                name: distant_instance.name,
                                status: distant_instance.status.into(),
                                created_at: chrono::Utc::now(),
                                updated_at: chrono::Utc::now(),
                            })
                        }
                        Err(err) => Err(Problem::from(err)),
                    }
                }
            })
            .buffer_unordered(4)
            .try_collect::<Vec<Instance>>()
            .await?;

        let instances = repository::upsert(&self.pool, &instances).await?;

        for instance in &instances {
            Relationship::new(
                instance.resource(),
                Relation::BelongsToProject,
                (Project::resource_name(), &instance.project_id),
            )
            .publish(&self.pool)
            .await?;
        }

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

        instance.create(&self.pool).await.map_err(Into::into)
    }

    pub async fn create(
        &self,
        options: InstanceConfig,
        project_id: Uuid,
    ) -> Result<Instance, Problem> {
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
            project_id,
            distant_id: result,
            ..Default::default()
        };

        instance.create(&self.pool).await.map_err(Into::into)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), Problem> {
        let instance = repository::read(&self.pool, id).await?;
        let hypervisor = self
            .hypervisors_service
            .read(instance.hypervisor_id)
            .await?;
        let connector = hypervisor_connector_resolver::resolve_for_hypervisor(&hypervisor);
        connector
            .delete(&instance.distant_id)
            .await
            .map_err(Problem::from)
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
            resources_service: ResourcesService::new(pool.clone()),
            pool,
        }
    }
}
