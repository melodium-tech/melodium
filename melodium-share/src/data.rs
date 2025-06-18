use super::{Attributes, DataTrait, Identifier};
use melodium_common::descriptor::Data as CommonData;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Data {
    pub identifier: Identifier,
    pub documentation: String,
    pub implements: Vec<DataTrait>,
    pub attributes: Attributes,
}

impl From<&dyn CommonData> for Data {
    fn from(value: &dyn CommonData) -> Self {
        Self {
            identifier: value.identifier().into(),
            documentation: value.documentation().to_string(),
            implements: value.implements().iter().map(|dt| dt.into()).collect(),
            attributes: value.attributes().into(),
        }
    }
}
