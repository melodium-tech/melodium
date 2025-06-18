use melodium_common::descriptor::Variability as CommonVariability;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub enum Variability {
    Const,
    Var,
}

impl Into<CommonVariability> for Variability {
    fn into(self) -> CommonVariability {
        match self {
            Variability::Const => CommonVariability::Const,
            Variability::Var => CommonVariability::Var,
        }
    }
}

impl Into<CommonVariability> for &Variability {
    fn into(self) -> CommonVariability {
        match self {
            Variability::Const => CommonVariability::Const,
            Variability::Var => CommonVariability::Var,
        }
    }
}

impl From<&CommonVariability> for Variability {
    fn from(value: &CommonVariability) -> Self {
        match value {
            CommonVariability::Const => Variability::Const,
            CommonVariability::Var => Variability::Var,
        }
    }
}
