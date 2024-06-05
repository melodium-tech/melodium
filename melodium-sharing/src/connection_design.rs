use crate::Attributes;
use melodium_engine::design::{Connection, IO};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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
