use std::str::FromStr;
use strum_macros::{Display, EnumString};

/// Actions that can be performed on resources
#[derive(Debug, Default, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Permission {
    // Instance operations
    Clone,
    CreateInstance,
    Delete,
    Get,
    List,
    Start,
    Stop,

    // Organization operations
    InviteMember,

    // VPC operations
    CreateVPC,
    UpdateVPC,

    // VNet operations
    CreateVNet,
    UpdateVNet,

    // Security Group operations
    CreateSecurityGroup,
    UpdateSecurityGroup,

    // IPAM operations
    AllocateIP,
    ReleaseIP,
    ReserveIP,

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
