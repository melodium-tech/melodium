use crate::Attributes;
use melodium_engine::design::{Connection, IO};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub enum IoDesign {
    Sequence(),
    Treatment(String),
}

impl From<&IO> for IoDesign {
    fn from(value: &IO) -> Self {
        match value {
            IO::Sequence() => IoDesign::Sequence(),
            IO::Treatment(name) => IoDesign::Treatment(name.clone()),
        }
    }
}

impl Into<IO> for &IoDesign {
    fn into(self) -> IO {
        match self {
            IoDesign::Sequence() => IO::Sequence(),
            IoDesign::Treatment(name) => IO::Treatment(name.clone()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ConnectionDesign {
    pub output_treatment: IoDesign,
    pub output_name: String,

    pub input_treatment: IoDesign,
    pub input_name: String,

    pub attributes: Attributes,
}

impl From<&Connection> for ConnectionDesign {
    fn from(value: &Connection) -> Self {
        Self {
            output_treatment: (&value.output_treatment).into(),
            output_name: value.output_name.clone(),
            input_treatment: (&value.input_treatment).into(),
            input_name: value.input_name.clone(),
            attributes: (&value.attributes).into(),
        }
    }
}

impl Into<Connection> for &ConnectionDesign {
    fn into(self) -> Connection {
        Connection {
            output_treatment: (&self.output_treatment).into(),
            output_name: self.output_name.clone(),
            input_treatment: (&self.input_treatment).into(),
            input_name: self.input_name.clone(),
            attributes: (&self.attributes).into(),
        }
    }
}
