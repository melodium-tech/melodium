
use std::sync::Arc;
use melodium_common::descriptor::{Buildable, TreatmentBuildMode};
use crate::building::{BuildId};

#[derive(Debug)]
pub struct SourceEntry {
    pub descriptor: Arc<dyn Buildable<TreatmentBuildMode>>,
    pub id: BuildId,
}
