use super::{Element, PlatformAvailability};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Jeu {
        file: Element,
    },
    Compiled {
        crate_name: String,
        platforms: Vec<PlatformAvailability>,
    },
}
