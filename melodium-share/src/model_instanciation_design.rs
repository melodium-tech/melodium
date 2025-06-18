use crate::{Attributes, Identifier, SharingError, SharingResult, Value};
use melodium_common::descriptor::{Collection, Identified, Identifier as CommonIdentifier};
use melodium_engine::{
    descriptor::Treatment as DesignedTreatment,
    design::{ModelInstanciation, Parameter as DesignedParameter},
    LogicError,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ModelInstanciationDesign {
    pub identifier: Identifier,
    pub parameters: BTreeMap<String, Value>,
    pub attributes: Attributes,
}

impl ModelInstanciationDesign {
    pub(crate) fn make_design(
        &self,
        collection: &Collection,
        scope: &Arc<DesignedTreatment>,
        name: &str,
    ) -> SharingResult<ModelInstanciation> {
        let mut result = SharingResult::new_success(());

        let identifier: CommonIdentifier = if let Ok(identifier) = (&self.identifier).try_into() {
            identifier
        } else {
            return SharingResult::new_failure(SharingError::invalid_identifier(
                16,
                self.identifier.clone(),
            ));
        };

        let model = if let Some(melodium_common::descriptor::Entry::Model(model)) =
            collection.get(&(&identifier).into())
        {
            model
        } else {
            return SharingResult::new_failure(
                LogicError::unexisting_model(
                    240,
                    scope.identifier().clone(),
                    identifier.into(),
                    None,
                )
                .into(),
            );
        };

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
            SharingResult::new_success(ModelInstanciation {
                descriptor: Arc::downgrade(model),
                name: name.to_string(),
                attributes: (&self.attributes).into(),
                parameters,
            })
        })
    }
}

impl From<&ModelInstanciation> for ModelInstanciationDesign {
    fn from(value: &ModelInstanciation) -> Self {
        Self {
            identifier: value.descriptor.upgrade().unwrap().identifier().into(),
            parameters: value
                .parameters
                .iter()
                .map(|(name, param)| (name.clone(), (&param.value).into()))
                .collect(),
            attributes: (&value.attributes).into(),
        }
    }
}
