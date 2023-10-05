use cargo_author::Author as AuthorParsing;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Author {
    pub name: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
}

impl Author {
    pub fn new(s: &str) -> Self {
        let author = AuthorParsing::new(s);
        Self {
            name: author.name,
            email: author.email,
            url: author.url,
        }
    }
}

impl Serialize for Author {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(
            &vec![
                self.name.clone(),
                self.email.clone().map(|e| format!("<{e}>")),
                self.url.clone().map(|u| format!("({u})")),
            ]
            .into_iter()
            .filter_map(|o| o)
            .collect::<Vec<_>>()
            .join(" "),
        )
    }
}

impl<'de> Deserialize<'de> for Author {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).map(|author: String| Author::new(&author))
    }
}
