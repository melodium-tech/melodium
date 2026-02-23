use crate::{Collection, Identifier, RawValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ProgramDump {
    pub collection: Collection,
    pub entrypoint: Identifier,
    pub parameters: BTreeMap<String, RawValue>,
}
