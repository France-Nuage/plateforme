use std::str::FromStr;

use strum_macros::{Display, EnumString};

#[derive(Clone, Debug, Default, Display, EnumString)]
pub enum Relation {
    #[strum(serialize = "project")]
    BelongsToProject,

    #[default]
    None,
}

impl From<Relation> for String {
    fn from(value: Relation) -> Self {
        value.to_string()
    }
}

impl From<String> for Relation {
    fn from(value: String) -> Self {
        Relation::from_str(&value)
            .unwrap_or_else(|_| panic!("invalid permission string: {}", value))
    }
}
