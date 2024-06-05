use crate::Value;
use melodium_engine::design::Model;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelDesign {
    pub parameters: BTreeMap<String, Value>,
}

impl From<&Model> for ModelDesign {
    fn from(value: &Model) -> Self {
        Self {
            parameters: value
                .parameters
                .iter()
                .map(|(name, param)| (name.clone(), (&param.value).into()))
                .collect(),
        }
    }
}
