pub trait Authorize {
    type Id: ToString;
    fn any_resource() -> (&'static str, &'static str);
    fn resource(&self) -> (&'static str, &Self::Id);
    fn resource_name() -> &'static str;
}
