use crate::building::BuildId;
use melodium_common::descriptor::Identified;
use std::sync::Arc;

#[derive(Debug)]
pub struct SourceEntry {
    pub descriptor: Arc<dyn Identified>,
    pub id: BuildId,
}
