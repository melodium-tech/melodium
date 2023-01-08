
use std::sync::Arc;
use melodium_common::descriptor::Identified;
use crate::building::{BuildId};

#[derive(Debug)]
pub struct SourceEntry {
    pub descriptor: Arc<dyn Identified>,
    pub id: BuildId,
}
