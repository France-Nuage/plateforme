use crate::{error::Error, model::Instance, repository};
use database::Persistable;
use frn_core::authorization::{Authorize, Principal, Relation, Relationship};
use frn_core::compute::{Hypervisor, Hypervisors};
use frn_core::resourcemanager::{Project, Projects};
use futures::{StreamExt, TryStreamExt, stream};
use hypervisor::instance::{InstanceCreateRequest, Instances};
use sqlx::{PgPool, types::chrono};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct InstancesService<Auth: Authorize> {
    hypervisors: Hypervisors<Auth>,
    projects: Projects<Auth>,
    pool: PgPool,
}

impl<Auth: Authorize> InstancesService<Auth> {
    pub async fn list(&self) -> Result<Vec<Instance>, Error> {
        Instance::list(&self.pool).await.map_err(Into::into)
    }

    pub async fn sync<P: Principal>(&mut self, principal: &P) -> Result<Vec<Instance>, Error> {
        let hypervisors = self.hypervisors.list(principal).await?;
        let mut instances: Vec<Instance> = Vec::new();
        for hypervisor in hypervisors {
            let retrieved = self
                .sync_hypervisor_instances(principal, &hypervisor)
                .await?;
            instances.extend(retrieved);
        }

        Ok(instances)
    }

    pub async fn sync_hypervisor_instances<P: Principal>(
        &mut self,
        principal: &P,
        hypervisor: &Hypervisor,
    ) -> Result<Vec<Instance>, Error> {
        // Get the default project for the hypervisor
        let default_project = self
            .projects
            .get_default_project(principal, &hypervisor.organization_id)
            .await?;

        // Get a instance_service instance
        let instance_service = Arc::new(hypervisor::resolve(
            hypervisor.url.clone(),
            hypervisor.authorization_token.clone(),
        ));

        // Retrieve the distant instances
        let distant_instances = instance_service.list().await?;

        let instances = stream::iter(distant_instances)
            .map(|distant_instance| {
                let instance_service = instance_service.clone();
                let pool = self.pool.clone();
                async move {
                    let result =
                        repository::find_one_by_distant_id(&pool, &distant_instance.id).await;

                    match result {
                        Ok(maybe_instance) => {
                            let mut existing = maybe_instance.unwrap_or(Instance {
                                id: Uuid::new_v4(),
                                project_id: default_project.id,
                                ..Default::default()
                            });

                            // Try to retrieve the ip address if it is not known yet
                            if existing.ip_v4 == *"" {
                                let ip = match instance_service
                                    .get_ip_address(&distant_instance.id)
                                    .await
                                {
                                    Ok(value) => Ok(value),
                                    Err(hypervisor::Error::InstanceNotRunning(_)) => Ok(None),
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
                        Err(err) => Err(Error::from(err)),
                    }
                }
            })
            .buffer_unordered(4)
            .try_collect::<Vec<Instance>>()
            .await?;

        let instances = repository::upsert(&self.pool, &instances).await?;

        for instance in &instances {
            Relationship::new(
                &Project {
                    id: instance.project_id,
                    ..Default::default()
                },
                Relation::Parent,
                instance,
            )
            .publish(&self.pool)
            .await?;
        }

        Ok(instances)
    }

    pub async fn clone_instance<P: Principal>(
        &mut self,
        id: Uuid,
        principal: &P,
    ) -> Result<Instance, Error> {
        let existing = repository::read(&self.pool, id).await?;
        let hypervisor = self
            .hypervisors
            .read(principal, existing.hypervisor_id)
            .await?;
        let connector = hypervisor::resolve(hypervisor.url, hypervisor.authorization_token);
        let new_id = connector.clone(&existing.distant_id).await?;

        let instance = Instance {
            id: Uuid::new_v4(),
            distant_id: new_id,
            ..existing
        };

        instance.create(&self.pool).await.map_err(Into::into)
    }

    pub async fn create<P: Principal>(
        &mut self,
        options: InstanceCreateRequest,
        project_id: Uuid,
        principal: &P,
    ) -> Result<Instance, Error> {
        let hypervisors = self.hypervisors.list(principal).await?;
        let hypervisor = &hypervisors
            .first()
            .ok_or_else(|| Error::NoHypervisorsAvaible)?;

        let result = hypervisor::resolve(
            hypervisor.url.clone(),
            hypervisor.authorization_token.clone(),
        )
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

    pub async fn delete<P: Principal>(&mut self, principal: &P, id: Uuid) -> Result<(), Error> {
        let instance = repository::read(&self.pool, id).await?;
        let hypervisor = self
            .hypervisors
            .read(principal, instance.hypervisor_id)
            .await
            .inspect_err(|err| println!("received error from hp read: {:#?}", err))?;

        let connector = hypervisor::resolve(hypervisor.url, hypervisor.authorization_token);

        connector.delete(&instance.distant_id).await?;

        sqlx::query!("DELETE FROM instances WHERE id = $1", instance.id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn start<P: Principal>(&self, principal: &P, id: Uuid) -> Result<(), Error> {
        let instance = repository::read(&self.pool, id).await?;
        let hypervisor = self
            .hypervisors
            .clone()
            .read(principal, instance.hypervisor_id)
            .await?;
        let connector = hypervisor::resolve(hypervisor.url, hypervisor.authorization_token);
        connector
            .start(&instance.distant_id)
            .await
            .map_err(Error::from)
    }

    pub async fn stop<P: Principal>(&self, principal: &P, id: Uuid) -> Result<(), Error> {
        let instance = repository::read(&self.pool, id).await?;
        let hypervisor = self
            .hypervisors
            .clone()
            .read(principal, instance.hypervisor_id)
            .await?;
        let connector = hypervisor::resolve(hypervisor.url, hypervisor.authorization_token);
        connector
            .stop(&instance.distant_id)
            .await
            .map_err(Error::from)
    }

    pub fn new(pool: PgPool, hypervisors: Hypervisors<Auth>, projects: Projects<Auth>) -> Self {
        Self {
            hypervisors,
            projects,
            pool,
        }
    }
}
