pub trait Resource {
    type Id: ToString + Sync;
    const NAME: &'static str;

    fn id(&self) -> &Self::Id;
}
