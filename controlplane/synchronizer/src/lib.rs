use frn_core::authorization::{Relation, Relationship, Resource};
use frn_core::resourcemanager::Project;
use frn_core::{
    App, authorization::Authorize, compute::Instance, identity::ServiceAccount,
    resourcemanager::Organization,
};
use futures::{StreamExt, TryStreamExt, stream};
use hypervisor::instance::Instances;
use sqlx::types::Uuid;

mod error;
use error::Error;

pub async fn synchronize<Auth: Authorize>(app: &mut App<Auth>) -> Result<(), Error> {
    let principal = ServiceAccount::default();

    // Retrieve the connected hypervisors
    let hypervisors = app.hypervisors.list(&principal).await?;

    for hypervisor in hypervisors {
        let service = hypervisor::resolve(
            hypervisor.url.clone(),
            hypervisor.authorization_token.clone(),
        );
        let root_organization: Organization =
            sqlx::query_as("SELECT id, name, slug, parent_id, created_at, updated_at FROM organizations WHERE name = $1")
                .bind(&app.config.root_organization.name)
                .fetch_one(&app.db)
                .await?;
        let default_project = app
            .projects
            .get_default_project(&principal, &root_organization.id)
            .await?;

        let distant_instances = service.list().await?;
        let instances = stream::iter(distant_instances)
            .map(|distant| {
                let pool = app.db.clone();
                let service = Clone::clone(&service);

                async move {
                    let mut existing = sqlx::query_as!(
                        Instance,
                        "SELECT * FROM instances WHERE distant_id = $1 AND hypervisor_id = $2",
                        distant.id,
                        hypervisor.id
                    )
                    .fetch_optional(&pool)
                    .await?
                    .unwrap_or(Instance {
                        id: Uuid::new_v4(),
                        project_id: default_project.id,
                        ..Default::default()
                    });

                    // Try to retrieve the ip address if it is not known yet
                    if existing.ip_v4.is_empty() {
                        let ip = match service.get_ip_address(&distant.id).await {
                            Ok(value) => Ok(value),
                            Err(hypervisor::Error::InstanceNotRunning(_)) => Ok(None),
                            Err(err) => Err(err),
                        }?;

                        if let Some(ip) = ip {
                            existing.ip_v4 = ip.to_string();
                        }
                    }

                    Ok::<Instance, Error>(Instance {
                        id: existing.id,
                        hypervisor_id: hypervisor.id,
                        project_id: existing.project_id,
                        zero_trust_network_id: existing.zero_trust_network_id,
                        distant_id: distant.id,
                        cpu_usage_percent: distant.cpu_usage_percent as f64,
                        disk_usage_bytes: distant.disk_usage_bytes as i64,
                        ip_v4: existing.ip_v4,
                        max_cpu_cores: distant.max_cpu_cores as i32,
                        max_disk_bytes: distant.max_disk_bytes as i64,
                        max_memory_bytes: distant.max_memory_bytes as i64,
                        memory_usage_bytes: distant.memory_usage_bytes as i64,
                        name: distant.name,
                        status: distant.status,
                        created_at: existing.created_at,
                        updated_at: existing.updated_at,
                    })
                }
            })
            .buffer_unordered(4)
            .try_collect::<Vec<Instance>>()
            .await?;

        let instances = Instance::upsert(&app.db, &instances).await?;

        for instance in &instances {
            Relationship::new(
                &Project::some(instance.project_id),
                Relation::Parent,
                instance,
            )
            .publish(&app.db)
            .await?;
        }
    }

    Ok(())
}

pub async fn heartbeat(client: &reqwest::Client, url: &Option<String>) {
    if let Some(url) = url
        && let Err(e) = client.get(url).send().await
    {
        tracing::error!(error = %e, "Failed to ping heartbeat");
    }
}
