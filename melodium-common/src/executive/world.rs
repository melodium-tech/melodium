use crate::executive::{Context, ContinuousFuture, ModelId, Outputs, TrackFuture, Value};
use async_trait::async_trait;
use core::fmt::Debug;
use std::{collections::HashMap, sync::Arc};

pub type TrackId = usize;
pub type TrackCreationCallback = Box<dyn FnOnce(Box<dyn Outputs>) -> Vec<TrackFuture> + Send>;

#[async_trait]
pub trait World: Debug + Send + Sync {
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
}
