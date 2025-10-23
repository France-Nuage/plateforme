use strum_macros::{Display, EnumString};

/// Actions that can be performed on resources
#[derive(Debug, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Permission {
    Create,
    Delete,
    Get,
    List,
    InviteMember,
    Start,
    Stop,
}
