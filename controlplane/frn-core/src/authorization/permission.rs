use strum_macros::{Display, EnumString};

#[derive(Debug, Display, EnumString)]
pub enum Permission {
    #[strum(serialize = "create")]
    Create,

    #[strum(serialize = "get")]
    Get,
}
