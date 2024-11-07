use crate::{SharingResult, Value};
use melodium_common::descriptor::{Collection, Identified};
use melodium_engine::{
    descriptor::Model as DesignedModel,
    design::{Model, Parameter as DesignedParameter},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelDesign {
    pub parameters: BTreeMap<String, Value>,
}

impl ModelDesign {
    pub(crate) fn make_design(
        &self,
        collection: &Collection,
        descriptor: &Arc<DesignedModel>,
    ) -> SharingResult<Model> {
        let mut result = SharingResult::new_success(());
        let mut parameters = HashMap::with_capacity(self.parameters.len());

        for (name, value) in &self.parameters {
            if let Some(value) =
                result.merge_degrade_failure(value.to_value(collection, descriptor.identifier()))
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
            SharingResult::new_success(Model {
                descriptor: Arc::downgrade(descriptor),
                parameters,
            })
        })
    }
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
