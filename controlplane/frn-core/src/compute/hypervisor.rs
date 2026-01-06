use crate::Error;
use crate::authorization::{Authorize, Permission, Principal, Relation, Relationship, Resource};
use crate::compute::{Zone, ZoneFactory, ZoneIdColumn};
use crate::longrunning::Operation;
use crate::resourcemanager::{Organization, OrganizationFactory, OrganizationIdColumn};
use fabrique::{Factory, Model, Query};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Default, Factory, Model, Resource)]
pub struct Hypervisor {
    /// The hypervisor id
    #[fabrique(primary_key)]
    pub id: Uuid,

    #[fabrique(belongs_to = Zone)]
    pub zone_id: Uuid,

    /// The id of the organization the hypervisor belongs to
    #[fabrique(belongs_to = Organization)]
    pub organization_id: Uuid,

    /// The hypervisor url
    pub url: String,

    /// The hypervisor authentication token
    pub authorization_token: String,

    /// The hypervisor storage name
    pub storage_name: String,
}

impl Hypervisor {
    pub async fn find_one_by_id(
        pool: &Pool<Postgres>,
        id: Uuid,
    ) -> Result<Hypervisor, sqlx::Error> {
        sqlx::query_as!(Hypervisor, "SELECT id, zone_id, organization_id, url, authorization_token, storage_name FROM hypervisors WHERE id = $1", id).fetch_one(pool).await
    }
}

pub struct HypervisorCreateRequest {
    pub authorization_token: String,
    pub storage_name: String,
    pub organization_id: Uuid,
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
            .organization_id(request.organization_id)
            .create(&self.db)
            .await?;

        Operation::write_relationships(vec![Relationship::new(
            &Organization::some(request.organization_id),
            Relation::Parent,
            &hypervisor,
        )])
        .dispatch(&self.db)
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

        Hypervisor::find_one_by_id(&self.db, id)
            .await
            .map_err(Into::into)
    }

    /// Deletes a hypervisor.
    pub async fn delete<P: Principal>(&mut self, principal: &P, id: Uuid) -> Result<(), Error> {
        self.auth
            .can::<P>(principal)
            .perform(Permission::Delete)
            .over::<Hypervisor>(&id)
            .await?;

        sqlx::query!("DELETE FROM hypervisors WHERE id = $1", id)
            .execute(&self.db)
            .await
            .map(|_| ())
            .map_err(Into::into)
    }
}
