/// Entities that can be subjects of authorization checks
pub trait Resource {
    /// Identifier type for this resource
    type Id: ToString + Sync;

    /// Resource type name used in authorization checks
    const RESOURCE_NAME: &'static str;

    /// Create a lightweight resource reference from an ID
    fn some(id: Self::Id) -> impl Resource<Id = Self::Id>;

    /// Get the resource's identifier
    fn id(&self) -> &Self::Id;

    /// Get the resource type name used in authorization checks
    fn name(&self) -> &'static str;
}
