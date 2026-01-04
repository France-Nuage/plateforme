use fabrique::{Factory, Persistable};
use uuid::Uuid;

#[derive(Default, Factory, Persistable)]
pub struct Operation {
    #[fabrique(primary_key)]
    pub id: Uuid,
}
