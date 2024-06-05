use crate::{Attributes, DescribedType, Value};
use melodium_engine::design::TreatmentInstanciation;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TreatmentInstanciationDesign {
    pub generics: BTreeMap<String, DescribedType>,
    pub models: BTreeMap<String, String>,
    pub parameters: BTreeMap<String, Value>,
    pub attributes: Attributes,
}

impl From<&TreatmentInstanciation> for TreatmentInstanciationDesign {
    fn from(value: &TreatmentInstanciation) -> Self {
        Self {
            generics: value
                .generics
                .iter()
                .map(|(name, dt)| (name.clone(), dt.into()))
                .collect(),
            models: value
                .models
                .iter()
                .map(|(param_name, local_name)| (param_name.clone(), local_name.clone()))
                .collect(),
            parameters: value
                .parameters
                .iter()
                .map(|(name, param)| (name.clone(), (&param.value).into()))
                .collect(),
            attributes: (&value.attributes).into(),
        }
    }
}
