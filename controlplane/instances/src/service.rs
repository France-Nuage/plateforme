use std::sync::Arc;

use hypervisor_connector::{InstanceConfig, InstanceInfo, InstanceService};
use hypervisors::HypervisorsService;
use sea_orm::{DatabaseConnection, DbErr};
use uuid::Uuid;

use crate::problem::Problem;

pub struct InstancesService {
    hypervisors_service: HypervisorsService,
}

impl InstancesService {
    pub async fn list(&self) -> Result<Vec<InstanceInfo>, DbErr> {
        let hypervisors = self.hypervisors_service.list().await?;
        let mut all_instances = Vec::new();

        let client = reqwest::Client::new();
        for hypervisor in hypervisors {
            let instances = hypervisor_connector_resolver::resolve(
                hypervisor.url,
                client.clone(),
                hypervisor.authentication_token,
            )
            .list()
            .await
            .unwrap();

            all_instances.extend(instances);
        }
        Ok(all_instances)
    }

    pub async fn create(&self, options: InstanceConfig) -> Result<crate::model::Model, Problem> {
        let hypervisors = self.hypervisors_service.list().await?;
        let hypervisor = hypervisors[0].clone();
        hypervisor_connector_resolver::resolve_model(hypervisor)
            .create(options)
            .await?;
        todo!()
    }

    pub async fn start(&self, id: Uuid) -> Result<(), Problem> {
        todo!()
    }

    pub async fn stop(&self, id: Uuid) -> Result<(), Problem> {
        todo!()
    }

    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            hypervisors_service: HypervisorsService::new(db),
        }
    }
}
