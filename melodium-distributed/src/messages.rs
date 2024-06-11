use melodium_common::descriptor::Version;
use melodium_sharing::{Collection, Identifier, RawValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Message {
    AskDistribution(AskDistribution),
    ConfirmDistribution(ConfirmDistribution),
    LoadAndLaunch(LoadAndLaunch),
    LaunchStatus(LaunchStatus),
    InputData(InputData),
    OutputData(OutputData),
    Ended,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AskDistribution {
    pub melodium_version: Version,
    pub distribution_version: Version,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfirmDistribution {
    pub accept: bool,
    pub melodium_version: Version,
    pub distribution_version: Version,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoadAndLaunch {
    pub collection: Collection,
    pub entrypoint: Identifier,
    pub parameters: HashMap<String, RawValue>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LaunchStatus {
    Ok,
    Failure(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputData {
    pub name: String,
    pub data: Vec<RawValue>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutputData {
    pub name: String,
    pub data: Vec<RawValue>,
}
