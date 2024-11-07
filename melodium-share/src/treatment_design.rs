use crate::{
    ConnectionDesign, ModelInstanciationDesign, SharingResult, TreatmentInstanciationDesign,
};
use melodium_common::descriptor::Collection;
use melodium_engine::{descriptor::Treatment as DesignedTreatment, design::Treatment};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TreatmentDesign {
    pub model_instanciations: BTreeMap<String, ModelInstanciationDesign>,
    pub treatments: BTreeMap<String, TreatmentInstanciationDesign>,
    pub connections: Vec<ConnectionDesign>,
}

impl TreatmentDesign {
    pub(crate) fn make_design(
        &self,
        collection: &Collection,
        descriptor: &Arc<DesignedTreatment>,
    ) -> SharingResult<Treatment> {
        let mut result = SharingResult::new_success(());

        let mut model_instanciations = HashMap::with_capacity(self.model_instanciations.len());

        for (name, model_instanciation) in &self.model_instanciations {
            if let Some(model_instanciation) = result.merge_degrade_failure(
                model_instanciation.make_design(collection, descriptor, name),
            ) {
                model_instanciations.insert(name.clone(), model_instanciation);
            }
        }

        let mut treatments = HashMap::with_capacity(self.treatments.len());

        for (name, treatment) in &self.treatments {
            if let Some(treatment) =
                result.merge_degrade_failure(treatment.make_design(collection, descriptor, name))
            {
                treatments.insert(name.clone(), treatment);
            }
        }

        result.and_then(|_| {
            SharingResult::new_success(Treatment {
                descriptor: Arc::downgrade(descriptor),
                model_instanciations,
                treatments,
                connections: self.connections.iter().map(|conn| conn.into()).collect(),
            })
        })
    }
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
