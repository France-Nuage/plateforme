use crate::Error;
use crate::authorization::{Authorize, Permission, Principal, Relation, Relationship, Resource};
use crate::identity::{ServiceAccount, User};
use crate::resourcemanager::{DEFAULT_PROJECT_NAME, Project, generate_project_slug};
use fabrique::{Factory, Model, Persist, Query};
use sqlx::types::chrono;
use sqlx::{Pool, Postgres};

#[derive(Debug, Default, Factory, Model, Resource)]
pub struct Organization {
    /// The organization slug (CITEXT primary key)
    #[fabrique(primary_key)]
    #[resource(id)]
    pub slug: String,
    /// The organization name
    pub name: String,
    /// The parent organization slug, if any
    pub parent_slug: Option<String>,
    /// Creation time of the organization
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update time of the organization
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Generate a slug from a name: lowercase letters and hyphens only, max 49 chars.
pub fn generate_organization_slug(name: &str) -> String {
    let raw: String = slug::slugify(name)
        .chars()
        .filter(|c| c.is_ascii_alphabetic() || *c == '-')
        .collect();
    collapse_and_trim_slug(&raw, 49)
}

pub(crate) fn collapse_and_trim_slug(raw: &str, max_len: usize) -> String {
    let mut result = String::with_capacity(raw.len());
    let mut prev_hyphen = true;
    for c in raw.chars() {
        if c == '-' {
            if !prev_hyphen {
                result.push('-');
                prev_hyphen = true;
            }
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }
    result
        .trim_matches('-')
        .chars()
        .take(max_len)
        .collect::<String>()
        .trim_end_matches('-')
        .to_string()
}

#[derive(Clone)]
pub struct Organizations<A: Authorize> {
    auth: A,
    db: Pool<Postgres>,
}

impl<A: Authorize> Organizations<A> {
    pub fn new(auth: A, db: Pool<Postgres>) -> Self {
        Self { auth, db }
    }

    pub async fn list<P: Principal>(&mut self, principal: &P) -> Result<Vec<Organization>, Error> {
        self.auth
            .lookup::<Organization>()
            .on_behalf_of(principal)
            .with(Permission::Get)
            .against(&self.db)
            .await
    }

    pub async fn create_organization<P: Principal + Sync>(
        &mut self,
        connection: &Pool<Postgres>,
        _principal: &P,
        name: String,
        parent_slug: Option<String>,
    ) -> Result<Organization, Error> {
        tracing::info!(
            "received request to create organization with name '{}' and parent slug '{:?}'",
            &name,
            &parent_slug
        );

        let slug = generate_organization_slug(&name);
        check_slug_available(connection, &slug).await?;

        let organization = Organization {
            slug: slug.clone(),
            name,
            parent_slug: parent_slug.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
        .create(connection)
        .await?;

        if let Some(ref parent_slug) = parent_slug {
            let parent = Organization::find(&self.db, parent_slug.clone()).await?;

            self.auth
                .write_relationship(&Relationship::new(&parent, Relation::Parent, &organization))
                .await?;
        }

        let default_project_slug = generate_project_slug(&slug, DEFAULT_PROJECT_NAME);
        let project = Project::factory()
            .slug(default_project_slug)
            .name(DEFAULT_PROJECT_NAME.to_owned())
            .organization_slug(slug)
            .create(&self.db)
            .await?;

        self.auth
            .write_relationship(&Relationship::new(
                &organization,
                Relation::Parent,
                &project,
            ))
            .await?;

        Ok(organization)
    }

    pub async fn add_service_account(
        &mut self,
        organization: &Organization,
        service_account: &ServiceAccount,
    ) -> Result<(), Error> {
        // fabrique raw query: ON CONFLICT on non-PK unique constraint
        sqlx::query!("INSERT INTO organization_service_account(organization_slug, service_account_id) VALUES ($1::citext, $2) ON CONFLICT (service_account_id, organization_slug) DO NOTHING", organization.id(), service_account.id()).execute(&self.db).await?;

        self.auth
            .write_relationship(&Relationship::new(
                service_account,
                Relation::Member,
                organization,
            ))
            .await?;

        Ok(())
    }

    pub async fn add_user(
        &mut self,
        organization: &Organization,
        user: &User,
    ) -> Result<(), Error> {
        // fabrique raw query: ON CONFLICT on non-PK unique constraint
        sqlx::query!("INSERT INTO organization_user(organization_slug, user_id) VALUES ($1::citext, $2) ON CONFLICT (user_id, organization_slug) DO NOTHING", organization.id(), user.id()).execute(&self.db).await?;

        self.auth
            .write_relationship(&Relationship::new(user, Relation::Member, organization))
            .await?;

        Ok(())
    }

    pub async fn initialize_root_organization(
        &mut self,
        organization_name: String,
    ) -> Result<Organization, Error> {
        let maybe_organization: Option<Organization> = Organization::query()
            .select()
            .r#where(Organization::NAME, "=", organization_name.clone())
            .first(&self.db)
            .await?;

        let organization = match maybe_organization {
            Some(organization) => organization,
            None => {
                let slug = generate_organization_slug(&organization_name);
                check_slug_available(&self.db, &slug).await?;

                Organization {
                    slug: slug.clone(),
                    name: organization_name,
                    parent_slug: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }
                .create(&self.db)
                .await?
            }
        };

        // fabrique raw query: ON CONFLICT on non-PK unique constraint + conditional INSERT
        let default_project_slug = generate_project_slug(&organization.slug, DEFAULT_PROJECT_NAME);
        sqlx::query!(
            r#"
            INSERT INTO projects (slug, name, organization_slug)
            SELECT $1::citext, 'unattributed', $2::citext
            WHERE NOT EXISTS (
                SELECT 1 FROM projects
                WHERE name = 'unattributed' AND organization_slug = $2::citext
            )
            "#,
            &default_project_slug,
            &organization.slug
        )
        .execute(&self.db)
        .await?;

        Ok(organization)
    }
}

async fn check_slug_available<'e, E>(executor: E, slug: &str) -> Result<(), Error>
where
    E: sqlx::Executor<'e, Database = Postgres>,
{
    let existing: Option<Organization> = Organization::query()
        .select()
        .r#where(Organization::SLUG, "=", slug.to_owned())
        .first(executor)
        .await?;

    if existing.is_some() {
        return Err(Error::SlugAlreadyExists(slug.to_owned()));
    }

    Ok(())
}
