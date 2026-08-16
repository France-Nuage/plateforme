use crate::Error;
use crate::authorization::{Authorize, Principal};
use crate::resourcemanager::Organization;
use fabrique::{Factory, Model, Persist, Query};
use frn_derive::Resource;
use sqlx::{Pool, Postgres, types::chrono};
use uuid::Uuid;

#[derive(Debug, Default, Factory, Model, Resource)]
pub struct User {
    /// Unique identifier for the user
    #[fabrique(primary_key)]
    pub id: Uuid,

    /// The user email
    pub email: String,

    /// OIDC subject (`sub`) pinned to this row on first login.
    ///
    /// `email` is a mutable, reassignable handle; `sub` is the immutable subject
    /// the identity provider issues. Pinning `sub` here lets the resolver reject a
    /// token whose verified email matches this row but whose subject differs —
    /// e.g. a departed platform-admin's address later recycled to someone else.
    /// `None` on rows created before subject-pinning (or provisioned by
    /// invitation); the value is recorded the first time such a user authenticates.
    pub sub: Option<String>,

    /// Platform-administration flag.
    ///
    /// Indicates whether this user holds platform-wide administrative
    /// privileges. This field is part of the transitional authorization model
    /// and will be replaced by fine-grained SpiceDB permissions in the future.
    ///
    /// **Note**: despite its historical name, this flag grants platform-level
    /// access (for example the Kubernetes cluster registry), not
    /// organization-scoped rights. See
    /// [`crate::authorization::Principal::is_platform_admin`].
    pub is_admin: bool,

    /// Creation time of the instance
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Time of the instance last update
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub async fn find_one_by_email(
        pool: &sqlx::Pool<Postgres>,
        email: &str,
    ) -> Result<Option<User>, fabrique::Error> {
        User::query()
            .select()
            .r#where(User::EMAIL, "=", email.to_owned())
            .first(pool)
            .await
    }

    /// Resolves the user for a **verified** `email`, pinning the OIDC subject
    /// `sub` to the row.
    ///
    /// `email` is a mutable, reassignable handle, so keying identity on it alone
    /// lets a recycled address (e.g. a departed platform-admin's, later reassigned
    /// to someone else) inherit the original row — including its `is_admin` flag.
    /// The immutable subject is therefore pinned on first login and compared on
    /// every later one:
    /// - the row has no pinned subject yet (created before pinning, or by
    ///   invitation) → record `sub`;
    /// - the pinned subject equals `sub` → the same principal, allow;
    /// - the pinned subject differs → the email was reassigned to another subject
    ///   → fail closed with [`crate::Error::SubjectMismatch`].
    ///
    /// An empty `sub` is refused for the same reason (no stable subject to key on).
    /// Callers pass the subject from an already-validated credential (the sealed
    /// session cookie or a signature-verified bearer token).
    pub async fn find_or_create_one_by_email(
        pool: &Pool<Postgres>,
        email: &str,
        sub: &str,
    ) -> Result<User, crate::Error> {
        if sub.is_empty() {
            return Err(crate::Error::SubjectMismatch);
        }

        match User::find_one_by_email(pool, email).await? {
            Some(user) => {
                // Clone the pinned value so the match scrutinee borrows the local,
                // leaving `user` free to move into the arms.
                let pinned = user.sub.clone();
                match pinned.as_deref() {
                    // Already pinned to this subject: the same principal.
                    Some(existing) if existing == sub => Ok(user),
                    // Pinned to a DIFFERENT, non-empty subject: the email was
                    // reassigned to another subject → fail closed.
                    Some(existing) if !existing.is_empty() => {
                        tracing::warn!(
                            email,
                            "auth rejected: verified email resolves to a row pinned to a different subject"
                        );
                        Err(crate::Error::SubjectMismatch)
                    }
                    // Not yet pinned — the row predates subject-pinning (`NULL`) or
                    // is an empty placeholder from invitation: bind this subject now.
                    _ => {
                        User::update()
                            .set(User::SUB, Some(sub.to_owned()))
                            .r#where(User::ID, "=", user.id)
                            .execute(pool)
                            .await?;
                        Ok(User {
                            sub: Some(sub.to_owned()),
                            ..user
                        })
                    }
                }
            }
            None => User {
                id: Uuid::new_v4(),
                sub: Some(sub.to_owned()),
                is_admin: false,
                email: email.to_owned(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }
            .create(pool)
            .await
            .map_err(Into::into),
        }
    }
}

impl Principal for User {
    /// Returns all organizations this user has access to
    async fn organizations(
        &self,
        connection: &Pool<Postgres>,
    ) -> Result<Vec<Organization>, crate::Error> {
        Organization::all(connection).await.map_err(Into::into)
    }

    fn is_platform_admin(&self) -> bool {
        self.is_admin
    }
}

#[derive(Clone)]
pub struct Users<Auth: Authorize> {
    _auth: Auth,
    db: Pool<Postgres>,
}

impl<Auth: Authorize> Users<Auth> {
    pub fn new(auth: Auth, db: Pool<Postgres>) -> Self {
        Self { _auth: auth, db }
    }

    pub async fn find_or_create<P: Principal>(
        &self,
        principal: &P,
        email: String,
    ) -> Result<User, Error> {
        match User::find_one_by_email(&self.db, &email).await? {
            Some(user) => Ok(user),
            None => self.create(principal, email).await,
        }
    }

    pub async fn create<P: Principal>(&self, _principal: &P, email: String) -> Result<User, Error> {
        User::factory()
            .id(Uuid::new_v4())
            .email(email)
            .is_admin(false)
            .create(&self.db)
            .await
            .map_err(Into::into)
    }
}
