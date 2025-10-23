use std::str::FromStr;
use strum_macros::{Display, EnumString};

/// Actions that can be performed on resources
#[derive(Debug, Default, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Permission {
    Create,
    Delete,
    Get,
    List,
    InviteMember,
    Start,
    Stop,
    #[default]
    Unspecified,
}

impl From<Permission> for String {
    fn from(value: Permission) -> Self {
        value.to_string()
    }
}

impl From<String> for Permission {
    fn from(value: String) -> Self {
        Permission::from_str(&value).expect("could not parse permission")
    }
}
