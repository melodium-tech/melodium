use crate::{
    building::{BuildId, ContextualEnvironment, HostTreatment},
    transmission::TransmissionDetails,
    world::InfoTrack,
};
use chrono::{DateTime, Utc};
use melodium_common::{
    descriptor::{Model, Treatment},
    executive::{Context, ModelId, TrackId, Value},
};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugLevel {
    None,
    Basic,
    Detailed,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub timestamp: DateTime<Utc>,
    pub kind: EventKind,
}

impl Event {
    pub fn new(kind: EventKind) -> Self {
        Self {
            timestamp: Utc::now(),
            kind,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EventKind {
    ModelBuilt {
        model: Arc<dyn Model>,
        parameters: HashMap<String, Value>,
        host_treatment: HostTreatment,
        host_build: Option<BuildId>,
        label: String,
    },
    ModelAdded {
        model_id: ModelId,
        model: Arc<dyn Model>,
    },
    ContinuousModelsStarted,
    ContinuousModelsFinished,
    TreatmentBuilt {
        treatment: Arc<dyn Treatment>,
        environment: ContextualEnvironment,
        host_treatment: HostTreatment,
        host_build: Option<BuildId>,
        build_id: BuildId,
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
        treatment: Arc<dyn Treatment>,
        host_treatment: HostTreatment,
        host_build: Option<BuildId>,
        build_id: BuildId,
        track_id: TrackId,
        label: String,
    },
    TreatmentFinished {
        treatment: Arc<dyn Treatment>,
        host_treatment: HostTreatment,
        host_build: Option<BuildId>,
        build_id: BuildId,
        track_id: TrackId,
        label: String,
    },
    DataSent {
        output: TransmissionDetails,
        track_id: TrackId,
        data: DataContent,
    },
    DataTransmitted {
        output: TransmissionDetails,
        input: TransmissionDetails,
        track_id: TrackId,
        data: DataContent,
    },
    DataReceived {
        input: TransmissionDetails,
        track_id: TrackId,
        data: DataContent,
    },
    OutputClosed {
        output: TransmissionDetails,
        track_id: TrackId,
    },
    InputClosed {
        input: TransmissionDetails,
        track_id: TrackId,
    },
}

#[derive(Debug, Clone)]
pub enum TrackCreation {
    Direct,
    Source {
        source: String,
        model_id: ModelId,
        parameters: HashMap<String, Value>,
        contexts: Vec<Arc<dyn Context>>,
    },
}

#[derive(Debug, Clone)]
pub enum DataContent {
    Undetailed,
    Count { count: usize },
    Values { values: Vec<Value> },
}
