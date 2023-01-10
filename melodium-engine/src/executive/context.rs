use melodium_common::descriptor::Context as ContextDescriptor;
use melodium_common::executive::{Context as ExecutiveContext, Value};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Context {
    descriptor: Arc<ContextDescriptor>,
    values: HashMap<String, Value>,
}

impl Context {
    pub fn new(descriptor: Arc<ContextDescriptor>) -> Self {
        Self {
            descriptor,
            values: HashMap::new(),
        }
    }
}

impl ExecutiveContext for Context {
    fn descriptor(&self) -> Arc<ContextDescriptor> {
        self.descriptor.clone()
    }

    fn set_value(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    fn get_value(&self, name: &str) -> &Value {
        self.values.get(name).unwrap()
    }
}
