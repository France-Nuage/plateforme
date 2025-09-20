use sqlx::PgPool;

pub trait Persistable: Sized {
    type Connection: Clone;
    type Error;

    fn list(executor: &Self::Connection) -> impl Future<Output = Result<Vec<Self>, Self::Error>>;

    fn create(self, executor: &Self::Connection)
    -> impl Future<Output = Result<Self, Self::Error>>;

    fn update(self, executor: &Self::Connection)
    -> impl Future<Output = Result<Self, Self::Error>>;
}

/// deprecated
pub trait HasFactory {
    type Factory;

    /// Get a new factory instance for the model.
    fn factory(pool: PgPool) -> Self::Factory;
}

/// deprecated
pub trait Factory {
    type Model;

    /// Create a single model and persist it into the database.
    fn create(self) -> impl Future<Output = Result<Self::Model, sqlx::Error>> + Send;

    /// Add a new state transformation to the model definition.
    fn state(self, data: Self::Model) -> Self;
}
