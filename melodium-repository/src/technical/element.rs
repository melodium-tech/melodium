use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Element {
    pub name: String,
    pub checksum: String,
}
