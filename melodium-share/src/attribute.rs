use melodium_common::descriptor::Attributes as CommonAttributes;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type Attribute = String;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Attributes(pub BTreeMap<String, Attribute>);

impl Attributes {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
}

impl From<&CommonAttributes> for Attributes {
    fn from(value: &CommonAttributes) -> Self {
        Self(value.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
    }
}

impl From<CommonAttributes> for Attributes {
    fn from(value: CommonAttributes) -> Self {
        Self(value.into_iter().collect())
    }
}

impl Into<CommonAttributes> for Attributes {
    fn into(self) -> CommonAttributes {
        self.0.into_iter().collect()
    }
}

impl Into<CommonAttributes> for &Attributes {
    fn into(self) -> CommonAttributes {
        self.0.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
}
