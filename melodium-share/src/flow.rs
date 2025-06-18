use melodium_common::descriptor::Flow as CommonFlow;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub enum Flow {
    Block,
    Stream,
}

impl From<&CommonFlow> for Flow {
    fn from(value: &CommonFlow) -> Self {
        match value {
            CommonFlow::Block => Flow::Block,
            CommonFlow::Stream => Flow::Stream,
        }
    }
}

impl Into<CommonFlow> for Flow {
    fn into(self) -> CommonFlow {
        Into::into(&self)
    }
}

impl Into<CommonFlow> for &Flow {
    fn into(self) -> CommonFlow {
        match self {
            Flow::Block => CommonFlow::Block,
            Flow::Stream => CommonFlow::Stream,
        }
    }
}
