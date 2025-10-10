use strum_macros::{Display, EnumString};

#[derive(Debug, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Permission {
    Create,
    Delete,
    Get,
    List,
    Start,
    Stop,
}
