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
    Instanciate(Instanciate),
    InstanciateStatus(InstanciateStatus),
    InputData(InputData),
    CloseInput(CloseInput),
    OutputData(OutputData),
    CloseOutput(CloseOutput),
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
pub struct Instanciate {
    pub id: u64,
    pub parameters: HashMap<String, RawValue>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InstanciateStatus {
    Ok { id: u64 },
    Failure { id: u64, message: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputData {
    pub id: u64,
    pub name: String,
    pub data: Vec<RawValue>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CloseInput {
    pub id: u64,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutputData {
    pub id: u64,
    pub name: String,
    pub data: Vec<RawValue>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CloseOutput {
    pub id: u64,
    pub name: String,
}
