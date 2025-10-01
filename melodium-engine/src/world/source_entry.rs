use crate::building::BuildId;
use melodium_common::{descriptor::Treatment, executive::Value};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub struct SourceEntry {
    pub descriptor: Arc<dyn Treatment>,
    pub id: BuildId,
    pub params: HashMap<String, Value>,
}
