use uuid::Uuid;

pub trait Authorize {
    type Id: ToString;
    fn any_resource() -> (&'static str, &'static str);
    fn resource(&self) -> (&'static str, &Self::Id);
    fn resource_name() -> &'static str;
}

struct Anvil {
    id: Uuid,
}

impl Authorize for Anvil {
    type Id = Uuid;

    fn any_resource() -> (&'static str, &'static str) {
        ("anvil", "*")
    }

    fn resource(&self) -> (&'static str, &Self::Id) {
        ("anvil", &self.id)
    }

    fn resource_name() -> &'static str {
        "anvil"
    }
}
