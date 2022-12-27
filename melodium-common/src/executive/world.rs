
use std::collections::HashMap;
use async_trait::async_trait;
use crate::executive::{Context, ContinuousFuture, ModelId, Output, TrackFuture};

pub type TrackId = usize;

#[async_trait]
pub trait World {
    fn add_continuous_task(&self, task: ContinuousFuture);
    async fn create_track(&self, id: ModelId, source: &str, contexts: HashMap<String, Box<dyn Context>>, parent_track: Option<TrackId>, callback: Option<impl FnOnce(HashMap<String, Box<dyn Output>>) -> Vec<TrackFuture> + Send>) where Self: Sized;
}
