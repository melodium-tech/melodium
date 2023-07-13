use super::{Author, Tag, Type};
use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
//use iso639_1::Iso639_1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub authors: Vec<Author>,
    pub publication: DateTime<Utc>,
    // TODO when https://github.com/AlbanMinassian/iso639/tree/master/iso639-1 is released as 0.3.1 with serde
    // pub description: HashMap<Iso639_1, String>,
    pub description: HashMap<String, String>,
    pub version: Version,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub r#type: Type,
    pub tags: Vec<Tag>,
}

impl From<&crate::technical::Package> for Package {
    fn from(value: &crate::technical::Package) -> Self {
        Self {
            name: value.name.clone(),
            authors: Vec::new(),
            publication: DateTime::parse_from_rfc3339("2023-05-11T10:03:00+02:00")
                .unwrap()
                .into(),
            description: HashMap::new(),
            version: value.version.clone(),
            license: String::new(),
            homepage: None,
            repository: None,
            r#type: match value.r#type {
                crate::technical::Type::Jeu { .. } => Type::Jeu,
                crate::technical::Type::Compiled { .. } => Type::Compiled,
            },
            tags: Vec::new(),
        }
    }
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.version == other.version
    }
}
