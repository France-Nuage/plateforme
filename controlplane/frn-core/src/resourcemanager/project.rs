use crate::Error;
use crate::authorization::{Authorize, Permission, Principal, Relation, Relationship};
use crate::resourcemanager::Organization;
use fabrique::{Factory, Model, Query};
use frn_core::authorization::Resource;
use sqlx::types::chrono;
use sqlx::{Pool, Postgres};

pub const DEFAULT_PROJECT_NAME: &str = "unattributed";

#[derive(Debug, Default, Factory, Model, Resource)]
pub struct Project {
    /// The project slug (CITEXT primary key)
    #[fabrique(primary_key)]
    #[resource(id)]
    pub slug: String,

    /// The project name
    pub name: String,

    /// The organization this project belongs to
    pub organization_slug: String,

    /// Creation time of the project
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last update time of the project
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct ProjectCreateRequest {
    pub name: String,
    pub organization_slug: String,
}

/// Generate a project slug: {org_slug}-{name_slug}, max 49 chars, letters and hyphens only.
///
/// The org_slug prefix is truncated to leave at least 2 chars for the name
/// part (1 separator + 1 char minimum), so distinct project names always
/// produce distinct slugs even under very long organization slugs.
pub fn generate_project_slug(org_slug: &str, project_name: &str) -> String {
    use crate::resourcemanager::organization::collapse_and_trim_slug;

    let name_raw: String = slug::slugify(project_name)
        .chars()
        .filter(|c| c.is_ascii_alphabetic() || *c == '-')
        .collect();
    let name_part = collapse_and_trim_slug(&name_raw, 49);

    let max_prefix_len = 49_usize
        .saturating_sub(1)
        .saturating_sub(name_part.len())
        .max(1);
    let prefix: String = org_slug.chars().take(max_prefix_len).collect::<String>();
    let prefix = prefix.trim_end_matches('-');

    let full = format!("{}-{}", prefix, name_part);
    full.chars()
        .take(49)
        .collect::<String>()
        .trim_end_matches('-')
        .to_string()
}

#[derive(Clone)]
pub struct Projects<Auth: Authorize> {
    auth: Auth,
    db: Pool<Postgres>,
}

impl<Auth: Authorize> Projects<Auth> {
    pub fn new(auth: Auth, db: Pool<Postgres>) -> Self {
        Self { auth, db }
    }

    pub async fn list<P: Principal>(&mut self, principal: &P) -> Result<Vec<Project>, Error> {
        self.auth
            .lookup::<Project>()
            .on_behalf_of(principal)
            .with(Permission::Get)
            .against(&self.db)
            .await
    }

    pub async fn create<P: Principal>(
        &mut self,
        _principal: &P,
        request: ProjectCreateRequest,
    ) -> Result<Project, Error> {
        let slug = generate_project_slug(&request.organization_slug, &request.name);

        let project = Project::factory()
            .slug(slug)
            .name(request.name)
            .organization_slug(request.organization_slug.clone())
            .create(&self.db)
            .await?;

        self.auth
            .write_relationship(&Relationship::new(
                &Organization::some(request.organization_slug),
                Relation::Parent,
                &project,
            ))
            .await?;

        Ok(project)
    }

    pub async fn get_default_project<P: Principal>(
        &mut self,
        _principal: &P,
        organization_slug: &str,
    ) -> Result<Project, Error> {
        Project::query()
            .select()
            .r#where(
                Project::ORGANIZATION_SLUG,
                "=",
                organization_slug.to_owned(),
            )
            .r#where(Project::NAME, "=", DEFAULT_PROJECT_NAME.to_owned())
            .first_or_fail(&self.db)
            .await
            .map_err(Into::into)
    }
}
