
use std::collections::HashMap;
use std::sync::Arc;
use melodium_common::executive::{Model, TrackFuture};
use core::fmt::{Debug, Formatter, Result};
use super::{BuildId, FeedingInputs};

#[derive(Debug)]
pub enum StaticBuildResult {
    Model(Arc<dyn Model>),
    Build(BuildId),
}

pub struct DynamicBuildResult {
    pub prepared_futures: Vec<TrackFuture>,
    pub feeding_inputs: FeedingInputs,
}

impl DynamicBuildResult {

    pub fn new() -> Self {
        Self {
            prepared_futures: Vec::new(),
            feeding_inputs: HashMap::new(),
        }
    }
}

impl Debug for DynamicBuildResult {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("DynamicBuildResult")
         .field("feeding_inputs", &self.feeding_inputs)
         .field("prepared_futures", &self.prepared_futures.len())
         .finish()
    }
}
