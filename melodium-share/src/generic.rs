use super::DataTrait;
use melodium_common::descriptor::Generic as CommonGeneric;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Generic {
    pub name: String,
    pub traits: Vec<DataTrait>,
}

impl From<&CommonGeneric> for Generic {
    fn from(value: &CommonGeneric) -> Self {
        Self {
            name: value.name.clone(),
            traits: value.traits.iter().map(|tr| tr.into()).collect(),
        }
    }
}

impl Into<CommonGeneric> for &Generic {
    fn into(self) -> CommonGeneric {
        CommonGeneric {
            name: self.name.clone(),
            traits: self.traits.iter().map(|tr| tr.into()).collect(),
        }
    }
}

impl Into<CommonGeneric> for Generic {
    fn into(self) -> CommonGeneric {
        CommonGeneric {
            name: self.name,
            traits: self.traits.into_iter().map(|tr| tr.into()).collect(),
        }
    }
}
