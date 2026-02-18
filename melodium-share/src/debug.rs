use std::collections::BTreeMap;
use super::{Identifier, Value};
use chrono::{DateTime, Utc};
use melodium_engine::debug::{Event as EngineEvent, EventKind as EngineEventKind};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Event {
    pub timestamp: DateTime<Utc>,
    pub kind: EventKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub enum EventKind {
    ModelBuilt {
        model: Identifier,
        parameters: BTreeMap<String, Value>,
        host_treatment: HostTreatment,
        host_build: Option<u64>,
        label: String,
    },
    ModelAdded {
        model_id: u64,
        model: Identifier,
    },
    ContinuousModelsStarted,
    ContinuousModelsFinished,
    TreatmentBuilt {
        treatment: Identifier,
        environment: ContextualEnvironment,
        host_treatment: HostTreatment,
        host_build: Option<u64>,
        build_id: u64,
        label: String,
    },
    TrackAdded {
        info: InfoTrack,
        creation: TrackCreation,
    },
    TrackFinished {
        info: InfoTrack,
    },
    TreatmentStarted {
        treatment: Identifier,
        host_treatment: HostTreatment,
        host_build: Option<u64>,
        build_id: u64,
        track_id: u64,
        label: String,
    },
    TreatmentFinished {
        treatment: Identifier,
        host_treatment: HostTreatment,
        host_build: Option<u64>,
        build_id: u64,
        track_id: u64,
        label: String,
    },
    DataSent {
        output: TransmissionDetails,
        track_id: u64,
        data: DataContent,
    },
    DataTransmitted {
        output: TransmissionDetails,
        input: TransmissionDetails,
        track_id: u64,
        data: DataContent,
    },
    DataReceived {
        input: TransmissionDetails,
        track_id: u64,
        data: DataContent,
    },
    OutputClosed {
        output: TransmissionDetails,
        track_id: u64,
    },
    InputClosed {
        input: TransmissionDetails,
        track_id: u64,
    },
    Distant {
        run_id: Uuid,
        text: String,
        //event: Box<Event>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub enum HostTreatment {
    Treatment(Identifier),
    Direct,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub struct InfoTrack {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub ancestry_level: u64,
    pub results: Option<TrackResult>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub enum TrackResult {
    AllOk,
    NotAllOk,
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]

pub enum TrackCreation {
    Direct,
    Source {
        source: String,
        model_id: u64,
        parameters: BTreeMap<String, Value>,
        contexts: Vec<Identifier>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub enum DataContent {
    Undetailed,
    Count { count: u64 },
    Values { values: Vec<Value> },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ContextualEnvironment {
    track_id: u64,
    contexts: BTreeMap<String, Identifier>,
    variables: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]

pub struct TransmissionDetails {
    pub treatment: Identifier,
    pub host_treatment: HostTreatment,
    pub host_build: Option<u64>,
    pub build_id: u64,
    pub label: String,
    pub name: String,
}