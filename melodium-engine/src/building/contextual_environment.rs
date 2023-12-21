use melodium_common::executive::{Context, TrackId, Value};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct ContextualEnvironment {
    track_id: TrackId,
    contexts: HashMap<String, Arc<dyn Context>>,
    // Variables determined at track creation, coming from some processing involving context
    variables: HashMap<String, Value>,
}

impl ContextualEnvironment {
    pub fn new(track_id: TrackId) -> Self {
        Self {
            track_id,
            variables: HashMap::new(),
            contexts: HashMap::new(),
        }
    }

    pub fn base_on(&self) -> Self {
        Self {
            track_id: self.track_id,
            variables: HashMap::new(),
            contexts: self.contexts.clone(),
        }
    }

    pub fn track_id(&self) -> TrackId {
        self.track_id
    }

    pub fn add_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn variables(&self) -> &HashMap<String, Value> {
        &self.variables
    }

    pub fn add_context(&mut self, name: &str, context: Arc<dyn Context>) {
        self.contexts.insert(name.to_string(), context);
    }

    pub fn get_context(&self, name: &str) -> Option<&Arc<dyn Context>> {
        self.contexts.get(name)
    }
}
