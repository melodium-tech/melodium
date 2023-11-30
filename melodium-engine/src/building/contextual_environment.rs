use melodium_common::executive::{Context, Model, TrackId, Value};
use std::collections::HashMap;
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub struct ContextualEnvironment {
    track_id: TrackId,
    models: HashMap<String, Arc<dyn Model>>,
    variables: HashMap<String, Value>,
    contexts: HashMap<String, Arc<dyn Context>>,
    parent: Option<Arc<Self>>,
    me: Option<Weak<Self>>,
}

impl ContextualEnvironment {
    pub fn new(track_id: TrackId) -> Self {
        Self {
            track_id,
            models: HashMap::new(),
            variables: HashMap::new(),
            contexts: HashMap::new(),
            parent: None,
            me: None,
        }
    }

    pub fn base_on(&self) -> Self {
        Self {
            track_id: self.track_id,
            models: HashMap::new(),
            variables: HashMap::new(),
            contexts: self.contexts.clone(),
            parent: Some(self.me.as_ref().unwrap().upgrade().unwrap()),
            me: None,
        }
    }

    pub fn enriched_upper(&self) -> Self {
        if let Some(parent) = self.parent.as_ref() {
            Self {
                track_id: self.track_id,
                models: parent.models.clone(),
                variables: parent.variables.clone(),
                contexts: self.contexts.clone(),
                parent: parent.parent.clone(),
                me: None,
            }
        } else {
            Self {
                track_id: self.track_id,
                models: HashMap::new(),
                variables: HashMap::new(),
                contexts: self.contexts.clone(),
                parent: None,
                me: None,
            }
        }
    }

    pub fn commit(self) -> Arc<Self> {
        Arc::new_cyclic(|me| Self {
            me: Some(me.clone()),
            ..self
        })
    }

    pub fn track_id(&self) -> TrackId {
        self.track_id
    }

    pub fn add_model(&mut self, name: &str, model: Arc<dyn Model>) {
        self.models.insert(name.to_string(), model);
    }

    pub fn get_model(&self, name: &str) -> Option<&Arc<dyn Model>> {
        self.models.get(name)
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
