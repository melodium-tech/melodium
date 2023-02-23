use crate::executive::{Context, ContinuousFuture, ModelId, Output, TrackFuture};
use async_trait::async_trait;
use core::fmt::Debug;
use std::collections::HashMap;

pub type TrackId = usize;

#[async_trait]
pub trait World: Debug + Send + Sync {
    fn add_continuous_task(&self, task: ContinuousFuture);
    async fn create_track(
        &self,
        id: ModelId,
        source: &str,
        contexts: Vec<Box<dyn Context>>,
        parent_track: Option<TrackId>,
        callback: Option<impl FnOnce(HashMap<String, Box<dyn Output>>) -> Vec<TrackFuture> + Send>,
    ) where
        Self: Sized;
}
