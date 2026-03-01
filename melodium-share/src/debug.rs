use super::Identifier;
use crate::RawValue;
use chrono::{DateTime, Utc};
use melodium_engine::{
    build::{
        ContextualEnvironment as EngineContextualEnvironment, HostTreatment as EngineHostTreatment,
    },
    debug::{
        DataContent as EngineDataContent, Event as EngineEvent, EventKind as EngineEventKind,
        InfoTrack as EngineInfoTrack, TrackCreation as EngineTrackCreation,
        TrackResult as EngineTrackResult, TransmissionDetails as EngineTransmissionDetails,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Event {
    pub timestamp: DateTime<Utc>,
    pub kind: EventKind,
}

impl From<&EngineEvent> for Event {
    fn from(event: &EngineEvent) -> Self {
        Self {
            timestamp: event.timestamp,
            kind: (&event.kind).into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub enum EventKind {
    ModelBuilt {
        model: Identifier,
        parameters: BTreeMap<String, RawValue>,
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

impl From<&EngineEventKind> for EventKind {
    fn from(event_kind: &EngineEventKind) -> Self {
        match event_kind {
            EngineEventKind::ModelBuilt {
                model,
                parameters,
                host_treatment,
                host_build,
                label,
            } => EventKind::ModelBuilt {
                model: model.identifier().into(),
                parameters: parameters
                    .iter()
                    .map(|(name, value)| (name.clone(), value.into()))
                    .collect(),
                host_treatment: host_treatment.into(),
                host_build: host_build.map(|id| id as u64),
                label: label.clone(),
            },
            EngineEventKind::ModelAdded { model_id, model } => EventKind::ModelAdded {
                model_id: *model_id as u64,
                model: model.identifier().into(),
            },
            EngineEventKind::ContinuousModelsStarted => EventKind::ContinuousModelsStarted,
            EngineEventKind::ContinuousModelsFinished => EventKind::ContinuousModelsFinished,
            EngineEventKind::TreatmentBuilt {
                treatment,
                environment,
                host_treatment,
                host_build,
                build_id,
                label,
            } => EventKind::TreatmentBuilt {
                treatment: treatment.identifier().into(),
                environment: environment.into(),
                host_treatment: host_treatment.into(),
                host_build: host_build.map(|id| id as u64),
                build_id: *build_id as u64,
                label: label.clone(),
            },
            EngineEventKind::TrackAdded { info, creation } => EventKind::TrackAdded {
                info: info.into(),
                creation: creation.into(),
            },
            EngineEventKind::TrackFinished { info } => {
                EventKind::TrackFinished { info: info.into() }
            }
            EngineEventKind::TreatmentStarted {
                treatment,
                host_treatment,
                host_build,
                build_id,
                track_id,
                label,
            } => EventKind::TreatmentStarted {
                treatment: treatment.identifier().into(),
                host_treatment: host_treatment.into(),
                host_build: host_build.map(|id| id as u64),
                build_id: *build_id as u64,
                track_id: *track_id as u64,
                label: label.clone(),
            },
            EngineEventKind::TreatmentFinished {
                treatment,
                host_treatment,
                host_build,
                build_id,
                track_id,
                label,
            } => EventKind::TreatmentFinished {
                treatment: treatment.identifier().into(),
                host_treatment: host_treatment.into(),
                host_build: host_build.map(|id| id as u64),
                build_id: *build_id as u64,
                track_id: *track_id as u64,
                label: label.clone(),
            },
            EngineEventKind::DataSent {
                output,
                track_id,
                data,
            } => EventKind::DataSent {
                output: output.into(),
                track_id: *track_id as u64,
                data: data.into(),
            },
            EngineEventKind::DataTransmitted {
                output,
                input,
                track_id,
                data,
            } => EventKind::DataTransmitted {
                output: output.into(),
                input: input.into(),
                track_id: *track_id as u64,
                data: data.into(),
            },
            EngineEventKind::DataReceived {
                input,
                track_id,
                data,
            } => EventKind::DataReceived {
                input: input.into(),
                track_id: *track_id as u64,
                data: data.into(),
            },
            EngineEventKind::OutputClosed { output, track_id } => EventKind::OutputClosed {
                output: output.into(),
                track_id: *track_id as u64,
            },
            EngineEventKind::InputClosed { input, track_id } => EventKind::InputClosed {
                input: input.into(),
                track_id: *track_id as u64,
            },
            EngineEventKind::Distant { run_id, text } => EventKind::Distant {
                run_id: *run_id,
                text: text.clone(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub enum HostTreatment {
    Treatment(Identifier),
    Direct,
}

impl From<&EngineHostTreatment> for HostTreatment {
    fn from(host_treatment: &EngineHostTreatment) -> Self {
        match host_treatment {
            EngineHostTreatment::Treatment(treatment) => {
                HostTreatment::Treatment(treatment.identifier().into())
            }
            EngineHostTreatment::Direct => HostTreatment::Direct,
        }
    }
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

impl From<&EngineInfoTrack> for InfoTrack {
    fn from(info_track: &EngineInfoTrack) -> Self {
        Self {
            id: info_track.id as u64,
            parent_id: info_track.parent_id.map(|id| id as u64),
            ancestry_level: info_track.ancestry_level,
            results: info_track.results.as_ref().map(TrackResult::from),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub enum TrackResult {
    AllOk,
    NotAllOk,
}

impl From<&EngineTrackResult> for TrackResult {
    fn from(track_result: &EngineTrackResult) -> Self {
        match track_result {
            EngineTrackResult::AllOk(_) => TrackResult::AllOk,
            EngineTrackResult::NotAllOk(_, _) => TrackResult::NotAllOk,
        }
    }
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
        parameters: BTreeMap<String, RawValue>,
        contexts: Vec<Identifier>,
    },
}

impl From<&EngineTrackCreation> for TrackCreation {
    fn from(track_creation: &EngineTrackCreation) -> Self {
        match track_creation {
            EngineTrackCreation::Direct => TrackCreation::Direct,
            EngineTrackCreation::Source {
                source,
                model_id,
                parameters,
                contexts,
            } => TrackCreation::Source {
                source: source.clone(),
                model_id: *model_id as u64,
                parameters: parameters
                    .iter()
                    .map(|(name, value)| (name.clone(), value.into()))
                    .collect(),
                contexts: contexts
                    .iter()
                    .map(|context| context.descriptor().identifier().into())
                    .collect(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub enum DataContent {
    Undetailed,
    Count { count: u64 },
    Values { values: Vec<RawValue> },
}

impl From<&EngineDataContent> for DataContent {
    fn from(data_content: &EngineDataContent) -> Self {
        match data_content {
            EngineDataContent::Undetailed => DataContent::Undetailed,
            EngineDataContent::Count { count } => DataContent::Count {
                count: *count as u64,
            },
            EngineDataContent::Values { values } => DataContent::Values {
                values: values.iter().map(|value| value.into()).collect(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ContextualEnvironment {
    track_id: u64,
    contexts: BTreeMap<String, Identifier>,
    variables: BTreeMap<String, RawValue>,
}

impl From<&EngineContextualEnvironment> for ContextualEnvironment {
    fn from(contextual_environment: &EngineContextualEnvironment) -> Self {
        Self {
            track_id: contextual_environment.track_id() as u64,
            contexts: contextual_environment
                .contexts()
                .iter()
                .map(|(name, context)| (name.clone(), context.descriptor().identifier().into()))
                .collect(),
            variables: contextual_environment
                .variables()
                .iter()
                .map(|(name, value)| (name.clone(), value.into()))
                .collect(),
        }
    }
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

impl From<&EngineTransmissionDetails> for TransmissionDetails {
    fn from(details: &EngineTransmissionDetails) -> Self {
        Self {
            treatment: details.treatment.identifier().into(),
            host_treatment: (&details.host_treatment).into(),
            host_build: details.host_build.map(|id| id as u64),
            build_id: details.build_id as u64,
            label: details.label.clone(),
            name: details.name.clone(),
        }
    }
}
