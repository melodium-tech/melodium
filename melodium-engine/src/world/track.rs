
use futures::future::JoinAll;
use melodium_common::executive::{ResultStatus, TrackFuture, TrackId};

// We don't use id nor parent_id for now, but might be useful for reporting implementations.
#[allow(dead_code)]
pub struct InfoTrack {
    pub id: TrackId,
    pub parent_id: Option<TrackId>,
    pub ancestry_level: u64,
    pub results: Option<TrackResult>,
}

impl InfoTrack {
    pub fn new(id: TrackId, parent_id: Option<TrackId>, ancestry_level: u64) -> Self {
        Self {
            id,
            parent_id,
            ancestry_level,
            results: None,
        }
    }
}


// We don't use ancestry_level for now, but might be useful for scheduling implementations.
#[allow(dead_code)]
pub struct ExecutionTrack {
    pub id: TrackId,
    pub ancestry_level: u64,
    pub future: JoinAll<TrackFuture>,
}

impl ExecutionTrack {
    pub fn new(id: TrackId, ancestry_level: u64, future: JoinAll<TrackFuture>) -> Self {
        Self {
            id,
            ancestry_level,
            future,
        }
    }
}

pub enum TrackResult {
    AllOk(TrackId),
    NotAllOk(TrackId, Vec<ResultStatus>),
}

