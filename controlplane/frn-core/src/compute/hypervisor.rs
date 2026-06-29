use crate::Error;
use crate::authorization::{Authorize, Permission, Principal, Relation, Relationship, Resource};
use crate::compute::{Zone, ZoneFactory, ZoneIdColumn};
use crate::resourcemanager::Organization;
use fabrique::{Delete, Factory, Model, Query};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Default, Factory, Model, Resource)]
pub struct Hypervisor {
    /// The hypervisor id
    #[fabrique(primary_key)]
    pub id: Uuid,

    #[fabrique(belongs_to = Zone)]
    pub zone_id: Uuid,

    /// The slug of the organization the hypervisor belongs to
    pub organization_slug: String,

    /// The hypervisor url
    pub url: String,

    /// The hypervisor authentication token
    pub authorization_token: String,

    /// The hypervisor storage name
    pub storage_name: String,
}

pub struct HypervisorCreateRequest {
    pub authorization_token: String,
    pub storage_name: String,
    pub organization_slug: String,
    pub url: String,
    pub zone_id: Uuid,
}

#[derive(Clone)]
pub struct Hypervisors<Auth: Authorize> {
    auth: Auth,
    db: Pool<Postgres>,
}

impl<Auth: Authorize> Hypervisors<Auth> {
    /// Creates a new hypervisors service.
    pub fn new(auth: Auth, db: Pool<Postgres>) -> Self {
        Self { auth, db }
    }

    /// Lists all hypervisors accessible to the principal.
    pub async fn list<P: Principal>(&mut self, _principal: &P) -> Result<Vec<Hypervisor>, Error> {
        // self.auth
        //     .can(principal)
        //     .perform(Permission::List)
        //     .over(&Hypervisor::any())
        //     .check()
        //     .await?;

        Hypervisor::all(&self.db).await.map_err(Into::into)
    }

    /// Creates a new hypervisor.
    pub async fn create<P: Principal>(
        &mut self,
        _principal: &P,
        request: HypervisorCreateRequest,
    ) -> Result<Hypervisor, Error> {
        // self.auth
        //     .can(principal)
        //     .perform(Permission::Create)
        //     .over(&Hypervisor::any())
        //     .check()
        //     .await?;

        let hypervisor = Hypervisor::factory()
            .storage_name(request.storage_name)
            .url(request.url)
            .authorization_token(request.authorization_token)
            .zone_id(request.zone_id)
            .organization_slug(request.organization_slug.clone())
            .create(&self.db)
            .await?;

        self.auth
            .write_relationship(&Relationship::new(
                &Organization::some(request.organization_slug.clone()),
                Relation::Parent,
                &hypervisor,
            ))
            .await?;

        Ok(hypervisor)
    }

    pub async fn read<P: Principal>(
        &mut self,
        principal: &P,
        id: Uuid,
    ) -> Result<Hypervisor, Error> {
        self.auth
            .can::<P>(principal)
            .perform(Permission::Get)
            .over::<Hypervisor>(&id)
            .await?;

        Hypervisor::find(&self.db, id).await.map_err(Into::into)
    }

    /// Deletes a hypervisor.
    pub async fn delete<P: Principal>(&mut self, principal: &P, id: Uuid) -> Result<(), Error> {
        self.auth
            .can::<P>(principal)
            .perform(Permission::Delete)
            .over::<Hypervisor>(&id)
            .await?;

        Hypervisor::destroy(&self.db, id).await.map_err(Into::into)
    }
}
