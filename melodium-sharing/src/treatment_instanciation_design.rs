use crate::{Attributes, DescribedType, Identifier, SharingError, SharingResult, Value};
use melodium_common::descriptor::{Collection, Identified, Identifier as CommonIdentifier};
use melodium_engine::{
    descriptor::Treatment as DesignedTreatment,
    design::{Parameter as DesignedParameter, TreatmentInstanciation},
    LogicError,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TreatmentInstanciationDesign {
    pub identifier: Identifier,
    pub generics: BTreeMap<String, DescribedType>,
    pub models: BTreeMap<String, String>,
    pub parameters: BTreeMap<String, Value>,
    pub attributes: Attributes,
}

impl TreatmentInstanciationDesign {
    pub(crate) fn make_design(
        &self,
        collection: &Collection,
        scope: &Arc<DesignedTreatment>,
        name: &str,
    ) -> SharingResult<TreatmentInstanciation> {
        let mut result = SharingResult::new_success(());

        let identifier: CommonIdentifier = if let Ok(identifier) = (&self.identifier).try_into() {
            identifier
        } else {
            return SharingResult::new_failure(SharingError::invalid_identifier(
                17,
                self.identifier.clone(),
            ));
        };

        let treatment = if let Some(melodium_common::descriptor::Entry::Treatment(treatment)) =
            collection.get(&(&identifier).into())
        {
            treatment
        } else {
            return SharingResult::new_failure(
                LogicError::unexisting_treatment(
                    241,
                    scope.identifier().clone(),
                    identifier.into(),
                    None,
                )
                .into(),
            );
        };

        let mut generics = HashMap::with_capacity(self.generics.len());

        for (name, described_type) in &self.generics {
            if let Some(described_type) = result.merge_degrade_failure(
                described_type.to_described_type(collection, scope.identifier()),
            ) {
                generics.insert(name.clone(), described_type);
            }
        }

        let mut parameters = HashMap::with_capacity(self.parameters.len());

        for (name, value) in &self.parameters {
            if let Some(value) =
                result.merge_degrade_failure(value.to_value(collection, scope.identifier()))
            {
                parameters.insert(
                    name.clone(),
                    DesignedParameter {
                        name: name.clone(),
                        value,
                    },
                );
            }
        }

        result.and_then(|_| {
            SharingResult::new_success(TreatmentInstanciation {
                descriptor: Arc::downgrade(treatment),
                name: name.to_string(),
                attributes: (&self.attributes).into(),
                models: self
                    .models
                    .iter()
                    .map(|(param_name, local_name)| (param_name.clone(), local_name.clone()))
                    .collect(),
                generics,
                parameters,
            })
        })
    }
}

impl From<&TreatmentInstanciation> for TreatmentInstanciationDesign {
    fn from(value: &TreatmentInstanciation) -> Self {
        Self {
            identifier: value.descriptor.upgrade().unwrap().identifier().into(),
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
