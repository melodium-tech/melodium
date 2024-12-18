use super::{Attributes, DataType, Identifier};
use melodium_common::descriptor::Context as CommonContext;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Context {
    pub identifier: Identifier,
    pub documentation: String,
    pub values: BTreeMap<String, DataType>,
    pub attributes: Attributes,
}

impl From<&dyn CommonContext> for Context {
    fn from(value: &dyn CommonContext) -> Self {
        Self {
            identifier: Identifier::from(value.identifier()),
            documentation: value.documentation().to_string(),
            values: value
                .values()
                .iter()
                .map(|(name, datatype)| (name.clone(), DataType::from(datatype)))
                .collect(),
            attributes: value.attributes().into(),
        }
    }
}
