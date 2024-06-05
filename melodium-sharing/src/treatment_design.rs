use crate::{ConnectionDesign, ModelInstanciationDesign, TreatmentInstanciationDesign};
use melodium_engine::design::Treatment;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreatmentDesign {
    pub model_instanciations: BTreeMap<String, ModelInstanciationDesign>,
    pub treatments: BTreeMap<String, TreatmentInstanciationDesign>,
    pub connections: Vec<ConnectionDesign>,
}

impl From<&Treatment> for TreatmentDesign {
    fn from(value: &Treatment) -> Self {
        Self {
            model_instanciations: value
                .model_instanciations
                .iter()
                .map(|(name, model)| (name.clone(), model.into()))
                .collect(),
            treatments: value
                .treatments
                .iter()
                .map(|(name, treatment)| (name.clone(), treatment.into()))
                .collect(),
            connections: value.connections.iter().map(|conn| conn.into()).collect(),
        }
    }
}
