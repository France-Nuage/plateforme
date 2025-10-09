use std::str::FromStr;

use strum_macros::{Display, EnumString, IntoStaticStr};

#[derive(Debug, Default, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Status {
    /// Instance is active and operational.
    Running,

    /// Instance is inactive.
    Stopped,

    /// Instance status is unknown.
    #[default]
    Unknown,
}

impl From<String> for Status {
    fn from(value: String) -> Self {
        Status::from_str(&value).expect("could not parse value to status")
    }
}

impl From<Status> for String {
    fn from(value: Status) -> Self {
        value.to_string()
    }
}
