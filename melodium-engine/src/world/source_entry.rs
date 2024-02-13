use crate::building::BuildId;
use melodium_common::{descriptor::Identified, executive::Value};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub struct SourceEntry {
    pub descriptor: Arc<dyn Identified>,
    pub id: BuildId,
    pub params: HashMap<String, Value>,
}
