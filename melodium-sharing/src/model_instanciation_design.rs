use crate::{Attributes, Value};
use melodium_engine::design::ModelInstanciation;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelInstanciationDesign {
    pub parameters: BTreeMap<String, Value>,
    pub attributes: Attributes,
}

impl From<&ModelInstanciation> for ModelInstanciationDesign {
    fn from(value: &ModelInstanciation) -> Self {
        Self {
            parameters: value
                .parameters
                .iter()
                .map(|(name, param)| (name.clone(), (&param.value).into()))
                .collect(),
            attributes: (&value.attributes).into(),
        }
    }
}
