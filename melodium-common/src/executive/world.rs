use crate::{
    descriptor::Collection,
    executive::{Context, ContinuousFuture, Input, ModelId, Output, Outputs, TrackFuture, Value},
};
use async_trait::async_trait;
use core::fmt::{Debug, Display};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

pub type TrackId = usize;
pub type TrackCreationCallback = Box<dyn FnOnce(Box<dyn Outputs>) -> Vec<TrackFuture> + Send>;
pub type DirectCreationCallback = Box<
    dyn FnOnce(
            HashMap<String, Box<dyn Output>>,
            HashMap<String, Box<dyn Input>>,
        ) -> Vec<TrackFuture>
        + Send,
>;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Error => write!(f, "error"),
            Level::Warning => write!(f, "warning"),
            Level::Info => write!(f, "info"),
            Level::Debug => write!(f, "debug"),
            Level::Trace => write!(f, "trace"),
        }
    }
}

#[async_trait]
pub trait World: Debug + Send + Sync {
    fn collection(&self) -> Arc<Collection>;
    fn add_continuous_task(&self, task: ContinuousFuture);
    async fn create_track(
        &self,
        id: ModelId,
        source: &str,
        params: &HashMap<String, Value>,
        contexts: Vec<Arc<dyn Context>>,
        parent_track: Option<TrackId>,
        callback: Option<TrackCreationCallback>,
    );
    async fn log(&self, level: Level, label: String, message: String, track_id: Option<TrackId>);
}
