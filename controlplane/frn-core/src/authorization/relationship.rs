use crate::authorization::Resource;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

/// Type de relation entre un sujet et un objet.
#[derive(Clone, Debug, Default, Display, EnumString, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    Member,
    Parent,
    #[default]
    Unspecified,
}

impl From<Relation> for String {
    fn from(value: Relation) -> Self {
        value.to_string()
    }
}

impl From<String> for Relation {
    fn from(value: String) -> Self {
        Relation::from_str(&value).expect("could not parse relation")
    }
}

/// Relation d'autorisation entre un sujet et un objet.
///
/// Représente une relation SpiceDB de la forme `object_type:object_id#relation@subject_type:subject_id`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Relationship {
    pub relation: Relation,
    pub object_id: String,
    pub object_type: String,
    pub subject_id: String,
    pub subject_type: String,
}

impl Relationship {
    pub fn new<Subject: Resource, Object: Resource>(
        subject: &Subject,
        relation: Relation,
        object: &Object,
    ) -> Self {
        Self {
            object_id: object.id().to_string(),
            object_type: object.name().to_string(),
            relation,
            subject_id: subject.id().to_string(),
            subject_type: subject.name().to_string(),
        }
    }
}

impl Display for Relationship {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}#{}@{}:{}",
            self.object_type, self.object_id, self.relation, self.subject_type, self.subject_id
        )
    }
}
